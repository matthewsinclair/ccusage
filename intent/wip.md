---
verblock: "2026-11-04T19:53:48Z:v0.23: vc - EOD fold; ISSUE:0003 step 1 done, step 2 is Molt 1.4.3"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Ccusage after 0.1.8: ISSUE:0003's rollout**

dc builds and vc validates.

- ISSUE:0003 (medium): step 1 is done (shared mode on, engineering01 active; verified by vc). The rollout waits for Molt 1.4.3 (Molt issue 0088, step 2: the shell and Molt's LaunchAgent read `ccusage dir` at each start and login, exporting only an existing directory), whose cut stopped on a stranded macOS runner and goes again on 2026-11-05; then for estate slot `01M6K31TH20532K7NYN48GJE86`, step 3's restart of every Claude session, which pplr-vc grants with hv's go. vc checks each step against the issue's Rollout and Acceptance.

## Active Steel Threads

None.

## Upcoming Work

- The tap's CI runs `brew style` and `brew audit --strict` again once a devbin release fixes `lib/cmd/formula:424` and Ccusage vendors it (vc hold 3). devbin-vc and devbin-dc have the fix.

## Notes

- All work happens on `main` in the one checkout. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). hv pushes, and hv runs a release: one command, `bin/devbin release X.Y.Z` (`docs/releasing.md`).
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
