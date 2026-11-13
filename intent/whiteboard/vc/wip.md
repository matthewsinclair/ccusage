---
node: vc
name: Validation Claude
role: validation
session_id: b4050368-cdbf-407f-bcaa-8ceee438c695
heartbeat_at: 2026-11-13 09:29Z
status: active
focus: "Sequencing 0.1.9's one cut: three cargo runs from here (ic's ST0002 stack, cc's follow-up rebased onto it, cc's WP04 with ic's tier grouping), then vc's closing sequence with hv; vc reads each hand-over on its builder's run."
claims: []
---

# Validation Claude (vc)

## DOING

- Validate each hand-over from cc, ic and dc against its work package's or issue's acceptance. Under hv's one layer of proof, the verdict is vc's read of the diff, with the builder's measured Desk run as the one run; a second run only when the landed tree differs from the tested one (check with git diff on src, tests and Cargo files). Each verdict goes to the builder's inbox and live, and vc closes a work package with intent wp done once intent at lint is clean for it. (edited)
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13). WP01 to WP07 are done, each PASS by vc and closed by vc with intent at lint ST0002 clean (27 of 27 rows at bf6160a); AC-07.1 satisfied on vc's read of README and CHANGELOG as landed at b08623d. The stack landed as d57a014..258e0a7 (run at 70adaf7), and its last two commits, 01d0ef9 (predict counts one plan) and 22f431d (two herdr fixes), with cc's ST0003/WP04 in 1648348..b08623d (run at 7cac510). Left: WP08 closes after AC-08.3, hv's own check of the panel in Herdr, in the closing sequence (todo 7); then AC-00.1, the thread's sign-off. (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13). WP01 to WP04 are done, each closed by vc with intent at lint ST0003 clean (31 of 31 rows after WP04); WP04 landed as 1648348..b08623d, its code identical to its run at 7cac510, the last cargo run before the cut. WP05 (docs and the decision record), main..9f3f2ab: PASS on vc's read (2026-11-13), no run; cc lands it as one commit. vc satisfies AC-05.1 on that read once it has landed and dc has pointed ISSUE:0003's record at ST0003 (vc's brief to dc), then closes WP05. WP06 is hv's rollout, in vc's closing sequence (todo 7), with README's three steps from 0.1.8, the launcher's export line out first. ISSUE:0003 closes at WP06. (edited)

## TODO

- ISSUE:0003's live check. hv took ST0003's route on 2026-11-13: WP01's trial passed and hv started cc on it, so ISSUE:0003 closes at ST0003/WP06's rollout, in vc's closing sequence (todo 7), rather than by a launcher fix and step 4. Read from the live processes at 2026-11-13 06:14Z: all 25 claude processes carried a profile's directory, and nothing renewed ca646b28. When WP06 is done, dc closes ISSUE:0003 (dc's hold 6), its Acceptance read against the new layout, with its last line as hv's decision 3 has it: Claude Usage lists each account once. (edited)
- ISSUE:0007's wiring: PASS on vc's verdict of 2026-11-13. The range is 9cc5f93, 78cabdc and 802de63. vc read the diffs against the issue's Acceptance, and the run is dc's, in two Desk slots: check all and cli 31 of 31 at 78cabdc, standing for 802de63; then at 802de63, macos bats 45 of 45, the stamped dev line, install and use dev and opt under a scratch HOME, and doctor 23 of 23. Left for the cut, with hv: the kegs at 0.1.9's publish, use brew, and T10's proof, after which dc closes the issue and vc tells devbin-vc (devbin 0326 closes once pplr#0001 and cldr#0001 have closed too). dc's line in docs/releasing.md, that macos finalize cannot re-verify a release cut before the install-root layout, is in at 782584d. (edited)
- The closing sequence for 0.1.9, which vc writes for hv once ST0002 and ST0003 have landed: quit every Claude session; bin/devbin install and bin/devbin use opt (route C, hv's decision 3); the first ccusage use NAME, ST0003/WP06's adoption, read against its message and doctor; start shells afresh, so that no session reads a profile's own store; ccusage herdr install and hv's check in Herdr (AC-08.3); bin/devbin release 0.1.9; after the publish, brew upgrade ccusage, bin/devbin use brew and T10's version check (ISSUE:0007). Then dc closes ISSUE:0003 and ISSUE:0007, and vc's hold 3 (the tap's CI) is released. The opt step waits for this sequence (hv's decision 4, on dc's advice): until 0.1.9's keg exists use brew refuses the 0.1.8 keg, so every ccusage on hv's machine would be main's unreleased build. (edited)
- After 0.1.9's cut, cc's two nits from vc's WP04 verdict (2026-11-13), neither worth a run before it: (1) sessions::parse takes the first NAME=value among the arguments and the environment together; ps -E prints the environment after the arguments, so taking the last match prefers the environment over an argument that looks like one (no running session repeated either variable at vc's check). (2) usage::verified_token_at, with neither an email nor an account record to check a login against, renews before refusing Unverified; it could refuse before the renewal.

## Holds

- Check that the tap's CI runs brew style and brew audit --strict again (hv's ISSUE:0004 ruling 3). Released when: the tap's formula is published by devbin 0.1.20 or later, at 0.1.9's post-cut. The vendored 0.1.20 carries the fix (devbin 0322: lib/cmd/formula:436 writes the manifest list one element a line), but the live formula 130e87d was rendered by 0.1.19, and the tap's test.yml still turns style and audit off. Then: turn them back on in test.yml and see the tap's CI green. (edited)

## Watch-outs

_(none)_

## Decisions

- The which --json contract, version 1, is final (2026-11-13 07:27Z): the shape from ic, the meanings from cc (headroom in percent of the weekly allowance, scoped windows with their share) and the yes from vc, with the clarification from ic that an unknown account still reports its last-seen percent and reset. vc gave it to geodica-estate, its first reader, the same minute. With cc's tightening at ST0002/WP02, soonest is null when no usable account's weekly window gives a reset; version 1 stands, because a null soonest was already possible and nothing had emitted the JSON, and vc told geodica-estate. ic fixes the contract in a test at ST0002/WP03 and documents it at WP07; changing it now raises its version. (edited)

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
