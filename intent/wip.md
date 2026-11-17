---
verblock: "2026-11-17T07:34:18Z:v0.39: vc - the next release builds: ISSUE:0010, install's silence"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ISSUE:0010: the next release's fixes, building on vc's go (cc, with ic signing the text)**

cc's one unit fixes the issue's items 1, 2, 4 and 6 and rules out 3 and 5 with their reasons. It also takes in ic's nits from its read of ISSUE:0008's landed text, added to the issue's record as further items:
- one term for a session a switch leaves behind;
- three lines where a word can point two ways;
- predict's left-out lines, where `left_out` keeps to the accounts that are not projected, as README's version 1 says;
- the fall line in durations.
ic signs every changed string before the hand-over. One worktree, one Desk cargo slot; vc reads the diff and takes cc's run as the run.

**install's silence: the next release's Herdr fix (ic)**

While `herdr-app check` runs, `ccusage herdr install` says on stderr what it waits for, and one named constant bounds the wait. Past the bound, install reports the mod as not checked and exits 0. ic files it as an issue and builds it in one worktree and one Desk cargo slot, beside cc's unit; CHANGELOG.md is the one file both change.

When the next release is cut is hv's word: one cut with everything ready, never small ones.

## Active Steel Threads

None open. `intent outstanding` shows what is.

## Holds

- Claude Usage: `richhickson/claudecodeusage` PR #18 (opened 2026-11-17, head 52b846a) makes Claude Usage name an item's account again when its login changes, and keep history per account. Until it is merged and released, and hv updates Claude Usage from that release, Claude Usage 1.18 keeps each Keychain item's first account name for its whole run: read accounts from ccusage, not from Claude Usage, and restart Claude Usage after a switch. vc checks the PR's state at each pickup (vc's hold 5).

## Notes

- hv's standing rulings are the decisions on hv's board. Among them: ccusage works for any Claude account type and shows whatever it can for that type; a login a switch drops lapses and is not revoked; engineering04 stays a profile until its allowance is exhausted; Herdr's verbs are `ccusage herdr install`, `remove` and `launch`; hv alone runs `ccusage use`, `login` and `which --use`, on vc's steps; Claude Usage comes from its mainline release, not hv's fork; a PR or commit to someone else's repository carries no (C) line; everything ready goes into one cut, never small releases.
- hv opens the ccusage panel with cmd-shift-u, a `tab.new` line hv keeps in Molt-matts' `config/herdr-app/keys.json` (decision 17), which Herdr reads live. ccusage never writes keys.json.
- A Claude session started from a shell that exports `CLAUDE_SECURESTORAGE_CONFIG_DIR` reads that profile's own store, and no switch reaches it. `ccusage doctor` names such sessions, and a switch is refused while one reads the target's store.
- A `/login` typed inside a Claude session writes into whichever store that session reads, with no check of whose login it is. `ccusage login NAME` checks the email after, and a switch verifies the incoming login before it seats it.
- Before a cut, dc runs the project's gates on kovacs (Linux ARM64): Linux clippy sees code that only a macOS arm uses, which rhadamanth's runs miss, and the release config names no legs.
- A cut runs in hv's own terminal tab (`bin/devbin release X.Y.Z`), never as a `!` command. From vc's go until vc says it is published, the seats write nothing to the checkout, whiteboard verbs included, even when a first attempt stopped.
- All work lands on `main` in the one checkout; cc and ic build in their own worktrees and land by fast-forward. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). main goes to `local` once committed (hv's decision 4); nothing goes upstream but a release.
- Validation is hv's one layer of proof: vc reads the diff, and the builder's one measured run, in a Desk cargo slot, is the run. devbin-vc holds the Desk. Test only what changed; the full suite runs once, in the cut's gates.
- A landing that moves lines under another thread's acceptance-test rows breaks that thread's `intent at lint` until they are re-cited; whoever sees it re-cites and tells the other seat.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
- How Claude Code reads its login is in ST0003's Context (`intent st show ST0003`), with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
