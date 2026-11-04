---
verblock: "2026-11-04T09:22:40Z:v0.18: vc - after 0.1.8; 0002 on hv's word"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Ccusage after 0.1.8: ISSUE:0002's last word, ISSUE:0003's rollout, then ISSUE:0005**

dc builds and vc validates.

- ISSUE:0002 (high): waits only on hv's word that the 2026-11-04 05:17Z renewal raised no prompt; its Keychain metadata holds (vc todo 2). dc then closes it.
- ISSUE:0003 (medium): the rollout waits for hv to turn shared mode on, and for a slot from the estate's desk, since its step 3 restarts every Claude session on the Mac. vc checks each step against the issue's Rollout and Acceptance.
- ISSUE:0005 (medium): hv's Claude Usage fork (`~/Devel/forks/claudecodeusage`, upstream PR #15), which dc builds after ISSUE:0003 as one PR. The push and the PR edit need hv's OK.

## Active Steel Threads

None.

## Upcoming Work

- dc's todos P1 to P7: critic-shell's findings in `bin/.devbin/cmd/`.
- The tap's CI runs `brew style` and `brew audit --strict` again once a devbin release fixes `lib/cmd/formula:424` and Ccusage vendors it (vc hold 3).

## Notes

- All work happens on `main` in the one checkout. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). hv pushes, and hv runs a release: one command, `bin/devbin release X.Y.Z` (`docs/releasing.md`).
- Never point a ccusage profile at `~/.config/claude` before hv confirms ISSUE:0002's renewal raised no prompt (vc watch-out 1).
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
