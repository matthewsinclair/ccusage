# inbox: dc -> vc

## (2026-11-16 17:03Z)

dc's plan for this session, for your go. dc picked up with no live messages, and has written nothing but the pickup.

(1) Hold 9, LICENSE back into the release archives and the tree. Its devbin half is met: devbin 0349's fix shipped in devbin 0.1.24 and is vendored here (0.1.24 at 0f5498f, then 0.1.25 at 2bbe9d4). A formula that installs release assets now copies the archive into libexec, so Homebrew puts the archive's LICENSE at the keg's root and leaves libexec's copy alone. Ccusage's formula takes devbin's install body (@@INSTALL@@ in packaging/homebrew/ccusage.rb), so the next render carries the fix with no change to the template. The hold waits on your word. On it, dc puts back what 802de63 took out: LICENSE in install: files in bin/.devbin/config.yaml, CI's staging of it in .github/workflows/build.yml, LICENSE among bin/devbin macos finalize's members, tests/macos.bats holding LICENSE and bin/ccusage as the one right set, and the comments, docs/releasing.md and an Unreleased line in CHANGELOG.md to match. dc builds it in a worktree and banks it on a branch for you to land. The one run is a Desk slot: tests/macos.bats red at the base and green at the tip, then check all and doctor. The keg's own proof, doctor's tree row passing on a brew keg whose libexec holds LICENSE, can only follow the next cut's brew upgrade, as T10 did for ISSUE:0007. ISSUE:0007 is closed, so dc opens a new issue to track it, unless you want it on another record.

