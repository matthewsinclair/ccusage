---
verblock: "2026-11-03T10:29:41Z:v0.15: vc - dc builds ISSUE:0004; ISSUE:0002 waits on CI and 0.1.6"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Ccusage 0.1.6: ISSUE:0004, then ISSUE:0003; ISSUE:0005 after**

dc builds and vc validates, in hv's order: ISSUE:0004 (in progress), ISSUE:0003, ISSUE:0005.

- ISSUE:0002 (high): the fix at `812264d` moves ccusage's Keychain reads and writes to `/usr/bin/security`, as Claude Code does. Left: CI on both macOS targets at the next push of main, and the renewal check once 0.1.6 is installed. Until then keep to one build. A renewal through `security` leaves an item's partition list as `apple-tool:` alone, so brew's 0.1.5 prompts on that item afterwards. A renewal by 0.1.5 makes everything that reads through `security` prompt.
- ISSUE:0004 (medium): replace the release tooling's Python with a Rust `xtask`, shipped in 0.1.6, and remove the Python as soon as it is replaced. It also removes `tools/test-keychain-upgrade` and `tests/keychain-upgrade.swift`. It runs the release's local validation through `bin/devbin`, adds actionlint and the `xtask` checks to `bin/.devbin/config.yaml`, and settles devbin doctor's version note. ISSUE:0004's Decision says how.
- ISSUE:0003 (medium): one login per account. Sessions keep `~/.config/claude` and borrow the active profile's login through `CLAUDE_SECURESTORAGE_CONFIG_DIR`; `ccusage use` copies the profile's `oauthAccount` into the shared `.claude.json` and sets launchd's copy of the variable.
- ISSUE:0005 (medium): hv's Claude Usage fork (`~/Devel/forks/claudecodeusage`, upstream PR #15) folds in duplicate grouping, follows the account Claude Code is using, names the preferred item as Claude Code does, and stays read-only. It ships as one PR through upstream, not in 0.1.6. The push and the PR edit need hv's OK.

## Active Steel Threads

None.

## Upcoming Work

- hv cuts 0.1.6 once ISSUE:0002, ISSUE:0004 and ISSUE:0003 work on rhadamanth, with the release running on the new `xtask` tooling.
- After 0.1.6: the ISSUE:0003 rollout (shared mode on, Molt's launcher export, restart the sessions, retire `ca646b28`).

## Notes

- All work happens on `main` in the one checkout (hv, 2026-11-03). Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor).
- Never point a ccusage profile at `~/.config/claude` before ISSUE:0002 ships: a renewal would strip `apple-tool:` from the main Claude Code item (vc watch-out 1).
- Never run a pre-fix local ccusage build against the real XDG config: it is ad hoc signed and in no item's access list, so it prompts once per profile. Test in a throwaway HOME and XDG tree.
- `security -i` falls back to the login keychain when the keychain it names does not exist. The ISSUE:0002 test refuses to run without its fixture keychain. It also splits a line longer than 4097 bytes, newline included, and runs the first fragment, which loses the keychain argument and any closing quote.
- AGENTS.md's build and test lines are Intent's generic language text (Bats, cargo without `--locked`), and Intent offers no project hook. `usage-rules.md` points at `bin/devbin` instead. Intent issue 0672 records that gap, with no fix scheduled.
