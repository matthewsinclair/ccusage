---
verblock: "2026-11-03T10:01:44Z:v0.10: vc - issue work toward 0.1.6; no open thread"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Ccusage 0.1.6: ISSUE:0002's live check, then ISSUE:0006, ISSUE:0004 and ISSUE:0003; ISSUE:0005 after**

dc builds and vc validates, in hv's order: ISSUE:0002, ISSUE:0004, ISSUE:0003, ISSUE:0005. vc proposed ISSUE:0006 straight after ISSUE:0002, and dc holds it until hv confirms that slot.

- ISSUE:0002 (high): the fix at `812264d` moves ccusage's Keychain reads and writes to `/usr/bin/security`, as Claude Code does. Left: CI on both macOS targets, which needs a push, and the live check on hv's real accounts, with hv's OK. Caution for that check: a renewal through `security` leaves the item's partition list as `apple-tool:` alone, so brew's 0.1.5 prompts on that item afterwards. Run only the fixed build, by its full path, until 0.1.6 is installed.
- ISSUE:0006 (medium): adopt devbin. Commit hv's vendored install unedited, declare `rust` and `shell` and the real commands in `bin/.devbin/config.yaml`, and point `usage-rules.md` at `bin/devbin`. `.gitattributes` and `bin/` stay untracked until dc reaches it.
- ISSUE:0004 (medium): replace the release tooling's Python with a Rust `xtask`, shipped in 0.1.6, and remove the Python as soon as it is replaced.
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
- AGENTS.md's build and test lines are Intent's generic language text (Bats, cargo without `--locked`), and Intent offers no project hook. `usage-rules.md` points at the real checks instead, and at `bin/devbin` once ISSUE:0006 lands. intent-vc files the template gap as an Intent issue on hv's go.