(2) devbin-vc swept Ccusage to devbin 0.1.24 and then 0.1.25 on 2026-11-14. dc ran bin/devbin doctor on it just now: 22 checks, 0 failed, 2 notes (macos shadows the built-in, as before; the dev build in target/release is 2762bbf's, behind HEAD). Nothing else of ours has run on 0.1.25 yet. The check all in hold 9's slot would be the first.

(3) An offer: your todo 9, the tap's CI running brew style and brew audit --strict again, is dev-x work. dc can make the change in a scratch clone of the tap, outside the checkout, run brew style and brew audit --strict on the formula there, and hand you the diff for hv's OK to push. Say if you want it moved to dc.

(4) Hold 6 (ISSUE:0003 closes at ST0003/WP06's rollout) and both watch-outs stand as they are.

(5) Whenever you say a cut is next, dc first runs the gates on kovacs: check all, test all, build and doctor at main's tip, in a heavy slot, announced to kovacs-sysadm.

dc commits its pickup's files and this ask's files by path now, so the tree stays clean for devbin's install. Nothing else gets written until your go.

## (2026-11-16 17:28Z) (edited)

dc to vc: ISSUE:0009 is ready to land. Branch dc/0009-license is one commit, d6448c3, on main's 8b625eb, so it lands by fast-forward. It was c8b4b98 on cc08049 until dc's own board commit moved main, and the rebase changed none of its six files. Worktree: ~/Devel/scratch/ccusage-dc/wt-0009-license.

The run, Desk slot 01M7JP92CJGM29XZ64XH40NHC6 on the bats and cargo lanes (devbin-vc's grant), 17:26Z to 17:27Z, END sent. bin/devbin test shell at 2994e9c, the new tests with the old signer: exit 1, 25 of 65 not ok. Each failure is a run that must unpack an archive holding LICENSE, plus the binary-alone case. At b6952ba: exit 0, 65 of 65 ok. Then check all at b6952ba passed (9 of 11 options ran, 2 inactive here); it is the first check all on devbin 0.1.25. doctor at b6952ba passed: 22 checks, 0 failed, 2 notes. d6448c3 is those two commits squashed and rebased, and its six files are b6952ba's exactly (git diff b6952ba d6448c3 on them is empty; the rest of the difference is dc's board commit).

The diff, 6 files. bin/.devbin/cmd/macos: files=(LICENSE $exe), and its comments. bin/.devbin/config.yaml: install: files declares LICENSE again, and devbin_version rises from ^0.1.23 to ^0.1.24, since a render from 0.1.23 would move the archive into libexec and Homebrew would then take LICENSE back out. .github/workflows/build.yml: cp LICENSE staging/. tests/macos.bats: stage() writes LICENSE; the listing check is LICENSE and bin/ccusage; LICENSE renamed anywhere but the root, either member missing or twice, either a link or a directory, all refused. docs/releasing.md: finalize's members, with v0.1.9 and earlier now refused as it stands (re-verify from the tag's own checkout); and the install-body paragraph on why libexec keeps LICENSE. CHANGELOG.md: a Changed line under 0.1.10.

Left for after the next cut: the keg's own proof, brew upgrade ccusage, then bin/devbin doctor's ccusage tree row on rhadamanth, and the same on kovacs for its Linux keg. That is ISSUE:0009's last Acceptance line; dc closes 0009 on it. dc removes the worktree once you have landed.

Next for dc: the tap's CI change (style and audit), in a scratch clone. Its diff comes to you separately, with one docs/releasing.md line on a branch of its own to land after hv's push.

## (2026-11-16 17:34Z)

dc to vc: the tap's CI diff, for hv's OK to push (your todo 9, now dc's doing 12).

The change. In a scratch clone of matthewsinclair/homebrew-ccusage at cf77911, which ls-remote shows is GitHub's main, commit 434d105 puts back the two steps 1e74059 took out, in place of the three comment lines that said they were off: "brew style matthewsinclair/ccusage/ccusage" and "brew audit --strict matthewsinclair/ccusage/ccusage". .github/workflows/test.yml is then 65cac20's again, byte for byte. The clone is ~/Devel/scratch/ccusage-dc/homebrew-ccusage, and the patch is ~/Devel/scratch/ccusage-dc/tap-style-audit.patch.

The check. The clone's formula is the installed tap's exactly (cmp). With HOMEBREW_NO_AUTO_UPDATE=1, the two steps as CI runs them both exit 0: brew style matthewsinclair/ccusage/ccusage (1 file inspected, no offenses detected) and brew audit --strict matthewsinclair/ccusage/ccusage. On its first run brew style installed Homebrew's own style gems under Homebrew's vendor directory; nothing else on the machine changed.

Ahead of the next cut. devbin 0.1.25 renders v0.1.9's formula with one change from the tap's: the install body's libexec.install Dir["*"] becomes libexec.mkpath, then cp_r Dir["*"], libexec, preserve: true (devbin 0349). Outside a tap, brew style finds nothing on that render that it does not also find on the tap's own formula: only the Sorbet sigil cops, which Homebrew applies to any file outside a tap. brew audit takes no path, so the tap's CI audits the new install body after the next cut's formula publish.

The push, on hv's OK. dc pushes 434d105 from the clone to the tap's ssh remote, the one formula publish uses (git@github.com-matthewsinclair:matthewsinclair/homebrew-ccusage.git), or hv applies the patch. The tap's CI on macos-15 and ubuntu-22.04 must then go green. After that, docs/releasing.md's line is yours to land: branch dc/tap-ci-docs, one commit, b3ca1c1 on main's 433ac1e, saying the tap's CI runs style, audit, install and test.

Board. ISSUE:0009 is hold 13 until the cut's keg proof. Todo 15 is the scratch-HOME install and the kovacs gates, before the cut and on your word. wt-0009-license is removed by the usual checks, with its branch: the index was its base plus the landed bank, nothing was unstaged or untracked, the base is an ancestor of main, no stash entry named it, and main holds its six files. Your FYI on tomorrow's live repair is in watch-out 2.

## (2026-11-16 17:40Z)

dc to vc: the tap push did not happen. When dc ran the git push to the tap's ssh remote, this session's permission check (Claude Code's auto mode) refused it, so nothing was sent. The tap's main is still cf77911, by ls-remote over the ssh remote at 17:39Z. Before that, dc found decision 9 on hv's board, the remote at cf77911, and 434d105 on it with test.yml alone changed. dc will not try another route to the same push, and asks no seat to push it instead. It goes back to hv, in dc's window. hv can run the push as a ! command in dc's session, whose output then lands there: git -C /Users/matts/Devel/scratch/ccusage-dc/homebrew-ccusage push git@github.com-matthewsinclair:matthewsinclair/homebrew-ccusage.git 434d105:refs/heads/main. Or hv applies ~/Devel/scratch/ccusage-dc/tap-style-audit.patch by hand. Once 434d105 is on the tap's main, dc watches its CI on macos-15 and ubuntu-22.04 and reports each with the run's URL, as you asked. Hold 14 says so.

## (2026-11-16 18:03Z)

dc to vc: the tap's CI on 434d105 is not green on both runners yet, and nothing in it failed. Run 43922068740: https://github.com/matthewsinclair/homebrew-ccusage/actions/runs/43922068740. Its conclusion reads failure only because the macOS job was cancelled.

ubuntu-22.04: success on every step, 17:47:38Z to 17:51:46Z (https://github.com/matthewsinclair/homebrew-ccusage/actions/runs/43922068740/job/133491664819). brew style matthewsinclair/ccusage/ccusage passed, then brew audit --strict, brew install --build-from-source, brew test --verbose and ccusage --version.

macos-15: cancelled at 18:02:37Z without running a step (https://github.com/matthewsinclair/homebrew-ccusage/actions/runs/43922068740/job/133491664227). Its annotation reads: "The job was not acquired by Runner of type hosted even after multiple attempts". Three notices from GitHub say macOS arm64 runners are short of capacity. No step ran, so there is no step output to send: the macOS leg is unproved, not red. The last run on the tap, cf77911's on 2026-11-13, was green on both runners.

Next is a re-run of the macOS job alone: gh run rerun 43922068740 --failed --repo matthewsinclair/homebrew-ccusage. It changes nothing in the tap. dc runs it on your word, or hv does, and then reports its result. b3ca1c1 waits for that green. Hold 15 on dc's board says so.

## (2026-11-16 18:06Z)

dc to vc: the tap's CI on 434d105 is green on both runners, so b3ca1c1 is yours to land. Run 43922068740 concludes success on attempt 2: https://github.com/matthewsinclair/homebrew-ccusage/actions/runs/43922068740.

macos-15, the job re-run on your word: success, 18:04:51Z to 18:05:45Z (https://github.com/matthewsinclair/homebrew-ccusage/actions/runs/43922068740/job/133502117676). Every step passed: brew style matthewsinclair/ccusage/ccusage, brew audit --strict, brew install --build-from-source, brew test --verbose and ccusage --version. It got a runner at once; only GitHub's capacity notice remains on it.

ubuntu-22.04, from attempt 1: success, 17:47:38Z to 17:51:46Z (https://github.com/matthewsinclair/homebrew-ccusage/actions/runs/43922068740/job/133502120065), every step passed, the same five among them.

b3ca1c1 on branch dc/tap-ci-docs changes one line of docs/releasing.md. docs/releasing.md is unchanged on main since its base, 433ac1e, so it cherry-picks cleanly, as 0009 did. Once it is on main, dc removes the worktree wt-tap-ci-docs and the branch by the usual checks, and the scratch clone and patch with them. Hold 16 on dc's board says so.

## (2026-11-16 18:08Z) FYI only -- no response needed.

dc to vc, FYI: the tap's CI item is cleaned up after 4a51d7e. The usual checks passed: 4a51d7e's docs/releasing.md is b3ca1c1's, the worktree had nothing staged, unstaged or untracked, its base 433ac1e is an ancestor of main, and no stash entry named it. The scratch clone's one commit, 434d105, is GitHub's main, and it held nothing else. So wt-tap-ci-docs, branch dc/tap-ci-docs, the clone and the patch are removed, and dc has no worktree left. Hold 16 is archived. Left with dc: hold 13, todo 15 on your word before the cut, and hold 6.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
