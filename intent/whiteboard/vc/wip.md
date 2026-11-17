---
node: vc
name: Validation Claude
role: validation
session_id: 1f8ad5f9-8ef2-4931-bffe-b7473752f05d
heartbeat_at: 2026-11-17 05:44Z
status: active
focus: "The live repair (todo 13): steps 1 to 4 done on 2026-11-17. AC-06.2 is satisfied (its event carries the stamp): hv's ccusage use engineering00 (05:38Z) and ccusage which --use to engineering02 (05:41Z) each reached vc's open session with no restart, /status and its Usage tab naming the account each time. AC-06.1 waits for hv's look at Claude Usage; then the day closes"
claims: []
---

# Validation Claude (vc)

## DOING

- Validate each hand-over from cc, ic and dc against its work package's or issue's acceptance. Under hv's one layer of proof, the verdict is vc's read of the diff, with the builder's measured Desk run as the one run; a second run only when the landed tree differs from the tested one (check with git diff on src, tests and Cargo files). Each verdict goes to the builder's inbox and live, and vc closes a work package with intent wp done once intent at lint is clean for it. (edited)
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13), shipped in 0.1.9. WP01 to WP07 are closed, and intent at lint ST0002 is clean (27 of 27 at 0f66154). Left: AC-08.3, hv's second look at the panel. hv's first look (2026-11-13 12:29Z) found the weekly bars drawing no fill; ic's fix, with the look's two nits, landed on main as 9be58a6, green in its Desk slot. hv installs from a build at or after it (ccusage herdr install, --force only if it refuses) and looks again, which also checks that the pane refreshes by itself; it has not happened yet, and it is not asked for before the live repair of 2026-11-17 (doing 5). Open with hv after the look: preview.png (vc recommends hv's capture, cropped with the names blurred by ic; or as is; or drop the link). Open for ic: a way into the mod from inside Herdr, and its todo 5 (install says nothing while herdr-app check runs). ic's session has not run since 2026-11-13, though its board still reads active; ISSUE:0008's items 6 and 7 are interface text and wait on hv's word about starting ic. Then vc satisfies AC-08.3 and closes WP08; AC-00.1 follows. (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13), shipped in 0.1.9. WP01 to WP06 are closed: WP06, the rollout, closed on 2026-11-17 (fbcc2f5) on the live repair's end state, with AC-06.1 and AC-06.2 satisfied, and dc closed ISSUE:0003 (9304bb9). Left: ISSUE:0008's code on this thread's claim, landing for 0.1.10 (doing 5), then AC-00.1, the thread's sign-off. (edited)
- ISSUE:0008 (high), for 0.1.10 first thing on hv's word (decision 12). The live repair is done (2026-11-17, WP06). The code: cc's unit, items 1 to 5 with its two nits from WP04 (todo 8), is on cc/ISSUE-0008 at 47aa80b; cc replays it onto main's tip as cc/ISSUE-0008-land with a git diff proof, its slot 01M7JQCS4KTNMQVRNWBQQ3KTGM standing as the run, and vc lands it by fast-forward on its read (todo 12). Then cc's follow-up unit on the landed tip: items 6 and 7, text only, and item 8, the incoming login verified before a switch seats it (added to the issue at 708bd5e), in one Desk cargo slot, red then green; vc reads and lands it. ic reads the new user-facing text as nits after the landing. Then the cut (doing 6). (edited)
- The 0.1.10 cut, first thing on hv's word (decision 12): ISSUE:0008 whole, ISSUE:0009 (LICENSE back) and ic's Herdr bar fix (9be58a6), one cut. Sequence: cc's unit lands, cc's follow-up lands, vc says the tip is final, dc runs the kovacs gates and the scratch-HOME install at it in one heavy Desk slot (dc's todo 15), vc writes the cut's steps for hv's own terminal tab (bin/devbin release 0.1.10, never a ! command, seats frozen until the publish), then brew upgrade on rhadamanth and kovacs, dc's keg proof closes ISSUE:0009 (dc's hold 13), and hv's switch to brew. ST0002's sitting (AC-08.3 and AC-00.1) runs before the cut, so anything it finds rides the same cut. The Desk was HELD with its control orphaned at 05:58Z; devbin-vc is back and has hv's word from vc.
- The Claude Usage PR, upstream to richhickson/claudecodeusage, on hv's word (decision 12): Claude Usage 1.18 names each Keychain item once per run and keeps history per item, so after a switch its names are stale and its history mixes accounts. ic builds the fix in hv's fork (/Users/matts/Devel/forks/claudecodeusage) on a branch from upstream's v1.18, with tests and a Desk run, and drafts the PR text in simple ASD-STE100 bullets. vc reads the change, puts the text to hv, and nothing is pushed or opened until hv says so.

## TODO

- cc's two nits from vc's WP04 verdict (2026-11-13): sessions::parse takes the last NAME=value on the ps line, and verified_token_at refuses Unverified before any renewal. Given to cc on 2026-11-16 as part of its ISSUE:0008 unit, in the same worktree and Desk slot (cc's todo 4). vc reads them in that hand-over. (edited)
- First on the bounce after hv's compact: read cc's hand-over of ISSUE:0008's unit, which reached vc at 2026-11-16 17:51Z and is live in vc's inbox from cc (a reply to vc's 17:19Z). Branch cc/ISSUE-0008 in /Users/matts/Devel/scratch/ccusage-cc/wt-issue-0008, base 8a8e9b5: tests 6f077cd, fix 23b177f, AT rows 47aa80b (its code identical to 23b177f, by cc). The run is Desk slot 01M7JQCS4KTNMQVRNWBQQ3KTGM: red at 6f077cd on the seven new tests, green at 23b177f (build, test rust, check format and clippy), and cc reports intent at lint clean on ST0003 (32 of 32) and ST0002 (27 of 27). No override flag was built. vc reads the diff against ISSUE:0008's Acceptance and fix outline items 1 to 5 and the two nits, takes that slot as the run, checks the landed tree against 23b177f, and lands it. Also to place: cc's finding, not built, that a switch to a profile whose own store holds another account's login seats that login, because the incoming login is never verified; vc decides whether it joins this unit before the cut. Items 6 and 7 still wait on hv's word about ic.
- Tell geodica-estate, when it next runs, that Ccusage's live repair is done (2026-11-17): no Claude session carries CLAUDE_SECURESTORAGE_CONFIG_DIR, one verified login an account, a live switch reaches an open session, Claude Usage lists each account once. devbin-vc was told live on 2026-11-17 with hv's word.

## Holds

_(none)_

## Watch-outs

- Claude Usage 1.18 (mainline, /Applications since 2026-11-17) keeps each Keychain item's first email for its whole run, so after any ccusage switch its names are stale until it restarts, and the shared item's history mixes the accounts seated there. Until ic's PR lands upstream: read accounts from ccusage, not Claude Usage's names, and expect hv to restart it after a switch. Its login item: the fork's test build has the same bundle id, so check at the next login that the one running is /Applications/ClaudeUsage.app.

## Decisions

_(none)_

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
