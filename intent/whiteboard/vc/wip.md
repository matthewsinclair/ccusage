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
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13). WP01 to WP04 and WP06 are done, each PASS by vc and closed by vc with intent at lint ST0002 clean. The rest of the stack landed on main as d57a014..258e0a7 (9cd35e1 WP03's follow-ups; 447b20f and 258e0a7 WP05, predict; 217cf08 WP06, the next marker; 102676f, 710f2e1 and b8a12dd WP08, the TUI's refresh and the Herdr mod; 30e9b63 WP07, README and CHANGELOG), run once at 70adaf7 in slot 01M7A1JYDSQNNHNNN55ACXCX40; PASS on vc's read of each. Its last two commits rode cc's ST0003/WP04 run at 7cac510 and PASS on vc's read: 8234a5e, predict counting the plan most usable accounts record, its rate on that plan's samples alone, with README's predict paragraph losing its one-plan assumption; and e953ca3, ic's two herdr fixes (a hidden directory counts as an edit; herdr-app's cause when it cannot run). Left: WP05 and WP07 close once cc has landed them, ic has minted AT-05.4 and re-cited ST0002's rows, and ST0002 lints clean, AC-07.1 satisfied on vc's read of README and CHANGELOG as landed. WP08 closes after AC-08.3, hv's own check in Herdr, in the closing sequence (todo 7). (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13). WP01 to WP03 are done, each closed by vc with intent at lint ST0003 clean; WP03 landed as eea2dea, and its follow-up as 26f373f..ed77832 (PASS, code identical to its tested 46295e6). WP04 (doctor) with todo 3a, and ic's 8234a5e and e953ca3 riding it, 348a1a1..7cac510: PASS on vc's read of 2026-11-13, with cc's run in slot 01M7A68EK9GKKRVBEVE6G2B8MA at 7cac510 (22 of 22 mutants red; fmt, clippy, cargo test 93 unit and 48 cli, and the keychain tests exit 0), the last cargo run before the cut. cc lands it as four commits whose code equals 7cac510 and mints AT-04.1 to 04.8, 04.10 and 04.11; vc closes WP04 once ST0003 lints clean. Then WP05 (docs and the decision record, drafted, README signed by ic) rebases onto the landing for vc's read, with no cargo run. WP06 is hv's rollout, in vc's closing sequence (todo 7). ISSUE:0003 closes at WP06. (edited)

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
