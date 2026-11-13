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
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13). WP01 to WP04 are done, each PASS by vc and closed by vc with intent at lint ST0002 clean: WP03 is the which verb (5139af2, its JSON version 1 signed by cc at 90ca48e) and WP04 the usage history (68376eb..1831d5c). One text change rides with WP05's run: which's line gives used and below-the-cap apart, and never says left for the headroom to the cap. hv's ruling 3: which, the TUI and the Herdr panel show whatever windows any account reports, and which chooses only among accounts that report a weekly window. Next: WP05 (predict), WP06, WP08 and WP07, each handed to vc on its Desk run. (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13). WP01 and WP02 are done: hv's trial passed, and the login move landed as 9afdf86 and 20b7dd2 on vc's PASS, closed by vc with intent at lint ST0003 clean. switch::to is on main at 31cf2c8. cc is building WP03 (the verbs on one live login, with ic's spelling use --none) with WP04 (doctor) beside it, then WP05, carrying vc's six notes; adoption replaces a shared login only when its account holds another login in a profile or the token endpoint refuses its refresh outright, never by default. vc writes WP06's rollout steps once WP03 names its commands, in this order: no session reads a profile's own store (CLAUDE_SECURESTORAGE_CONFIG_DIR) when the first move runs, since a move empties the incoming profile's store. ISSUE:0003 closes at WP06. (edited)

## TODO

- ISSUE:0003's live check. hv took ST0003's route on 2026-11-13: WP01's trial passed and hv started cc on it, so ISSUE:0003 closes at ST0003/WP06's rollout, which keeps the shared config's own login as the live one, rather than by a launcher fix and step 4. Read from the live processes at 2026-11-13 06:14Z: all 25 claude processes carried a profile's directory, and nothing renewed ca646b28. When WP06 is done, dc closes ISSUE:0003 (dc's hold 6), its Acceptance read against the new layout. (edited)
- ISSUE:0007's wiring: PASS on vc's verdict of 2026-11-13. The range is 9cc5f93, 78cabdc and 802de63. vc read the diffs against the issue's Acceptance, and the run is dc's, in two Desk slots: check all and cli 31 of 31 at 78cabdc, standing for 802de63; then at 802de63, macos bats 45 of 45, the stamped dev line, install and use dev and opt under a scratch HOME, and doctor 23 of 23. Left for the cut, with hv: the kegs at 0.1.9's publish, use brew, and T10's proof, after which dc closes the issue and vc tells devbin-vc (devbin 0326 closes once pplr#0001 and cldr#0001 have closed too). dc adds one line to docs/releasing.md: macos finalize cannot re-verify a release cut before 78cabdc. (edited)
- 0.1.9's cut takes route C past devbin 0348, on hv's ruling (decision 3 on hv's board). Just before the cut hv runs bin/devbin install and bin/devbin use opt, so doctor passes with the link on opt, then bin/devbin release 0.1.9; after the publish, brew upgrade ccusage, bin/devbin use brew and T10's version check. vc writes these steps for hv together with ST0003/WP06's rollout, whose first steps they share. devbin-vc and dc are told; devbin 0349 (LICENSE in the archives) waits for a later devbin, as dc's hold 9. (edited)

## Holds

- Check that the tap's CI runs brew style and brew audit --strict again (hv's ISSUE:0004 ruling 3). Released when: the tap's formula is published by devbin 0.1.20 or later, at 0.1.9's post-cut. The vendored 0.1.20 carries the fix (devbin 0322: lib/cmd/formula:436 writes the manifest list one element a line), but the live formula 130e87d was rendered by 0.1.19, and the tap's test.yml still turns style and audit off. Then: turn them back on in test.yml and see the tap's CI green. (edited)

## Watch-outs

_(none)_

## Decisions

- The which --json contract, version 1, is final (2026-11-13 07:27Z): the shape from ic, the meanings from cc (headroom in percent of the weekly allowance, scoped windows with their share) and the yes from vc, with the clarification from ic that an unknown account still reports its last-seen percent and reset. vc gave it to geodica-estate, its first reader, the same minute. With cc's tightening at ST0002/WP02, soonest is null when no usable account's weekly window gives a reset; version 1 stands, because a null soonest was already possible and nothing had emitted the JSON, and vc told geodica-estate. ic fixes the contract in a test at ST0002/WP03 and documents it at WP07; changing it now raises its version. (edited)

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
