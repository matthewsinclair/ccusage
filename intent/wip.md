---
verblock: "2026-11-04T11:41:25Z:v0.22: vc - fix-all validated; ISSUE:0003's rollout left"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Ccusage after 0.1.8: ISSUE:0003's rollout**

dc builds and vc validates.

- ISSUE:0003 (medium): the rollout waits for hv's step 1 (shared mode on); for Molt 1.4.3's step 2, which pplr-dc builds after Molt 1.4.2, with the shell and Molt's LaunchAgent running `ccusage dir` at each start and login; and for estate slot `01M6K31TH20532K7NYN48GJE86`, step 3's restart of every Claude session, which pplr-vc grants with hv's go. vc checks each step against the issue's Rollout and Acceptance.

## Active Steel Threads

None.

## Upcoming Work

- The tap's CI runs `brew style` and `brew audit --strict` again once a devbin release fixes `lib/cmd/formula:424` and Ccusage vendors it (vc hold 3). devbin-vc and devbin-dc have the fix.

## Notes

- All work happens on `main` in the one checkout. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). hv pushes, and hv runs a release: one command, `bin/devbin release X.Y.Z` (`docs/releasing.md`).
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
