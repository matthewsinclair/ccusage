---
verblock: "2026-11-13T12:18:12Z:v0.30: vc - localfold for hv's compact; 0.1.9 cut, rollout later"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0003: live account switching (hv, 2026-11-13), built by cc**

In 0.1.9. WP01 to WP05 are closed. Left: WP06, hv's rollout (below), later on hv's decision 5, with AC-06.1 and AC-06.2; it closes ISSUE:0003. Then AC-00.1, the thread's sign-off.

**ST0002: the which and predict verbs and the Herdr mod (hv, 2026-11-13), built by ic**

In 0.1.9. WP01 to WP07 are closed. Left: AC-08.3, hv's look at the mod in hv's own Herdr, taken now rather than at the rollout: `ccusage herdr install`, then `herdr-app do pane.split '{"direction":"right","mod":"ccusage"}'`; each account's weekly bar, reset, state and `next` marker, refreshing by itself, with `herdr-app check` passed. A card's dialog has a `Use NAME` button that runs `ccusage use`, so it is left alone until the rollout. Then vc closes WP08, and AC-00.1 is the thread's sign-off. Open with hv, from ic: a screenshot of the pane as `herdr/ccusage/preview.png` (it shows hv's account names and usage), or `RECIPE.md` drops the link.

**Ccusage 0.1.9: cut now, roll out later (hv's decision 5, 2026-11-13)**

- Nobody runs `ccusage use` or `ccusage which --use` on 0.1.9 before the rollout, on any Mac: either one moves that account's login into the shared config, and every session pinned to it loses its login (25 on hello and 2 on engineering01 at 10:55Z).
- ISSUE:0007 (medium) stays open on dc's hold 10 until T10 passes on hv's Mac and on Linux with brew alone. Mac, hv in the checkout: `bin/devbin use dev`, `use opt` and `use brew` in turn, each followed by `ccusage --version` and `bin/devbin doctor`, the link resting at brew. Linux, dc on hv's OK in a heavy Desk slot: on kovacs, `brew install ccusage` from the tap, then in a clone at v0.1.9 `bin/devbin use brew`, the version line, doctor and a listing of the keg's libexec. Acceptance 4 closes on vc's PASS at 802de63, with ST0003 named for the change.
- The rollout, later (ST0003/WP06):
  1. `ccusage doctor`: exit 0; the shared config's own login warned as no profile's, each profile's login OK in its own store, each running session warned as pinned.
  2. `ccusage which`, then `ccusage use <account>` for the higher ranked account no running session is pinned to (engineering02 or engineering04 at 10:55Z), or, once every session has quit, the account `which` names.
  3. `ccusage doctor`: no FAIL; the account's login in the shared config and verified as its.
  4. In a new tab, `echo ${CLAUDE_SECURESTORAGE_CONFIG_DIR-unset}; launchctl getenv CLAUDE_SECURESTORAGE_CONFIG_DIR` prints `unset`, then an empty line.
  5. Claude sessions restarted from fresh tabs, never switching to an account a session is still pinned to. vc checks one login per account and the pinned count falling to 0; hv checks that Claude Usage lists each account once.
  6. The live switch (AC-06.2): with a restarted session open, `ccusage use <another unpinned account>`; the session answers its next prompt, and `/status` names the new account. Once no session is pinned, `ccusage which --use` too.
  7. dc closes ISSUE:0003; vc closes ST0003/WP06.
- Back-out: `ccusage use --none` puts the active login back in its profile and points launchd at it, as 0.1.8 does.

## Active Steel Threads

- ST0002: Add 'which' verb to indicate the best account to use to maximise token usage across accounts. WIP; WP08 waits on AC-08.3, hv's look.
- ST0003: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow. WIP; WP06, the rollout, later.

## Upcoming Work

- cc's two nits from WP04's verdict, in one Desk slot after vc's compact: `sessions::parse` takes the first NAME=value among the arguments and the environment (take the last, so the environment wins); `usage::verified_token_at` renews before refusing Unverified when nothing names the account (refuse first).
- The tap's CI runs `brew style` and `brew audit --strict` again (hv's ISSUE:0004 ruling 3): 0.1.9's formula, rendered by devbin 0.1.23, meets the condition. Turn them back on in the tap's test.yml, push on hv's OK, and see it green (vc todo 9).
- ISSUE:0003 (medium) closes at ST0003/WP06; its last acceptance line reads each account once in Claude Usage (hv's decision 3).
- devbin#0349 (LICENSE back in the archives) waits for a later devbin (dc's hold 9).
- vc tells geodica-estate when the rollout is done.

## Notes

- hv's standing rulings are decisions 1 to 5 on hv's board: ccusage works for any Claude account type and shows whatever it can for that type; a login a switch drops lapses and is not revoked; engineering04 stays a profile until its allowance is exhausted; Herdr's verb is `ccusage herdr install` and `remove`; 0.1.9 is cut before the rollout, which follows later, fixing forward.
- Before a cut, dc runs the project's gates on kovacs (Linux ARM64): Linux clippy sees code that only a macOS arm uses, which rhadamanth's runs miss, and the release config names no legs.
- A cut runs in hv's own terminal tab (`bin/devbin release X.Y.Z`), never as a `!` command. The seats write nothing to the checkout, whiteboard verbs included, until vc says it is published.
- All work lands on `main` in the one checkout; cc and ic build in their own worktrees and land by fast-forward. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). main goes to `local` once committed (hv's decision 4); nothing goes upstream but a release.
- Validation is hv's one layer of proof: vc reads the diff, and the builder's one measured run, in a Desk cargo slot, is the run. devbin-vc holds the Desk. Test only what changed; the full suite runs once, in the cut's gates.
- A landing that moves lines under another thread's acceptance-test rows breaks that thread's `intent at lint` until they are re-cited; whoever sees it re-cites and tells the other seat.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
- How Claude Code reads its login is in ST0003's Context, with the command that reads it from the installed binary again. How a Herdr mod is built is in `~/.config/herdr-app/AGENTS.md`, which the Herdr app rewrites at each start.
