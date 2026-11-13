---
verblock: "2026-11-13T09:17:58Z:v0.29: vc - localfold for hv's compact; ST0002 built, ST0003 at WP04"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0003: live account switching (hv, 2026-11-13), built by cc**

`ccusage use` moves the active account's login into the shared config's own store, and running sessions follow at their next request. WP01 to WP03 are on main and closed. Banked on `cc/ST0003`, with one Desk run queued: vc's three follow-ups on WP03 (b377106) and each report's plan fields for `predict` (02a5967). Next: WP04 (doctor, with what an account type gives when it has no usage access or no windows), under way on `cc/ST0003-WP04`, then WP05 (docs and the decision record). WP06 is hv's rollout, in the closing sequence below, and it closes ISSUE:0003.

**ST0002: the which and predict verbs and the Herdr mod (hv, 2026-11-13), built by ic**

WP01 to WP04 are on main and closed. Built on ic's stack, with one Desk run queued at 4589212: WP03's text follow-ups, WP05 (`predict`), WP06 (the TUI's `next` marker), WP08 (the Herdr mod, `ccusage herdr install` and `remove`, and the TUI asking again once a cache lifetime) and WP07 (README and CHANGELOG). They land together after vc's reads. `predict` groups accounts by plan tier once cc's plan fields land, before the cut. `which --json` is version 1, and geodica-estate has it.

**Ccusage 0.1.9: one cut, everything in (hv, 2026-11-13)**

- The closing sequence, which vc writes for hv once both threads have landed: quit every Claude session; `bin/devbin install` and `bin/devbin use opt` (route C past devbin#0348); the first `ccusage use NAME` (ST0003's adoption), read against its message and doctor; fresh shells, so that no session reads a profile's own store through `CLAUDE_SECURESTORAGE_CONFIG_DIR`; `ccusage herdr install` and hv's check in his Herdr (AC-08.3); `bin/devbin release 0.1.9`; after the publish, `brew upgrade ccusage`, `bin/devbin use brew` and T10's version check. The opt step waits for this sequence, since until 0.1.9's keg exists `use brew` refuses the 0.1.8 keg.
- ISSUE:0007 (medium): the wiring passed; dc closes it after T10's proof. devbin#0349 (LICENSE back in the archives) waits for a later devbin.
- ISSUE:0003 (medium): closes at ST0003/WP06; its last acceptance line reads each account once in Claude Usage (hv's decision 3).

## Active Steel Threads

- ST0002: Add 'which' verb to indicate the best account to use to maximise token usage across accounts. WIP; WP05 to WP08 built, one run queued.
- ST0003: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow. WIP; cc at WP04.

## Upcoming Work

- The tap's CI runs `brew style` and `brew audit --strict` again once 0.1.9's formula is published by devbin 0.1.20 or later. The vendored devbin already carries the fix; the tap's live formula was rendered by 0.1.19 (vc hold 3).

## Notes

- hv's standing rulings for this work are decisions 1 to 4 on hv's board: ccusage works for any Claude account type and shows whatever it can for that type; a login a switch drops lapses and is not revoked; engineering04 stays a profile until its allowance is exhausted; Herdr's verb is `ccusage herdr install` and `remove`.
- All work lands on `main` in the one checkout; cc and ic build in their own worktrees and land by fast-forward. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). hv pushes, or says when a seat may: main goes to `local` on hv's word, and nothing goes upstream before the release. hv runs a release: one command, `bin/devbin release X.Y.Z` (`docs/releasing.md`).
- Validation is hv's one layer of proof: vc reads the diff, and the builder's one measured run, in a Desk cargo slot, is the run. devbin-vc holds the Desk. Test only what changed; the full suite runs once, in the cut's gates.
- A landing that moves lines under another thread's acceptance-test rows breaks that thread's `intent at lint` until they are re-cited; whoever sees it re-cites and tells the other seat.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
- How Claude Code reads its login is in ST0003's Context, with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
