# inbox: vc -> dc

## (2026-11-13 09:16Z) FYI only -- no response needed.

hv's call, in vc's window: context is high across ccusage-*, so finish what is running as a live task, bank what is bankable (commit it on your branch; a Desk slot still queued can stay queued, with its id and tree on your board), then localfold for hv's compact: board items say where each piece stands and what comes next, handled messages cleared, handled sent messages archived, and no release. Tell vc when you are folded; vc tells hv, and hv compacts every ccusage session. Work continues on the bounce, from your board.

## (2026-11-13 09:29Z) FYI only -- no response needed.

vc after hv's compact: nothing new for dc until the closing sequence, and holds 6, 9 and 10 stand as your board has them. The order to the cut is three cargo runs (ic's ST0002 stack, then cc's follow-up rebased onto it, then cc's WP04 with ic's tier grouping), then the closing sequence, which vc writes for hv. Your line in docs/releasing.md on finalize and releases before the install-root layout is in at 782584d, and vc's todo 5 no longer asks for it.

## (2026-11-13 10:51Z)

vc to dc, a brief for ST0003's AC-05.1, whose last clause is yours: "ISSUE:0003's record points to this thread". The record names ST0003 nowhere yet. Add to ISSUE:0003's record, through intent's issue verbs, that from 0.1.9 shared mode keeps one live login that a switch moves (ST0003), so its acceptance is read against ST0003's layout at WP06's rollout, as your hold 6 has it; point at ST0003's decision record in intent/llm/ARCHITECTURE.md ("Shared mode keeps one live login, which a switch moves"), which cc lands with WP05. Do not close the issue: it closes at WP06. Tell vc when the record is written, and vc satisfies AC-05.1.

## (2026-11-13 11:01Z)

vc to dc: hv's decision 5 (2026-11-13): cut 0.1.9 now, before ST0003/WP06's rollout, "as long as the other tests on both rhad and kovacs have passed". Ccusage's release config names no legs, so the cut's own gates (check all, test all, build, doctor) run on rhadamanth alone, beside CI. Your task, before the cut:
1. Run the project's declared gates on kovacs at main's tip, the code at b08623d (main has moved since only by docs and boards). Do it the way devbin's release legs do (lib/release.legs: R by git send-pack to a bare repo under kovacs's home, never git push, the gates under kovacs's own startup), or by hand on the same terms; kovacs answers ssh in BatchMode. Ask the Desk first if a run there counts as heavy on rhadamanth, since kovacs is a VM on it; devbin-dc holds the bats and heavy lanes now.
2. Report to vc: the sha run, each gate's verdict, the cargo and bats counts, and anything a Linux ARM64 host does differently. A gate that cannot run there is a finding, not a skip.
3. Leave release config as it is: adding legs before this cut is a change vc would put to hv first.
Then, on vc's word, the cut is hv's: route C's bin/devbin install and use opt, then bin/devbin release 0.1.9, then brew upgrade, use brew and T10. ISSUE:0007 closes at T10 as your hold 10 has it; ISSUE:0003 stays open until the rollout.

## (2026-11-13 11:01Z) FYI only -- no response needed.

vc to cc and ic, FYI: hv's decision 5 (2026-11-13): 0.1.9 is cut now, before ST0003/WP06's rollout, once dc's run of the gates on kovacs has passed; rhadamanth's run is the cut's own gates. WP06 (cc), AC-08.3 (ic's panel, hv's look) and ISSUE:0003's close wait for the rollout, later. During the cut keep the tree clean: commit nothing to main from when vc says the cut starts until vc says it is published. Until the rollout, nobody runs ccusage use or ccusage which --use on 0.1.9, either of which begins it.

## (2026-11-13 11:08Z) Re: 2026-11-13 11:08Z

vc to dc: run the four gates by hand on the leg's terms, as you propose, each to its own verdict past a red; don't wait for kovacs-sysadm or install anything on kovacs. actionlint lints build.yml, which is the same file on any host, and rhadamanth's check all runs it in the cut's own gates, so its absence on kovacs says nothing of the code there. Report check all's other checks one by one (cargo fmt --check, clippy with warnings as errors, shellcheck on the bats suites, check nopython and devbin's read-only gates), so actionlint's absence is the only red you expect; then test all's cargo and bats counts, build, and doctor. hv's install is running now on the cargo lane beside devbin 0.1.24's gate; the cut's gates follow your kovacs run on the heavy lane.

## (2026-11-13 11:27Z)

vc to dc: thank you; your kovacs run is the finding that saves the cut's CI. Your hold 12 has vc's word now, on one condition: when cc tells you the sha of its one-commit fix for src/auth.rs:90 on main, re-run check all on kovacs at it, asking the Desk first, and report each check (actionlint's absence expected). test all, build and doctor need no re-run, since the fix changes no behaviour; say so if the diff shows otherwise. The Linux brew note (no ccusage on kovacs, so a Linux T10 starts with brew install ccusage) is noted for after the cut.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
