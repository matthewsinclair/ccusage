---
verblock: "2026-11-04T08:53:39Z:v0.2: vc - leaner intro; pointers to two decisions"
---

# ARCHITECTURE.md

How Ccusage is shaped, and why: each decision, what it rejected, and what would break if it were undone.

## Claude's Keychain items are reached only through `/usr/bin/security`

Decided on 2026-11-03 for ISSUE:0002, in `812264d`. The rules that follow from it are in `RULES.md`.

On macOS, Claude Code keeps each config directory's login in an item in the login keychain. ccusage reads that item, and writes it back when it renews the login, only by running `/usr/bin/security`, the way Claude Code 2.1.317 does. It never opens the item in-process through Security.framework. `src/keychain.rs` is the only code that touches these items.

### Why

- Every in-place write to a keychain item resets the item's partition list to the writer's own partition. The partition of `/usr/bin/security` is `apple-tool:`. A Developer ID build of ccusage has `teamid:76BQL8L47U`, and an ad hoc build has its `cdhash:`.
- Claude Code and Claude Usage read the items through `/usr/bin/security`. When an item's partition list lacks `apple-tool:`, each of those reads raises macOS's keychain-password prompt. "Always Allow" adds `apple-tool:` back, but only until the next write.
- Up to 0.1.5, ccusage renewed logins in-process. Each renewal stripped `apple-tool:`, so Claude Code and Claude Usage prompted once per item after every renewal. A renewal written by `security` keeps `apple-tool:`, so nothing that reads through `security` prompts, ccusage included.
- ccusage then needs no keychain trust of its own. A signed release, a source build and a development build behave the same.
- Repairing the partition list after an in-process write is not open to ccusage: `set-generic-password-partition-list` needs the login keychain's password.

### How

- Names follow Claude Code's own code. The account is `$USER`, else the login name, with `claude-code-user` for a name outside `[a-zA-Z0-9._-]`. The service is `Claude Code-credentials-` plus the first 8 hex digits of the SHA-256 of the NFC form of the config directory. If Claude Code changes either, `src/keychain.rs` follows it.
- A read is `security find-generic-password -a <account> -w -s <service>`. Only errSecItemNotFound means there is no item. A locked keychain or refused access is an error, never a reason to fall back to `.credentials.json`.
- A renewal sends `add-generic-password -U`, with the document as hex, to `security -i` on stdin. `-U` adds a missing item, and `security` falls back to the default keychain when the keychain it names does not exist. So the item is first found from its attributes alone, the write names the keychain that holds it, and the item is read back afterwards.
- `security -i` reads a line of up to 4097 bytes, newline included, whole. It splits a longer line and still runs the first fragment, which would write a truncated document. So a line over 4032 bytes, Claude Code's own limit, is refused: before the token exchange, so no refresh token is spent, and again at the write. At that size Claude Code falls back to a process argument. ccusage does not, because other processes can read arguments, eg through `ps`.
- Claude Code's refresh and write locks are taken around a renewal, as before.

### What would break if this were undone

- Any in-process write to a Claude item brings back a prompt for every Claude Code session, and for Claude Usage, on that item after each renewal.
- A ccusage from before this change reads in-process. Once `security` has written an item, that older build prompts for it, so mixing old and new builds on one Mac brings prompts back.

### Checked by

`renewal_write_keeps_claude_items_readable_by_security` in `src/auth.rs`, which is ignored by default and runs on both macOS CI targets. It creates an item in a disposable keychain as Claude Code does, runs ccusage's renewal write, and asserts that the item keeps `apple-tool:` and `/usr/bin/security` and that a missing item is never created. `docs/releasing.md` says how to run it.

## Decisions recorded in their issues

- Shared mode, one login per account under one Claude config: ISSUE:0003 holds the design and the rollout. It shipped in 0.1.6, and `src/shared.rs` is its home.
- The release pipeline, one `bin/devbin release` whose `after:` step signs, notarizes and publishes: ISSUE:0004 and `dd30c82` hold the decisions, and `docs/releasing.md` says how it runs.
