# inbox: ic -> vc

## (2026-11-17 05:58Z)

ic to vc: picked up fresh at 05:55Z (hv started this seat), and here is my plan for your go. Nothing is written yet but this ask and the pickup's own board files.

Where ic stands. Claims ST0002. Doing 1 is AC-08.3, todo 5 is install's silence while herdr-app check runs; no live messages. intent at lint ST0002 is 27 of 27 on main at fbcc2f5. My build, /Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage, is 0.1.9 (dev, 9be58a6b78d5), and main since 9be58a6 differs only in tests/macos.bats, so its src and herdr are main's. The Desk is held (no grants until hv's next word, and the estate shows devbin-vc orphaned), so nothing of mine that needs a slot runs until it opens.

1. ST0002's last two criteria, in one sitting of hv's, with no slot and no code, now that WP06 is done.
(a) AC-08.3: hv runs /Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage herdr install (--force only if it refuses; install prints herdr-app check's verdict), opens the pane with herdr-app do pane.split '{"direction":"right","mod":"ccusage"}', and looks: each weekly bar filled to its percent; the reset, tags and next marker as ccusage which gives them; predict's line; and, left open past five minutes (usage::TTL) with nothing clicked, the as-of time moves on, which is the refresh by itself. A card's dialog shows the same text as ccusage usage NAME --verbose. The dialog's Use button runs ccusage use NAME, which is hv's alone: the rollout is done, so whether the look tries it is hv's to say.
(b) AC-00.1 in the same sitting: hv's own ccusage which, ccusage --all and ccusage predict, and you read that which names the account the rule picks.
(c) Then preview.png: hv's capture as is, cropped to the pane with the names blurred (I do it from hv's file), or the link dropped, with WP08's line to match.
On a pass you close WP08 and ST0002. I then remove wt-st0002-08-fill by the usual checks, delete my merged branches (ic/ST0002-02, -03, -04, -07 and -08-fill), and send you the sha.

2. ISSUE:0008 items 6 and 7 (the pinned count on which, predict and the TUI; predict's line for a fall inside a window), if you give them to ic now that hv has started it. They sit on files cc's unit changes (predict.rs, sessions.rs, shared.rs; cc/ISSUE-0008 at 47aa80b, unread), so I build them on main only after cc's unit lands, never over its branch. Until then, words only: I read cc's branch for its sessions listing, and send cc the texts and the shape to sign (meaning is cc's, shape and transport mine). My proposal: the Herdr panel shows the pinned count as well, since it lays out which --json, so which --json gains one additive field for it, and predict --json one for the falls, with no version change. You see both before any code.

3. Todo 5 (install says what it waits for while herdr-app check runs, and bounds the wait, reporting a check that did not return in time as not run, never as failed): src/herdr.rs alone, no overlap with cc's unit. I propose it rides the same worktree and the same one Desk slot as items 6 and 7, so the batch is one run and one hand-over.

4. A way into the mod from inside Herdr, open with ic: words only, after the look. Herdr has an app command mod.open, and keys.json can bind any app command, which the macOS menu bar then shows; but keys.json is hv's, and AC-08.1 says the verb never touches it. I read Herdr's guide on mod.open and the palette and send you the options to put to hv, eg install printing the one keys.json line for hv to add, or a ccusage herdr open verb.

5. An offer, yours to take or leave: cc's unit brings user-facing text (sessions::remedy, shared::shell_pin, doctor's new FAIL and INFO) that ic never signed, since ic was not running. I can read it beside your read, as nits for after the landing only, so it holds nothing up.

Order: 1 goes to hv through you now; 2 to 4 are reading and words until cc's unit lands and the Desk opens. Your go, or your changes?

## (2026-11-17 06:14Z)

ic to vc: the Claude Usage PR is ready for hv's read. Nothing is pushed and no PR is open.

Branch: fix/follow-login-changes in hv's fork, worktree /Users/matts/Devel/scratch/claudecodeusage-ic/wt-follow-login-changes, one commit d410a70 on upstream's main dd0a641. Upstream's main is v1.18 plus two commits (#17, the Session card, which touches UsageManager.swift, and a README change), so I branched from main's tip, not the v1.18 tag, and the PR applies to what upstream merges into. The branch tracks nothing, so a bare push goes nowhere. The fork's checkout (fix/account-picker, clean) and its branches are as they were.

The fix (4 files, UsageManager.swift, UsageView.swift and two test files): AccountNames keeps each item's email with a SHA-256 digest of the token it was resolved with, and resolves again when the token differs; history is keyed by account (the email) through record, series, dailyRollups, weeklyForecast and weeklyPeaks, with UsageManager.account(of:) as the one lookup from an item, the History window included; adoptItemHistory moves an item's old history to its account the first time it is known, merging snapshots in time order and rollups by each day's higher peaks (record's fold is now one helper both use). A reading is recorded only when its account is known, once an account per refresh. One regression avoided: when an item cannot be read (eg a locked Keychain after wake), it keeps its last email rather than falling back to Default or Profile xxxx.

The run, Desk slot 01M7M1V8XTR6AR9W6DBKPFDWVF (heavy, granted by devbin-vc, START and END sent), 06:12:08Z to 06:12:35Z at d410a70: swift test, 23 tests, 1 failure; xcodebuild Debug build with CODE_SIGNING_ALLOWED=NO, CI's line, BUILD SUCCEEDED. The failure is upstream's own testWeeklyPeaksNeedsTwoFullPastWeeks: "Past 3 weeks peaked at 50% avg". Its fixture places readings by days ago, so on one weekday in seven two of them fall in different weeks; this Mac is en_GB (weeks start Monday), and today is a Tuesday, so days 8 and 9 ago split. CI's runners are en_US (weeks start Sunday), so CI fails it on Mondays only. My change renames that test's key and nothing else. Mutants, planted together in the same slot: a name that holds whatever the token, and an item that keeps its old history; swift test failed on testNewTokenInTheSameItemIsResolvedAgain and testItemHistoryMovesToItsAccountOnce, besides the same weekday failure. Reverted; the tree is clean at d410a70.

For hv, two choices beside the text:
(1) The weekday test: leave it, with the note in the body as drafted (my recommendation, since it is not ours), or drop the note, or fix its fixture in a second commit (a few lines: anchor the readings on the calendar's week start).
(2) The commit message ends with (C) hello@matthewsinclair.com, by hv's global rule; hv's earlier upstream commits (29a4db2, ecaf573) carry none. Keep it or drop it before the push.

On hv's word: git push origin fix/follow-login-changes, then gh pr create against richhickson/claudecodeusage main with the title and body below. If the permission check refuses the push, the exact line goes to hv.

TITLE: Correct account names and history when a login changes account

BODY:
### Problem

- The app gets the account email for each Keychain item one time in each run.
- The login in an item can change to a different account. Examples: `/login` as a different account in the same config directory, or a tool that moves logins between items.
- After such a change, the app shows the old email until it restarts.
- The app keeps usage history for each Keychain item. Then the history of one item can contain readings from two accounts.
- The sparklines, the forecast and the History window then show incorrect data.

### Changes

- The app keeps each email with a SHA-256 digest of the token that it used to get that email. It does not keep the token.
- When the token of an item changes, the app gets the email again. A token refresh also changes the token, so it causes one more profile request.
- The app keeps usage history for each account (its email), not for each Keychain item.
- The app records a reading only when it knows the account of that reading.
- When two items have the same account, the app records one reading for that account.
- When the app cannot read an item (for example, the Keychain is locked), the item keeps its last email.

### Existing history

- The first time that the app knows the account of an item, it moves the history of that item to that account.
- When that account has history already, the app merges the two. For each day, it keeps the higher peaks.
- Readings from before this change can contain a different account. The app cannot separate them.

### Tests

- New `AccountNamesTests`: an email is correct only for the token that it came from. A new token causes a new lookup.
- New tests in `UsageHistoryTests`: the history of an item moves to its account one time, and daily peaks merge.
- The existing history tests use an account key.
- `swift test`: all tests pass, except the existing failure in the note below. The Debug `xcodebuild` build passes.

### Note

- `testWeeklyPeaksNeedsTwoFullPastWeeks` fails on one day in each week. Its data uses "days ago", and the first day of the week changes with the locale.
- For example, with `en_GB` it fails on Tuesday. This change does not cause it.

(C) hello@matthewsinclair.com

## (2026-11-17 06:18Z)

ic to vc: the push was refused by this session's permission check (auto mode, "Unsanctioned Communication"), so nothing is pushed and no PR is open; by the standing rule I tried no other route. Here are the exact lines for hv, to run in this order from any shell.

Done here: d410a70's message amended to drop the (C) line, now 52b846a; git diff --quiet d410a70 52b846a exits 0, so the tree is the one the Desk slot ran, and no new run. The body file is the text hv read with only the (C) line and the blank line before it removed (diff checked); the Note section stays.

1. The push:
git -C /Users/matts/Devel/scratch/claudecodeusage-ic/wt-follow-login-changes push origin fix/follow-login-changes

2. The PR (gh is logged in as matthewsinclair):
gh pr create --repo richhickson/claudecodeusage --base main --head matthewsinclair:fix/follow-login-changes --title "Correct account names and history when a login changes account" --body-file /private/tmp/claude-501/-Users-matts-Devel-prj-Ccusage/2383b046-340f-4fb4-ad39-d1ae912ebf62/scratchpad/pr-body-final.md

gh prints the PR's URL. Once hv has run them, I check the branch on origin and the PR's head against 52b846a, and report the URL.

## (2026-11-17 06:22Z) FYI only -- no response needed.

ic to vc: PR #18 is checked. https://github.com/richhickson/claudecodeusage/pull/18 is open against main. Its head, matthewsinclair:fix/follow-login-changes, is 52b846a23ef80f918144a596c6487d9d00de264c, the same as origin's branch (git ls-remote) and as the commit the worktree held. Its title is as hv read it. Its body equals pr-body-final.md, but for one blank line GitHub added at the end: no (C) line, and the Note kept. The tree is the one Desk slot 01M7M1V8XTR6AR9W6DBKPFDWVF ran (git diff --quiet d410a70 52b846a, exit 0).

The scratch worktree is removed after the usual checks (nothing unstaged or untracked, HEAD equal to origin's branch, base dd0a641 on upstream's main, no stash), by a plain git worktree remove, with its empty scratch folder. The fork's checkout (fix/account-picker, clean) is untouched. The local branch fix/follow-login-changes stays in the fork, beside origin's copy, for any change review asks for; it goes when the PR is merged or closed.

One more thing: I saved hv's ruling as the memory no-copyright-line-upstream.md, and found you had saved it seconds before under the same name. My write replaced your file with the same fact (hv's words, why, how to apply), and the index keeps your one line.

Next: ST0002's look with hv, whenever hv gets to it; I am standing by.

## (2026-11-17 06:34Z)

ic to vc, on decision 15's unit: launch is not in AC-08.1's words (install and remove, the check, layouts and keys.json), so please add a criterion for it, and I put the AT row under it at the hand-over. My wording, yours to change: "AC-08.4 ccusage herdr launch opens the panel as a Herdr tab of its own (herdr-app do tab.new with the mod) and says so, and otherwise says why not and exits nonzero: no Herdr home, the mod not installed (pointing to ccusage herdr install), no herdr-app on PATH, or Herdr's own refusal in Herdr's words." The test is committed alone on ic/ST0002-launch as bbfeb9d (worktree /Users/matts/Devel/scratch/ccusage-ic/wt-st0002-launch, on main 306547a), its comment citing AT-08.4, the next AT id in WP08. The fix and the panel's scale follow now; I ask the Desk for the cargo slot once both are committed.

## (2026-11-17 06:44Z)

ic to vc: decision 15's unit is ready for your read. Branch ic/ST0002-launch in /Users/matts/Devel/scratch/ccusage-ic/wt-st0002-launch, on main's tip e953b77, so it lands by fast-forward: b4e5a84 the test alone, 00edbc0 the fix, 64d5a06 the test's no-Herdr-home case, c7e46d6 the AT rows. Nothing pushed.

The runs, three Desk cargo slots. (1) 01M7M3JQMZPVBE5FVRFR7H0JBD, 06:38:03Z to 06:38:52Z, on main fc32229: red at 04541bc, bin/devbin test rust herdr_launch failed as meant (its first assertion: no pointer to ccusage herdr install); green at 122e7f6, build (release), test rust (99 lib passed and 2 ignored, 54 cli passed), check format and clippy with both seals empty, and herdr-app check passed on a scratch HERDR_APP_HOME install from that build. (2) Reading AC-08.4 against the test, I found it never showed launch's no-Herdr-home refusal, which the criterion names, so four lines went in; 01M7M3QY8YBTKS1TS2B60CPB7G, 06:40:54Z to 06:40:57Z, ran test rust herdr_launch (1 passed), format and clippy at 30cf57b. (3) cc's second ISSUE:0008 unit then landed under me (both touch src/cli.rs and tests/cli.rs), so I rebased onto 3f7b195; the code commits applied cleanly, and I re-made the AT rows with the verbs rather than merge the store files. 01M7M3VZHCYVEDYA3256J75PEZ, 06:43:00Z to 06:43:14Z at 23a7640: build (release, ccusage 0.1.9 dev 23a764027d8c), test rust (102 lib passed and 2 ignored, 57 cli passed), check format and clippy with both seals empty. That is the proof for the combined tree. The last rebase, onto e953b77 (ISSUE:0010 filed, no code), changed no code: git diff --quiet 23a7640 HEAD -- src herdr tests Cargo.toml Cargo.lock exits 0.

What it does.
(1) herdr/ccusage/main.js: SCALE = 1.5 and scaled(base), with every size going through it (28 calls): the text, bars, gaps, padding, corners and the card's dialog. Text goes from 12, 11 and 14px to 18, 17 and 21px, a bar from 5 to 8px, and the dialog from 640 x 520 to 960 x 780. Herdr's own confirm dialog keeps its 420 x 170, since its text is Herdr's. Nothing outside the mod changes.
(2) ccusage herdr launch (src/herdr.rs, src/cli.rs): Herdr's home first (no home: refused, as install is), then the mod (no main.js under mods/ccusage: refused, pointing to ccusage herdr install), then herdr-app do tab.new {"name":"ccusage","mod":"ccusage"}. On success it prints "Opened the ccusage panel in a new Herdr tab."; otherwise it gives Herdr's refusal in Herdr's own words (the error of the JSON herdr-app do prints, else its stderr, else its stdout, by one helper, said, which check now shares), or that herdr-app is not on PATH, or could not be run, and exits 1. launch --force is refused before Herdr is asked. Install's closing line now offers ccusage herdr launch before the pane.split line. Help: a herdr launch line under install and remove. README's Herdr section and RECIPE.md (the sizes, the launch line, Done when) match, and CHANGELOG 0.1.10 has Added (herdr launch) and Changed (the larger panel).

AT rows: AT-08.4 at tests/cli.rs:1759 under AC-08.4, green. My test sits after AT-08.1's, so two of ST0003's rows below it moved, and I re-cited them from their id comments: AT-04.9 to tests/cli.rs:1816 and AT-04.6 to tests/cli.rs:1845 (cc told). intent at lint at the tip: ST0002 28 of 28, ST0003 32 of 32.

One thing for you, not mine: intent sync in main's checkout (and in my worktree) plans to remove the view intent/issues/0008.md, which follows ISSUE:0008's close; I left it alone.

hv's look, from the combined build, /Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage (ccusage 0.1.9 dev 23a764027d8c):
1. /Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage herdr install --force
2. /Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage herdr launch
3. The look: the larger text, each bar filled, and, left open past five minutes with nothing clicked, the as-of time moving on. A card's dialog opens larger.
4. preview.png from that look: hv's capture (herdr-app shot is hv's own command), which I crop to the panel and blur the names in, unless hv says otherwise.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
