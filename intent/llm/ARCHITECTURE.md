---
verblock: "2026-11-13T10:08:59Z:v0.3: cc - shared mode's one live login, which a switch moves (ST0003)"
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

## Shared mode keeps one live login, which a switch moves

Decided on 2026-11-13 for ST0003, in `9afdf86` (the move) and `eea2dea` (the verbs), with `2019b93` and `ed77832`. The rules that follow from it are in `RULES.md`.

In shared mode, every Claude Code session on the machine reads one login: the one in the shared config's own store, its Keychain item on macOS or else its `.credentials.json`. That login is the active profile's, and every other profile's login is parked in that profile's own store. `ccusage use NAME` switches by moving logins, not by pointing sessions somewhere else: it parks the active profile's login in its own store and seats NAME's in the shared one. A running session reads its store again, so it follows a switch with no logout, login or restart. `src/seat.rs` is the move's home, and `src/switch.rs` the verbs'.

### Why

- WP01's trial, on Claude Code 2.1.323 on 2026-11-13, showed a running session holding a login that another session had stored for its config 73 seconds later at most, with no prompt, notice or restart. Its next request carried on the conversation after one 404 and a replay of the conversation, which Claude Code made itself within its retry allowance.
- From 0.1.6 to 0.1.8, each session was given its login's directory at launch, through `CLAUDE_SECURESTORAGE_CONFIG_DIR`. A running session read that directory until it ended, so a switch reached only new sessions, and a shell that kept a stale value launched sessions on the wrong account. Switching meant restarting every session.
- A login is moved, never copied, so each account keeps one login, and Claude Usage, which lists every Claude Code Keychain item, shows each account once (ISSUE:0003).

### How

- Only `claudeAiOauth` moves. Every other key of each store's document stays as it was, in its order.
- A move holds ccusage's own switch lock, and Claude Code's refresh and storage-write locks on every store it touches, for the whole move.
- A pure plan turns the stores' documents and the request into the ordered writes, or refuses before any write. The outgoing login is parked first and the incoming one seated after, so no account is ever without a login. Each write is read back, and the step it reached is journaled in `move.json` under `XDG_STATE_HOME/ccusage` before the next. The next `ccusage use` finishes a move that was interrupted, and until then every verb that would read a moving login refuses.
- Before the shared login leaves the shared store, it is verified as the outgoing profile's: by a usage cache that verified that exact token where it lives, else by one call to Claude's profile endpoint. A login that cannot be verified is not moved.
- `seat.json`, in the same state directory, names the seated profile, and every verb reads a profile's login where it lives.
- The first switch on 0.1.8's layout, where the shared config keeps a login of its own, replaces that login only when Claude can no longer renew it, or when its account keeps another login in a profile. A login it replaces is left to lapse, not revoked. Otherwise the switch refuses, because that login may be its account's only one.
- `ccusage use --none` returns the seated login to its profile and leaves the shared store with none, the layout an older ccusage reads.
- On macOS, `use` clears launchd's `CLAUDE_SECURESTORAGE_CONFIG_DIR`, so a session started from a GUI app reads the shared login as well. `use --none` points it at the active profile's directory, as 0.1.8 did.
- A switch relies on two of Claude Code's flags standing at their defaults: the sibling-login watch (`tengu_streamed_thimble`, on) and the session-owner prompt (`tengu_bubbly_fountain`, off). `ccusage doctor` reports either one that is off its default. It also reports where each login is, a move left unfinished, whose the shared login is, and each running session launched with a directory that does not follow a switch.

### What would break if this were undone

- With a login directory per launch again, running sessions stop following a switch, and a stale shell variable puts new sessions on the wrong account.
- With logins copied instead of moved, an account has two logins again, and Claude Usage lists it twice.
- Without the journal, a move interrupted between its writes leaves stores that no later run can tell from a deliberate layout, so nothing can finish the move or undo it.
- Without Claude Code's locks, a move that races a renewal can write back the document the renewal replaced, and lose the renewed login.

### Checked by

- WP01's trial, recorded in ST0003's WP01.
- The move's tests in `src/seat.rs` (ST0003's AT-02.1 to AT-02.10), among them `a_move_keeps_claude_items_readable_by_security`, which is ignored by default and works on a disposable keychain, as `docs/releasing.md` says.
- The verbs' tests (ST0003's AT-03.1 to AT-03.10), in `tests/cli.rs`, `src/switch.rs`, `src/seat.rs` and `src/usage.rs`.
- Doctor's tests of the layout (ST0003's AT-04.1 to AT-04.4, AT-04.10 and AT-04.11), in `src/doctor.rs`, `src/sessions.rs`, `src/usage.rs` and `tests/cli.rs`.

## Decisions recorded in their issues

- Shared mode's first form, from 0.1.6 to 0.1.8, which gave each session its login's directory at launch: ISSUE:0003 holds its design and rollout. Its one live login, from 0.1.9, is the decision above. `src/shared.rs` holds what both forms share.
- The release pipeline, one `bin/devbin release` whose `after:` step signs, notarizes and publishes: ISSUE:0004 and `dd30c82` hold the decisions, and `docs/releasing.md` says how it runs.
