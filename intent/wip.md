---
verblock: "2026-11-13T07:43:04Z:v0.28: vc - localfold for hv's compact; three builders mid-thread"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0003: live account switching (hv, 2026-11-13), built by cc**

`ccusage use` is to move the active account's login into the shared config's own store, so running sessions follow at their next request; WP01's trial proved Claude Code does, and its record is in the work package. The one use function, `switch::to`, is on main. Next: WP02 (the login move) on branch `cc/ST0003`, then WP03 with WP04 beside it, then WP05. WP06 is hv's rollout from the opt tree, whose steps vc writes, and it closes ISSUE:0003.

**ST0002: the which and predict verbs and the Herdr mod (hv, 2026-11-13), built by ic**

WP01 (typed usage windows) is on main and passed; it closes once its acceptance-test rows cite lines that carry their ids (`intent at lint ST0002`). WP02 (the rule and the `[which] cap` key) passed on branch `ic/ST0002-02` and lands once cc signs the rule's meaning, with one more commit that makes the scoped share lenient. The `which --json` contract, version 1, is final and with geodica-estate. Next: WP04 (history, under way), then WP03, WP05, WP06, WP08 (the Herdr mod) and WP07 (docs).

**Ccusage 0.1.9: one cut, everything in (hv, 2026-11-13)**

- ISSUE:0007 (medium): the wiring passed vc's verdict (9cc5f93, 78cabdc, 802de63; docs line 782584d). Left for the cut, with hv: the kegs at 0.1.9's publish, `bin/devbin use brew`, and T10's proof. Then dc closes the issue, and vc tells devbin-vc, which closes devbin#0326 once pplr#0001 and cldr#0001 have closed too.
- Cut blocker, devbin#0348: devbin 0.1.23's doctor fails on a Mac that still holds the pre-contract 0.1.8 keg, and the cut's gates run doctor. With hv: (A) go for devbin 0.1.24, vendored before the cut; (B) a ruling that the cut accepts doctor's red rows; or (C, recommended) `bin/devbin install` and `bin/devbin use opt` just before the cut, the same first steps as T10 and ST0003's rollout, then `bin/devbin use brew` after `brew upgrade` to 0.1.9. devbin#0349 is why LICENSE left the archives; it can come back if 0.1.24 fixes it before the cut.
- ISSUE:0003 (medium): closes at ST0003/WP06's rollout (vc todo 4, dc hold 6).
- hv is returning engineering04 to the free plan, which leaves four accounts on one plan.

## Active Steel Threads

- ST0002: Add 'which' verb to indicate the best account to use to maximise token usage across accounts. WIP; ic at WP04, WP02 waiting to land.
- ST0003: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow. WIP; cc at WP02.

## Upcoming Work

- The tap's CI runs `brew style` and `brew audit --strict` again once 0.1.9's formula is published by devbin 0.1.20 or later. The vendored devbin already carries the fix; the tap's live formula was rendered by 0.1.19 (vc hold 3).

## Notes

- All work lands on `main` in the one checkout; cc and ic build in their own worktrees and land by fast-forward. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). hv pushes, and hv runs a release: one command, `bin/devbin release X.Y.Z` (`docs/releasing.md`).
- Validation is hv's one layer of proof: vc reads the diff, and the builder's one measured run, in a Desk cargo slot, is the run. devbin-vc holds the Desk. Test only what changed; the full suite runs once, in the cut's gates.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
- How Claude Code reads its login is in ST0003's Context, with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
