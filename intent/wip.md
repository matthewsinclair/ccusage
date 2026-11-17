---
verblock: "2026-11-17T08:34:33Z:v0.41: vc - the 0.1.11 cut, on hv's word (decision 19)"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**0.1.11: the cut, on hv's word (decision 19)**

0.1.11's content is on main, under the CHANGELOG's 0.1.11 (unreleased): ISSUE:0010, ISSUE:0011 and ISSUE:0012, all closed. In order:
- dc's kovacs gates at the final code tip, which first build ISSUE:0010 item 2's Linux branch.
- The Desk's slot for the cut, then vc's go. From the go until vc says it is published, the seats write nothing to the checkout.
- hv runs `bin/devbin release 0.1.11` in hv's own tab.
- After the publish: vc tells devbin-vc, hv runs `brew upgrade ccusage`, and the seats fold.

## Active Steel Threads

None open. `intent outstanding` shows what is.

## Holds

- Claude Usage: `richhickson/claudecodeusage` PR #18 (opened 2026-11-17, head 52b846a) makes Claude Usage name an item's account again when its login changes, and keep history per account. Until it is merged and released, and hv updates Claude Usage from that release, Claude Usage 1.18 keeps each Keychain item's first account name for its whole run: read accounts from ccusage, not from Claude Usage, and restart Claude Usage after a switch. vc checks the PR's state at each pickup (vc's hold 5).

## Notes

- hv's standing rulings are the decisions on hv's board. Among them: ccusage works for any Claude account type and shows whatever it can for that type; a login a switch drops lapses and is not revoked; engineering04 stays a profile until its allowance is exhausted; Herdr's verbs are `ccusage herdr install`, `remove` and `launch`; hv alone runs `ccusage use`, `login` and `which --use`, on vc's steps; Claude Usage comes from its mainline release, not hv's fork; a PR or commit to someone else's repository carries no (C) line; everything ready goes into one cut, never small releases.
- hv opens the ccusage panel with cmd-shift-u, a `tab.new` line hv keeps in Molt-matts' `config/herdr-app/keys.json` (decision 17), and refreshes it with r, from that file's `"mod:ccusage": {"r": "ccusage.refresh"}` (decision 18). Herdr reads the file live; ccusage never writes it.
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
