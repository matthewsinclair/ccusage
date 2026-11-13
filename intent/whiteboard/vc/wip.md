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
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13). WP01 to WP04 are done, each PASS by vc and closed by vc with intent at lint ST0002 clean. Built and stacked on main's fac240d: eade2e4 (WP03's follow-ups: below the cap, and what an unusable account reports), 95b7b3b (WP05, predict), ef41e1b (WP06, the TUI's next marker), 7bd2655 and e2760d4 (WP08: the TUI asks again once a lifetime, the mod, and ccusage herdr install and remove as hv ruled in decision 4), b791e3a (WP07, README and CHANGELOG), and 4589212 (AT-08.3's id). One Desk run at 4589212 is queued behind hv's devvm build; vc reads each WP's hand-over and they land together. predict groups accounts by plan tier once cc's 02a5967 lands (ic's todo 3), before the cut. AC-08.3, hv's own check in his Herdr, is in the closing sequence (todo 7). (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13). WP01 to WP03 are done, each closed by vc with intent at lint ST0003 clean; WP03 landed as eea2dea. On cc/ST0003 and banked: b377106 (vc's notes 1 to 3 on WP03: a 400 is refused only with invalid_grant and a 401 never; parked_accounts prefers each profile's own account record; use --none points launchd at the active profile, with ic's text) and 02a5967 (each report's subscriptionType and rateLimitTier, for predict); their one run is queued behind hv's devvm build and ic's run, then the hand-over eea2dea..02a5967. WP04 (doctor, with what an account type gives when it has no usage access or no windows) is under way on cc/ST0003-WP04; then WP05 (docs and the decision record). WP06 is hv's rollout, in vc's closing sequence (todo 7). ISSUE:0003 closes at WP06. (edited)

## TODO

- ISSUE:0003's live check. hv took ST0003's route on 2026-11-13: WP01's trial passed and hv started cc on it, so ISSUE:0003 closes at ST0003/WP06's rollout, in vc's closing sequence (todo 7), rather than by a launcher fix and step 4. Read from the live processes at 2026-11-13 06:14Z: all 25 claude processes carried a profile's directory, and nothing renewed ca646b28. When WP06 is done, dc closes ISSUE:0003 (dc's hold 6), its Acceptance read against the new layout, with its last line as hv's decision 3 has it: Claude Usage lists each account once. (edited)
- ISSUE:0007's wiring: PASS on vc's verdict of 2026-11-13. The range is 9cc5f93, 78cabdc and 802de63. vc read the diffs against the issue's Acceptance, and the run is dc's, in two Desk slots: check all and cli 31 of 31 at 78cabdc, standing for 802de63; then at 802de63, macos bats 45 of 45, the stamped dev line, install and use dev and opt under a scratch HOME, and doctor 23 of 23. Left for the cut, with hv: the kegs at 0.1.9's publish, use brew, and T10's proof, after which dc closes the issue and vc tells devbin-vc (devbin 0326 closes once pplr#0001 and cldr#0001 have closed too). dc adds one line to docs/releasing.md: macos finalize cannot re-verify a release cut before 78cabdc. (edited)
- The closing sequence for 0.1.9, which vc writes for hv once ST0002 and ST0003 have landed: quit every Claude session; bin/devbin install and bin/devbin use opt (route C, hv's decision 3); the first ccusage use NAME, ST0003/WP06's adoption, read against its message and doctor; start shells afresh, so that no session reads a profile's own store; ccusage herdr install and hv's check in his Herdr (AC-08.3); bin/devbin release 0.1.9; after the publish, brew upgrade ccusage, bin/devbin use brew and T10's version check (ISSUE:0007). Then dc closes ISSUE:0003 and ISSUE:0007, and vc's hold 3 (the tap's CI) is released. The opt step waits for this sequence (hv's decision 4, on dc's advice): until 0.1.9's keg exists use brew refuses the 0.1.8 keg, so every ccusage on hv's machine would be main's unreleased build. (edited)

## Holds

- Check that the tap's CI runs brew style and brew audit --strict again (hv's ISSUE:0004 ruling 3). Released when: the tap's formula is published by devbin 0.1.20 or later, at 0.1.9's post-cut. The vendored 0.1.20 carries the fix (devbin 0322: lib/cmd/formula:436 writes the manifest list one element a line), but the live formula 130e87d was rendered by 0.1.19, and the tap's test.yml still turns style and audit off. Then: turn them back on in test.yml and see the tap's CI green. (edited)

## Watch-outs

_(none)_

## Decisions

- The which --json contract, version 1, is final (2026-11-13 07:27Z): the shape from ic, the meanings from cc (headroom in percent of the weekly allowance, scoped windows with their share) and the yes from vc, with the clarification from ic that an unknown account still reports its last-seen percent and reset. vc gave it to geodica-estate, its first reader, the same minute. With cc's tightening at ST0002/WP02, soonest is null when no usable account's weekly window gives a reset; version 1 stands, because a null soonest was already possible and nothing had emitted the JSON, and vc told geodica-estate. ic fixes the contract in a test at ST0002/WP03 and documents it at WP07; changing it now raises its version. (edited)

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
