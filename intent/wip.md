---
verblock: "2026-11-17T05:05:14Z:v0.37: vc - the live repair runs on hv's go; a fresh vc checks it and closes"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0003: the live repair, running on hv's go of 2026-11-17 (ISSUE:0008, which is WP06's rollout)**

hello was seated in the shared config on 2026-11-15 at 19:19Z by hv's own hand, so the rollout has begun. The fleet then restarted from zsh shells alive since 2026-11-12, each still exporting `CLAUDE_SECURESTORAGE_CONFIG_DIR` as Molt's `env.sh` set it at that shell's start. At 17:02Z on 2026-11-16, 24 of 29 sessions were pinned to hello's own store and 1 to engineering01's, so no switch reaches them, hello has two logins, a switch away from hello is refused, and engineering01's own store holds a login as hello. ISSUE:0008 (high) has the diagnosis. hv, 2026-11-16: "let's get ready to run this first thing tomorrow" (decision 8). On 2026-11-17 hv gave the go (decision 11): every session ends first, since the last three were pinned too, and a fresh ccusage-vc checks the result (vc's todo 13). devbin-vc has it: the Desk grants nothing into the window, and nothing after it until vc says the estate is clear.

Until it runs: no `ccusage login`, `use` or `which --use`, which would take the login from under the pinned sessions. If hello runs out first, one `/login` as an account with allowance, typed in any one session pinned to hello's store, moves all of them, and step 2 below clears it up.

The steps, hv's hands, vc checking from metadata:

1. Every Claude session quits. Then every shell that starts Claude loses the pin: `unset CLAUDE_SECURESTORAGE_CONFIG_DIR` in each pane. The shells belong to Herdr Core, which has run since 2026-11-12 and outlived the Herdr app's restart on 2026-11-15, so reopening the app clears nothing. Herdr can type the line into every pane (`hc ls --json` lists 91 terminals, `hc send <address> "<text>"` types into one), once no pane has a Claude prompt in it. Check in a pane: `echo ${CLAUDE_SECURESTORAGE_CONFIG_DIR-unset}; launchctl getenv CLAUDE_SECURESTORAGE_CONFIG_DIR` prints `unset`, then an empty line.
2. With no session running: `ccusage login hello`, then `ccusage login engineering01`, the browser signed in to that account each time (ccusage checks the email after, and says when it is another's). Then `ccusage doctor` exits 0 with no line about a pinned session, and `ccusage usage --all` reads all five accounts.
3. One session starts, ccusage-vc, and nothing else. vc checks that its own process carries no pin.
4. The live switch (AC-06.2), with that one session open: `ccusage use engineering00`, and vc's session answers its next prompt with `/status` naming engineering00; then `ccusage which --use`, and it answers again on the account `which` named. No restart either time.
5. The fleet starts. vc checks every `claude` process as it comes up: none carries the variable, and doctor shows one login an account. hv checks that Claude Usage lists each account once with nothing prompting (AC-06.1).
6. dc closes ISSUE:0003; vc satisfies AC-06.1 and AC-06.2 and closes ST0003/WP06, then tells devbin-vc and geodica-estate that the estate is clear.

Back-out: `ccusage use --none` puts the active login back in its profile and points launchd at it, as 0.1.8 kept them. It is refused while the active account's own store holds a login, so it works only after step 2.

**ST0003: ISSUE:0008's code, built by cc for the next cut**

cc builds the issue's fix outline items 1 to 5 with its two nits from WP04's verdict, in one worktree and one Desk slot (vc's brief, 2026-11-16): doctor says whose login each own store holds; the two-logins finding and usage's refusal give the remedy in the order that works and name the pinned sessions; `login`, `use`, `use --none` and `which --use` refuse a move that would empty a store a running session reads; a reporting verb says when its own shell exports the variable; predict's rate keeps a failed account's samples. Items 6 and 7 (the pinned count on which, predict and the TUI, and a fall inside a window) are interface text and wait on hv's word about ic. vc reads cc's diff and takes its run as the run.

**ST0002: the which and predict verbs and the Herdr mod (hv, 2026-11-13), built by ic**

In 0.1.9. WP01 to WP07 are closed. Left: AC-08.3, hv's second look at the panel. ic's fix for the weekly bars landed as 9be58a6, green in its Desk slot. hv installs from a build at or after it (`ccusage herdr install`, `--force` only if it refuses) and looks again, which also checks that the pane refreshes by itself. Open with hv: `herdr/ccusage/preview.png` (vc recommends hv's capture, cropped with the names blurred), and, for ic, a way into the mod from inside Herdr, which has no menu for a mod pane. Then vc closes WP08, and AC-00.1 is the thread's sign-off. ic's session has not run since 2026-11-13.

## Active Steel Threads

- ST0002: Add 'which' verb to indicate the best account to use to maximise token usage across accounts. WIP; WP08 waits on AC-08.3, hv's look.
- ST0003: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow. WIP; WP06 is the live repair above.

## Upcoming Work

- ISSUE:0009 (medium), dc: LICENSE is back in each release archive and installed tree, on main. Left: before the cut, with the kovacs gates, a scratch-HOME `bin/devbin install` listing the opt tree and its manifest; after the cut's `brew upgrade ccusage`, doctor's tree row on rhadamanth's keg and on kovacs's, where dc closes it.
- ISSUE:0003 (medium) closes at ST0003/WP06; its last acceptance line reads each account once in Claude Usage (hv's decision 3).
- One cut carries ISSUE:0008's code, cc's two nits and ISSUE:0009, on hv's word, with dc's run of the gates on kovacs first.
- Open with hv from ISSUE:0008: whether ic starts or cc takes the interface text; and whether the launch-time fix goes to Molt's or Intent's desk (a shell that outlives a switch keeps its pin, since `env.sh` runs at shell start alone).

## Notes

- hv's standing rulings are decisions 1 to 8 on hv's board: ccusage works for any Claude account type and shows whatever it can for that type; a login a switch drops lapses and is not revoked; engineering04 stays a profile until its allowance is exhausted; Herdr's verb is `ccusage herdr install` and `remove`; 0.1.9 was cut before the rollout; hv alone runs `ccusage use`, `login` and `which --use`, on vc's steps.
- A `/login` typed inside a Claude session writes into whichever store that session reads, with no check of whose login it is. `ccusage login NAME` checks the email after.
- Before a cut, dc runs the project's gates on kovacs (Linux ARM64): Linux clippy sees code that only a macOS arm uses, which rhadamanth's runs miss, and the release config names no legs.
- A cut runs in hv's own terminal tab (`bin/devbin release X.Y.Z`), never as a `!` command. The seats write nothing to the checkout, whiteboard verbs included, until vc says it is published.
- All work lands on `main` in the one checkout; cc and ic build in their own worktrees and land by fast-forward. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). main goes to `local` once committed (hv's decision 4); nothing goes upstream but a release.
- Validation is hv's one layer of proof: vc reads the diff, and the builder's one measured run, in a Desk cargo slot, is the run. devbin-vc holds the Desk. Test only what changed; the full suite runs once, in the cut's gates.
- A landing that moves lines under another thread's acceptance-test rows breaks that thread's `intent at lint` until they are re-cited; whoever sees it re-cites and tells the other seat.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
- How Claude Code reads its login is in ST0003's Context, with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
