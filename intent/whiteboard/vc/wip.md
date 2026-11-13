---
node: vc
name: Validation Claude
role: validation
session_id: b4050368-cdbf-407f-bcaa-8ceee438c695
heartbeat_at: 2026-11-13 07:10Z
status: active
focus: "Sequencing 0.1.9's one cut (hv, 2026-11-13): dc wires ISSUE:0007 and is in its measured run; ic builds ST0002 from WP01 and cc builds ST0003 from WP02, each on vc's brief in its inbox; vc validates each hand-over on its builder's one run. ST0003/WP01's trial passed."
claims: []
---

# Validation Claude (vc)

## DOING

- Validate each unit dc hands over against its issue's Acceptance list. Under hv's one layer of proof, the verdict is vc's read of the diff, with the author's measured run as the one run; a scratch-clone run only when the committed tree differs from the one the run tested. Now: read dc's revert of 29943b5 (hv's correction: use is bin/devbin's). Next: ISSUE:0007's wiring (hold 4) and ISSUE:0003's live check (todo 4), both for 0.1.9's one cut at hv's word of 2026-11-06. (edited)
- ST0002, the which and predict verbs and the Herdr mod (hv, 2026-11-13): eight sized WPs and 17 ACs keyed AC-<wp>.<n>, reviewed by hv. hv's word: the accounts are on one plan, and engineering04 returns to the free plan; one cut, everything in; a Herdr mod is WP08; hv starts ic and cc once ST0003/WP01's facts are in. ic builds this thread and cc signs the rule, predict's arithmetic and what the JSON means. No seat writes code before WP01's facts are in. Then: brief ic, WP01 first, and book runs on the Desk (devbin-vc) as normal slot asks. (edited)
- ST0003, live account switching (hv, 2026-11-13): WP01 passed and is done. hv's trial on a scratch config showed a running session holding a newly stored login 73 s later, with no restart and no prompt; its first request after the change got a 404 thread_not_found, which Claude Code answered by replaying the conversation at once. The record is in WP01, AC-01.1 is satisfied, and the trial is cleaned up: vc found its Keychain item gone and 13 items as before at 07:08Z. Now: wait for hv to start cc and ic, whose briefs are in their inboxes. Then: cc takes WP02, and its first commit makes use one function; vc validates each work package on its builder's one measured run. (edited)

## TODO

- ISSUE:0003's live check, to close it in 0.1.9's cut (hv, 2026-11-06). Read from the live processes and Keychain metadata at 2026-11-13 06:14Z: all 25 claude processes carry a profile's directory (23 hello's, the active profile; 2 engineering01's, one launched before the 05:22Z switch to hello and one from a shell older than it); none runs on the main config's own login; and ca646b28 was last written 2026-11-11 12:38Z. So nothing renews ca646b28, and what is left of step 3 is the launcher gap for a shell older than the last ccusage use. Waiting on hv's choice. Either ST0003, whose rollout replaces steps 3 and 4 and keeps the shared config's login as the live one. Or the launcher fix (fresh shells, or launching through ccusage run), then step 4 (hv retires ca646b28) and three checks: (a) ca646b28 stays unwritten and every claude process carries the active profile's directory; (b) with hv's OK, a scratch-config session with the exported directory reports the active profile's email from claude auth status and no new Keychain item appears; (c) hv sees Claude Usage list exactly five accounts with nothing prompting, and vc counts the items. dc closes ISSUE:0003 either way (dc's hold 6). (edited)
- ISSUE:0007's wiring: dc committed 9cc5f93 (--version and the install: block) and 78cabdc (formula, archive and signer) and is in its Desk run. vc read both against ISSUE:0007's Acceptance at 07:14Z, ahead of the hand-over, and found no blocker: no new verb; the version reader in main.rs; install: with prefix ~/Devel/opt/ccusage, devbin's own form; the formula on @@INSTALL@@ on both platforms with libexec/bin/ccusage; CI staging bin/ccusage and LICENSE; the signer unpacking exactly those two, with bin/ the one directory allowed; and the CHANGELOG's 0.1.9 heading in the open form devbin's release accepts. Now: wait for dc's hand-over and its run. Then: the verdict. The full suite runs once, in the cut's gates, with ST0002 and ST0003. T10's proof is with hv: dev and opt before the cut (hv publishes to the opt prefix and switches), and brew after the cut's post-cut. When 0007 closes, tell devbin-vc. (edited)
- 0.1.9 cut blocker, found by dc's measured run (2026-11-13): devbin 0.1.23's doctor exits 1 on a Mac that still holds the pre-contract 0.1.8 keg (its switch row and its tree row), against devbin's own T3, and release.gates runs doctor, so the cut's gate is red on hv's Mac. devbin-vc: devbin-cc files dc's three findings as devbin issues, and fixing them waits for hv's word for devbin 0.1.24. Put to hv in the live channel: go for devbin 0.1.24, vendored before the cut; or hv's ruling on the doctor gate. Then: pass hv's word to devbin-vc and dc. With 0.1.24, dc vendors it, and LICENSE may return to the archives if finding 2 is fixed too.

## Holds

- Check that the tap's CI runs brew style and brew audit --strict again (hv's ISSUE:0004 ruling 3). Released when: the tap's formula is published by devbin 0.1.20 or later, at 0.1.9's post-cut. The vendored 0.1.20 carries the fix (devbin 0322: lib/cmd/formula:436 writes the manifest list one element a line), but the live formula 130e87d was rendered by 0.1.19, and the tap's test.yml still turns style and audit off. Then: turn them back on in test.yml and see the tap's CI green. (edited)

## Watch-outs

_(none)_

## Decisions

_(none)_

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
