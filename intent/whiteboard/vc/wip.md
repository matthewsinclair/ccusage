---
node: vc
name: Validation Claude
role: validation
session_id: 5de51c0d-bc8f-4604-a46b-e4b7f7932b2e
heartbeat_at: 2026-11-16 17:19Z
status: active
focus: "ISSUE:0008 on hv's word (diagnose, then fix): the live repair is hv's, on vc's steps, and is WP06's rollout; cc builds the code with its two nits, dc returns LICENSE and takes the tap's CI; one cut follows"
claims: []
---

# Validation Claude (vc)

## DOING

- Validate each hand-over from cc, ic and dc against its work package's or issue's acceptance. Under hv's one layer of proof, the verdict is vc's read of the diff, with the builder's measured Desk run as the one run; a second run only when the landed tree differs from the tested one (check with git diff on src, tests and Cargo files). Each verdict goes to the builder's inbox and live, and vc closes a work package with intent wp done once intent at lint is clean for it. (edited)
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13), shipped in 0.1.9. WP01 to WP07 are closed, and intent at lint ST0002 is clean (27 of 27 rows). Left: AC-08.3, which hv's first look (2026-11-13 12:29Z, the mod as a split beside a terminal) did not meet. herdr-app check passed (about two minutes under a load average near 230, Herdr's check job waiting on the window server), and each account's reset, state and the next marker show, but the weekly bars draw no fill (22% and 39% look like 0%). The likely cause, sent to ic: bar() at herdr/ccusage/main.js:40 sits in an h_flex, which centres its children, so the fill div has no height; give it h_full(). main.js is built in (include_str in src/herdr.rs), so ic's fix needs a Desk build after vc's compact, then hv's ccusage herdr install --force from the dev build and a second look, which also checks that the pane refreshes by itself. ic's nits from the look: hello's reason prints twice, and the title shows twice. Open with hv: preview.png (vc recommends hv's capture, cropped with the names blurred by ic; or as is; or drop the link). Open for ic after the compact: a way into the mod from inside Herdr, since Herdr 0.1.15 has no menu for a mod pane (today: herdr-app do tab.new with mod ccusage, or a key in Molt-matts' keys.json). Then vc satisfies AC-08.3 and closes WP08; AC-00.1 follows. (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13), shipped in 0.1.9. WP01 to WP05 are closed, and intent at lint ST0003 is clean. Left: WP06, hv's rollout, later on hv's decision 5 (its steps in todo 7, with AC-06.1 and AC-06.2), which closes ISSUE:0003 (todo 4); then AC-00.1, the thread's sign-off. (edited)
- ISSUE:0008 (high), filed by vc on 2026-11-16 at 2e79f55 on hv's word (decision 7). hello was seated on 2026-11-15 at 19:19Z by hv's own hand, and the fleet restarted on 2026-11-16 from zsh shells alive since 2026-11-12 that still export CLAUDE_SECURESTORAGE_CONFIG_DIR: 24 of 29 sessions pinned to hello's own store and 1 to engineering01's at 17:02Z. Two parts. (1) The live repair, which is ST0003/WP06's rollout: first thing 2026-11-17 on hv's word (decision 8), hv's hands, vc checking from metadata; its six steps are in intent/wip.md. devbin-vc has it and acknowledged: the Desk grants nothing into the window and nothing after it until vc says the estate is clear, so vc tells devbin-vc and geodica-estate when it is done. The shells are Herdr Core's (up since 2026-11-12) and outlived the Herdr app's restart on 2026-11-15, so each pane's shell needs the unset, which hc send can type once no pane holds a Claude prompt; vc's first word to hv, that reopening Herdr clears them, was wrong and is corrected. On the day vc starts alone, checks its own process carries no pin, sees the live switch reach it (ccusage use engineering00, then ccusage which --use), then checks every session as the fleet starts. Open with hv: whether ic starts or cc takes the interface text (the issue's items 6 and 7), and whether the launch-time fix goes to Molt's or Intent's desk. (2) The code, for the next cut: cc has claimed ISSUE:0008 and builds items 1 to 5 with its two nits, one worktree and one Desk slot; vc reads the diff and takes its run as the run. Then dc closes ISSUE:0003 and vc closes WP06. (edited)

## TODO

- ISSUE:0003's live check. hv took ST0003's route on 2026-11-13: WP01's trial passed and hv started cc on it, so ISSUE:0003 closes at ST0003/WP06's rollout, in vc's closing sequence (todo 7), rather than by a launcher fix and step 4. Read from the live processes at 2026-11-13 06:14Z: all 25 claude processes carried a profile's directory, and nothing renewed ca646b28. When WP06 is done, dc closes ISSUE:0003 (dc's hold 6), its Acceptance read against the new layout, with its last line as hv's decision 3 has it: Claude Usage lists each account once. (edited)
- 0.1.9's rollout (ST0003/WP06, closing ISSUE:0003). It began on 2026-11-15 at 19:19Z by hv's own hand, when hello was seated, and stands half done: 25 of 29 sessions are pinned and do not follow a switch, hello has two logins, and engineering01's own store holds a login as hello. Its steps from here are ISSUE:0008's live repair (doing item on this board). AC-06.1 and AC-06.2 are read against its end state: doctor exits 0 with no pinned session, and a switch moves the running sessions. Back-out stays ccusage use --none. vc tells geodica-estate when it is done. (edited)
- cc's two nits from vc's WP04 verdict (2026-11-13): sessions::parse takes the last NAME=value on the ps line, and verified_token_at refuses Unverified before any renewal. Given to cc on 2026-11-16 as part of its ISSUE:0008 unit, in the same worktree and Desk slot (cc's todo 4). vc reads them in that hand-over. (edited)
- The tap's CI runs brew style and brew audit --strict again (hv's ISSUE:0004 ruling 3). dc's change is ready: 434d105 in dc's scratch clone ~/Devel/scratch/ccusage-dc/homebrew-ccusage, on cf77911, the tap's main; it changes test.yml alone, back to 65cac20's exactly, and both steps exit 0 on the tap's formula in dc's run (vc's read, 2026-11-16: right). Waiting on hv's OK to push, asked 2026-11-16 in vc's window. On it: vc passes the word to dc, dc pushes over the tap's ssh remote and reports the tap's CI on macos-15 and ubuntu-22.04, and once it is green vc lands b3ca1c1 from dc/tap-ci-docs (docs/releasing.md's one line). (edited)

## Holds

_(none)_

## Watch-outs

_(none)_

## Decisions

_(none)_

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
