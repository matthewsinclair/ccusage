---
verblock: "2026-11-17T10:28:32Z:v0.47: vc - ccusage closed for the day (decision 23)"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**No thread or issue open: ccusage is closed for the day (decision 23), and every seat waits for hv's next word**

- When the next release is cut is hv's word: one cut with everything ready, never small ones. Before it, dc runs the kovacs gates at its final code tip (dc's todo 17), first checking its kovacs driver against devbin 0.1.26 (310f3d0). From devbin-vc: devbin 0.1.25 renders a formula's install as `cp_r`, and from 0.1.26 the release gates empty their build cache before a run when it holds more than `release.gate_cache.max` (12G unless the project declares another), so if one Ccusage build is larger, the project declares a larger size or every gate run builds cold.
- At the next session's start, vc puts two rulings on the account verb to hv. In Ccusage's record, hv's word of 2026-11-06 reverted the account verb's rename to `activate` (977589e), so `ccusage use NAME` stays the account verb, as 0.1.9 to 0.1.11 ship it. In devbin's record, hv's answer of 2026-11-14 08:49Z (devbin's decision 178(3)) reads: "Ccusage's use-to-activate rename is ruled work at Ccusage's next session." No seat renames anything until hv says which stands. devbin's record does not show whether devbin-vc's question of 2026-11-14 set the 2026-11-06 revert beside the rename, so vc puts the two to hv as they stand, then tells devbin-vc hv's answer, which devbin-vc records beside 178(3).

## Active Steel Threads

None open. `intent outstanding` shows what is.

## Holds

- Claude Usage: `richhickson/claudecodeusage` PR #18 (opened 2026-11-17, head 52b846a) makes Claude Usage name an item's account again when its login changes, and keep history per account. Until it is merged and released, and hv updates Claude Usage from that release, Claude Usage 1.18 keeps each Keychain item's first account name for its whole run: read accounts from ccusage, not from Claude Usage, and restart Claude Usage after a switch. vc checks the PR's state at each pickup (vc's hold 5).

## Notes

- hv's standing rulings are the decisions on hv's board. Among them: ccusage works for any Claude account type and shows whatever it can for that type; a login a switch drops lapses and is not revoked; engineering04 stays a profile until its allowance is exhausted; Herdr's verbs are `ccusage herdr install`, `remove` and `launch`; hv alone runs `ccusage use`, `login` and `which --use`, on vc's steps; Claude Usage comes from its mainline release, not hv's fork; a PR or commit to someone else's repository carries no (C) line; everything ready goes into one cut, never small releases.
- hv opens the ccusage panel with cmd-shift-u, a `tab.new` line hv keeps in Molt-matts' `config/herdr-app/keys.json` (decision 17). Herdr reads that file live, and ccusage never writes it. r in the panel forces a fetch (decision 18). The mod takes the key itself through its focus handle (4ae853f), so keys.json carries no `mod:ccusage` section.
- A Claude session started from a shell that exports `CLAUDE_SECURESTORAGE_CONFIG_DIR` reads that profile's own store, and no switch reaches it. `ccusage doctor` names such sessions, and a switch is refused while one reads the target's store.
- A `/login` typed inside a Claude session writes into whichever store that session reads, with no check of whose login it is. `ccusage login NAME` checks the email after, and a switch verifies the incoming login before it seats it.
- Before a cut, dc runs the project's gates on kovacs (Linux ARM64): Linux clippy sees code that only a macOS arm uses, which rhadamanth's runs miss, and the release config names no legs.
- A cut runs in hv's own terminal tab (`bin/devbin release X.Y.Z`), never as a `!` command. From vc's go until vc says it is published, the seats write nothing to the checkout, whiteboard verbs included, even when a first attempt stopped.
- All work lands on `main` in the one checkout; cc and ic build in their own worktrees and land by fast-forward. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). main goes to `local` once committed (hv's decision 4); nothing goes upstream but a release.
- Validation is hv's one layer of proof: vc reads the diff, and the builder's one measured run, in a Desk cargo slot, is the run. devbin-vc holds the Desk. Test only what changed; the full suite runs once, in the cut's gates.
- A landing that moves lines under another thread's acceptance-test rows leaves its citations wrong until they are re-cited; whoever sees it re-cites and tells the other seat. For an open thread `intent at lint` finds them; for a Completed one it skips the cited-line check (Intent issues 0665 and 0698), so run `intent doctor --scope all`, whose closed-contract-drift advisory names each row.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
- How Claude Code reads its login is in ST0003's Context (`intent st show ST0003`), with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
