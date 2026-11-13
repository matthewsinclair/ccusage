---
node: vc
name: Validation Claude
role: validation
session_id: b4050368-cdbf-407f-bcaa-8ceee438c695
heartbeat_at: 2026-11-13 12:15Z
status: active
focus: "0.1.9 is published (v0.1.9 at 2762bbf, 2026-11-13 12:04Z). Next: hv's Mac T10 run (use dev, opt and brew, each with doctor) and hv's Herdr look (AC-08.3); then ISSUE:0007's Linux leg on kovacs on hv's word; the rollout (ST0003/WP06) later, and nobody runs ccusage use or which --use on 0.1.9 before it."
claims: []
---

# Validation Claude (vc)

## DOING

- Validate each hand-over from cc, ic and dc against its work package's or issue's acceptance. Under hv's one layer of proof, the verdict is vc's read of the diff, with the builder's measured Desk run as the one run; a second run only when the landed tree differs from the tested one (check with git diff on src, tests and Cargo files). Each verdict goes to the builder's inbox and live, and vc closes a work package with intent wp done once intent at lint is clean for it. (edited)
- ST0002, the which and predict verbs and the Herdr mod, built by ic (hv, 2026-11-13), shipped in 0.1.9. WP01 to WP07 are closed, each PASS by vc, and intent at lint ST0002 is clean (27 of 27 rows at d07b2ed). Left: AC-08.3, hv's look at the mod in hv's own Herdr, taken now rather than at the rollout: ccusage herdr install reports herdr-app check passed, and the pane shows each account's weekly bar, reset, state and next marker and refreshes by itself. A card's dialog has a Use button that runs ccusage use, so it is left alone until the rollout. On hv's word that the look passed, vc satisfies AC-08.3 and closes WP08; AC-00.1, the thread's sign-off, follows. Open with hv, from ic: a screenshot of the pane as herdr/ccusage/preview.png (it shows hv's account names and usage), or RECIPE.md drops the link. (edited)
- ST0003, live account switching, built by cc (hv, 2026-11-13), shipped in 0.1.9. WP01 to WP05 are closed, and intent at lint ST0003 is clean. Left: WP06, hv's rollout, later on hv's decision 5 (its steps in todo 7, with AC-06.1 and AC-06.2), which closes ISSUE:0003 (todo 4); then AC-00.1, the thread's sign-off. (edited)

## TODO

- ISSUE:0003's live check. hv took ST0003's route on 2026-11-13: WP01's trial passed and hv started cc on it, so ISSUE:0003 closes at ST0003/WP06's rollout, in vc's closing sequence (todo 7), rather than by a launcher fix and step 4. Read from the live processes at 2026-11-13 06:14Z: all 25 claude processes carried a profile's directory, and nothing renewed ca646b28. When WP06 is done, dc closes ISSUE:0003 (dc's hold 6), its Acceptance read against the new layout, with its last line as hv's decision 3 has it: Claude Usage lists each account once. (edited)
- ISSUE:0007's wiring passed on vc's verdict of 2026-11-13 (9cc5f93, 78cabdc and 802de63, dc's runs in two Desk slots) and shipped in 0.1.9. On gyges, hv's brew upgrade verified the signature and ccusage --version read ccusage 0.1.9 (brew, 2762bbf961ea). dc holds the issue open (dc's hold 10) until T10 passes on hv's Mac and on Linux with brew alone, as its Acceptance asks. The Mac run is hv's, in the checkout: bin/devbin use dev, use opt and use brew in turn, each followed by ccusage --version and bin/devbin doctor, the link resting at brew; vc relays it to dc. The Linux leg is dc's on hv's OK, in a heavy Desk slot: on kovacs, brew install ccusage from the tap (a source build that may pull brew's rust), then in a clone at v0.1.9, bin/devbin use brew, the version line, doctor and a listing of the keg's libexec. vc has put to hv whether it runs now, after the compact, or moves to a new issue. Acceptance 4 closes on vc's PASS at 802de63, with ST0003 named for the change in shared mode. When dc closes the issue, vc tells devbin-vc (devbin 0326 closes once pplr#0001 and cldr#0001 have closed too). (edited)
- 0.1.9's rollout, later (hv's decision 5, 2026-11-13; ST0003/WP06, closing ISSUE:0003). The cut is out: v0.1.9 at 2762bbf, published 12:04:31Z with signed and notarized Mac archives, formula cf77911 in the tap. The rollout's steps: ccusage doctor; ccusage which, then the first ccusage use for an account no running session is pinned to (25 sessions on hello and 2 on engineering01 at 10:55Z; engineering02 and engineering04 had none); doctor again; fresh shells, which Molt's env.sh leaves without CLAUDE_SECURESTORAGE_CONFIG_DIR once ccusage dir prints nothing; sessions restarted, never switching to an account a session is still pinned to; the live switch (AC-06.1, AC-06.2); then dc closes ISSUE:0003 and vc closes WP06. Back-out: ccusage use --none. Until the rollout nobody runs ccusage use or ccusage which --use on 0.1.9, on any Mac, gyges included. The steps are in intent/wip.md and in vc's scratchpad file closing-sequence-0.1.9.md. (edited)
- cc's two nits from vc's WP04 verdict (2026-11-13), for after the cut (cc's todo 4), in one Desk slot on vc's word after vc's compact: (1) sessions::parse takes the first NAME=value among the arguments and the environment together; ps -E prints the environment after the arguments, so taking the last match prefers the environment over an argument that looks like one (no running session repeated either variable at vc's check). (2) usage::verified_token_at, with neither an email nor an account record to check a login against, renews before refusing Unverified; it could refuse before the renewal. (edited)
- The tap's CI runs brew style and brew audit --strict again (hv's ISSUE:0004 ruling 3; was hold 3). Its condition is met: 0.1.9's formula, cf77911 in the tap, was rendered by devbin 0.1.23, which writes the manifest list one element a line (devbin 0322). Next, after vc's compact: in a clone of the tap, take out the lines in its test.yml that turn style and audit off, push on hv's OK, and see the tap's CI green.

## Holds

_(none)_

## Watch-outs

_(none)_

## Decisions

_(none)_

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
