---
verblock: "2026-11-24T10:47:02Z:v0.49: vc - closed on hv's word (27); usage-rules line 18 deleted"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**No thread or issue open: every seat holds until hv's next word (decision 27 closed the 2026-11-24 session)**

- The next release waits for hv's word to cut. Before it, dc runs the kovacs gates at its final code tip (dc's todo 17). Its first step checks dc's kovacs driver against the devbin that the tree then carries (0.1.30 since 73b6eb9).
- From devbin-vc, for that cut: devbin 0.1.25 renders a formula's install as `cp_r`. From 0.1.26, the release gates empty their build cache before a run when it holds more than `release.gate_cache.max` (12G unless the project declares another). If one Ccusage build is larger, the project declares a larger size, or every gate run builds cold.

## Active Steel Threads

None open. `intent outstanding` shows what is.

## Holds

- Claude Usage: `richhickson/claudecodeusage` PR #18 (opened 2026-11-17, head 52b846a) makes Claude Usage name an item's account again when its login changes, and keep history per account. Until it is merged and released, and hv updates Claude Usage from that release, read accounts from ccusage, not from Claude Usage, and restart Claude Usage after a switch. vc checks the PR's state at each pickup (vc's hold 5).
- The push of main to `local`: decision 4 sends main there once committed, and nothing goes upstream but a release. The permission check refuses the push from vc's session, so hv runs `git -C /Users/matts/Devel/prj/Ccusage push local main`. vc checks `git log local/main..main` at each pickup.

## Notes

- hv's standing rulings are the decisions on hv's board, which every pickup prints.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- How Claude Code reads its login is in ST0003's Context (`intent st show ST0003`), with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
