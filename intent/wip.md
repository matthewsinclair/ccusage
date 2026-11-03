---
verblock: "2026-11-03T18:35:13Z:v0.16: vc - 0.1.6 out; 0002 and 0003 await install; 0005 next"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Ccusage after 0.1.6: ISSUE:0002 and ISSUE:0003 checks on the installed build, then ISSUE:0005**

dc builds and vc validates. Both ISSUE:0002 and ISSUE:0003 wait for hv's `brew upgrade ccusage` and `ccusage use brew`.

- ISSUE:0002 (high): one Acceptance item left. The first renewal by the installed 0.1.6 raises no prompt and leaves `apple-tool:` on the item (`security dump-keychain -a`, metadata only). Until it passes, keep to one build.
- ISSUE:0003 (medium): hv's live check at rollout, with shared mode on. A session launched with `ccusage dir`'s export reports the active profile's email from `claude auth status`, with the Claude Code-credentials item count unchanged; `ccusage use NAME` leaves launchd's `CLAUDE_SECURESTORAGE_CONFIG_DIR` naming NAME's profile dir; Claude Usage lists exactly five accounts and nothing prompts.
- ISSUE:0005 (medium): hv's Claude Usage fork (`~/Devel/forks/claudecodeusage`, upstream PR #15) folds in duplicate grouping, follows the account Claude Code is using, names the preferred item as Claude Code does, and stays read-only. It ships as one PR through upstream. The push and the PR edit need hv's OK.

## Active Steel Threads

None.

## Upcoming Work

- The ISSUE:0003 rollout: shared mode on, Molt's launcher export, restart the sessions, retire `ca646b28`.
- The tap's CI runs `brew style` and `brew audit --strict` again once the vendored devbin carries the fix for `lib/cmd/formula:424` (vc hold 3).

## Notes

- All work happens on `main` in the one checkout. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor).
- Pushes from an agent session are refused by Claude Code's auto-mode check, so hv runs the outward steps of a release: the tap push, `bin/devbin release`, and `bin/devbin formula publish`.
- Never point a ccusage profile at `~/.config/claude` before ISSUE:0002's renewal check passes: a renewal by an older build would strip `apple-tool:` from the main Claude Code item (vc watch-out 1).
- Never run a pre-fix local ccusage build against the real XDG config: it is ad hoc signed and in no item's access list, so it prompts once per profile. Test in a throwaway HOME and XDG tree.
- `security -i` falls back to the login keychain when the keychain it names does not exist. The ISSUE:0002 test refuses to run without its fixture keychain. It also splits a line longer than 4097 bytes, newline included, and runs the first fragment, which loses the keychain argument and any closing quote.
- AGENTS.md's build and test lines are Intent's generic language text (Bats, cargo without `--locked`), and Intent offers no project hook. `usage-rules.md` points at `bin/devbin` instead. Intent issue 0672 records that gap, with no fix scheduled.
