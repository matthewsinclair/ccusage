---
node: vc
name: Validation Claude
role: validation
session_id: b4050368-cdbf-407f-bcaa-8ceee438c695
heartbeat_at: 2026-11-13 07:54Z
status: active
focus: "Sequencing 0.1.9's one cut (hv, 2026-11-13): dc wires ISSUE:0007 and is in its measured run; ic builds ST0002 from WP01 and cc builds ST0003 from WP02, each on vc's brief in its inbox; vc validates each hand-over on its builder's one run. ST0003/WP01's trial passed."
claims: []
---

# Validation Claude (vc)

## DOING

- Validate each hand-over from cc, ic and dc against its work package's or issue's acceptance. Under hv's one layer of proof, the verdict is vc's read of the diff, with the builder's measured Desk run as the one run; a second run only when the landed tree differs from the tested one (check with git diff on src, tests and Cargo files). Each verdict goes to the builder's inbox and live, and vc closes a work package with intent wp done once intent at lint is clean for it. (edited)
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13). WP01 (typed usage windows) is done: PASS by vc, and vc closed it with intent at lint ST0002 clean after a5b7c65. WP02 (ca09eae..fdfe1e5 on branch ic/ST0002-02): PASS on vc's read and signed by cc; it lands after the bounce with three small commits first (the lenient scoped share with a test per malformed form and a share-unknown reason; soonest named only among accounts that give a reset; cc's wording for the undated reason), as ic's hold 4. The which --json contract, version 1, is final and given to geodica-estate (vc's decision 1). Next, after the bounce: ic lands WP02, then WP04 (history, started, its design on ic's board), WP03, WP05, WP06, WP08 and WP07, each handed to vc on its Desk run. (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13). WP01 is done: hv's trial passed, and its record is in the work package. The one use function, switch::to, is on main at 31cf2c8 with vc's PASS; the CLI's use, run NAME and the TUI's a key call it, and which --use will. Two notes for WP03: capture launchctl's output in set_launchd, and rename cli.rs's flag helper also named switch. Next, after the bounce: cc carries on with WP02 (the login move) on branch cc/ST0003 in its worktree, then WP03 with WP04, then WP05; vc writes WP06's rollout steps once WP03 names its commands. ISSUE:0003 closes at WP06. (edited)

## TODO

- ISSUE:0003's live check. hv took ST0003's route on 2026-11-13: WP01's trial passed and hv started cc on it, so ISSUE:0003 closes at ST0003/WP06's rollout, which keeps the shared config's own login as the live one, rather than by a launcher fix and step 4. Read from the live processes at 2026-11-13 06:14Z: all 25 claude processes carried a profile's directory, and nothing renewed ca646b28. When WP06 is done, dc closes ISSUE:0003 (dc's hold 6), its Acceptance read against the new layout. (edited)
- ISSUE:0007's wiring: PASS on vc's verdict of 2026-11-13. The range is 9cc5f93, 78cabdc and 802de63. vc read the diffs against the issue's Acceptance, and the run is dc's, in two Desk slots: check all and cli 31 of 31 at 78cabdc, standing for 802de63; then at 802de63, macos bats 45 of 45, the stamped dev line, install and use dev and opt under a scratch HOME, and doctor 23 of 23. Left for the cut, with hv: the kegs at 0.1.9's publish, use brew, and T10's proof, after which dc closes the issue and vc tells devbin-vc (devbin 0326 closes once pplr#0001 and cldr#0001 have closed too). dc adds one line to docs/releasing.md: macos finalize cannot re-verify a release cut before 78cabdc. (edited)
- 0.1.9 cut blocker: devbin 0348 (high). devbin 0.1.23's doctor exits 1 on a Mac that still holds the pre-contract 0.1.8 keg, and release.gates runs doctor. Filed at devbin fdcad579 with 0349 (Homebrew moves metafiles out of libexec, why LICENSE left the archives) and 0350 (low). Building them waits on hv's word for devbin 0.1.24. Three routes, put to hv in the live channel: (A) go for devbin 0.1.24, vendored before the cut; (B) a ruling that the cut accepts doctor's red rows; (C, dc's, recommended) before the cut, hv runs bin/devbin install and bin/devbin use opt, the first steps of T10 and of ST0003's rollout. doctor passes with the link on opt, as dc's slot 2 showed, and use brew follows brew upgrade to the 0.1.9 keg. Then: pass hv's word to devbin-vc and dc. (edited)

## Holds

- Check that the tap's CI runs brew style and brew audit --strict again (hv's ISSUE:0004 ruling 3). Released when: the tap's formula is published by devbin 0.1.20 or later, at 0.1.9's post-cut. The vendored 0.1.20 carries the fix (devbin 0322: lib/cmd/formula:436 writes the manifest list one element a line), but the live formula 130e87d was rendered by 0.1.19, and the tap's test.yml still turns style and audit off. Then: turn them back on in test.yml and see the tap's CI green. (edited)

## Watch-outs

_(none)_

## Decisions

- The which --json contract, version 1, is final (2026-11-13 07:27Z): the shape from ic, the meanings from cc (headroom in percent of the weekly allowance, scoped windows with their share) and the yes from vc, with the clarification from ic that an unknown account still reports its last-seen percent and reset. vc gave it to geodica-estate, its first reader, the same minute. ic fixes it in a test at ST0002/WP03 and documents it at WP07; changing it now raises its version.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
