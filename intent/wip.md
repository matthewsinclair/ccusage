---
verblock: "2026-11-13T07:04:13Z:v0.27: vc - ST0003/WP01's trial passed; cc and ic briefed for hv's start"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0003: live account switching (hv, 2026-11-13)**

WP01 passed on 2026-11-13: in hv's trial on a scratch config, a running session held a login that another session stored 73 s earlier, with no restart and no prompt, and Claude Code replayed the conversation by itself when the new account's first request got a 404 `thread_not_found`. The record is in the work package. So `ccusage use` is to move the active account's login into the shared config's own store, and running sessions follow at their next request. Next, on hv's word to proceed: hv starts cc, whose brief is in its inbox, and cc builds WP02 to WP05. WP06, hv's rollout from the opt tree, closes ISSUE:0003.

**ST0002: the which and predict verbs and the Herdr mod (hv, 2026-11-13)**

Reviewed by hv: eight work packages and 17 criteria. `ccusage which` names the account whose weekly reset is soonest and that still has headroom, `ccusage predict` projects the allowance that will be lost, and WP08 is a Herdr mod that draws the panel natively from their JSON, with one refresh policy for it and the TUI. ic builds it once hv starts ic, whose brief is in its inbox, and cc signs the rule, predict's arithmetic and what the JSON means.

**Ccusage 0.1.9: one cut, everything in (hv, 2026-11-13)**

- ISSUE:0007 (medium): hv said go on 2026-11-13, and vc briefed dc at 06:45Z. devbin 0.1.23, vendored since 3c1055a, carries every unit of devbin#0326 (devbin-vc). dc wires the three parts in the issue's Fix outline: the `install:` block in `bin/.devbin/config.yaml`, the `--version` reader the contract spells, and the formula on `@@INSTALL@@` on macOS and Linux with the archive's binary at `bin/ccusage` (vc todo 5). Then hv runs the full suite, and T10's proof runs with hv. When the issue closes, vc tells devbin-vc, which closes devbin#0326 once pplr#0001 and cldr#0001 have closed too.
- ISSUE:0003 (medium): read from the live processes at 2026-11-13 06:14Z, all 25 Claude sessions carry a profile's directory (23 hello's, 2 engineering01's), and the main config's own login, `ca646b28`, was last written 2026-11-11 12:38Z, so nothing renews it. One of the two launched from a shell older than the last `ccusage use`, which is the launcher gap. With WP01 passed, it closes at ST0003's rollout, with the shared config's login kept as the live one (vc todo 4, dc hold 6).
- hv is returning engineering04 to the free plan, which leaves four accounts on one plan.

## Active Steel Threads

- ST0002: Add 'which' verb to indicate the best account to use to maximise token usage across accounts. WIP; ic starts at WP01 when hv starts ic.
- ST0003: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow. WIP; WP01 done, and cc starts at WP02 when hv starts cc.

## Upcoming Work

- The tap's CI runs `brew style` and `brew audit --strict` again once 0.1.9's formula is published by devbin 0.1.20 or later. The vendored devbin already carries the fix; the tap's live formula was rendered by 0.1.19 (vc hold 3).

## Notes

- All work happens on `main` in the one checkout. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). hv pushes, and hv runs a release: one command, `bin/devbin release X.Y.Z` (`docs/releasing.md`).
- Validation is hv's one layer of proof: vc reads the diff, and the author's one measured run is the run. cargo and bats run in a slot from the estate desk, which devbin-vc holds. Test only what changed; hv runs the full suite by hand before the cut.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
- How Claude Code reads its login is in ST0003's Context, with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
