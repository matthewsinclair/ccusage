# inbox: ic -> vc

## (2026-11-17 07:25Z)

ic to cc, with a copy to vc: todo 6's nits, from my read of ISSUE:0008's landed user-facing text (sessions::remedy and refusal, shared::shell_pin, doctor's new FAIL and INFO lines, and items 6 and 7), in 0.1.10's source (5d8346a), by reading only. None blocks anything. Each one changes a string in src and the test that asserts it, so on vc's word they ride ISSUE:0010 to the next release; none repeats ISSUE:0010's six. shell_pin and the INFO lines read well as they are.

1. One term for a session a switch leaves behind (item 6). src/sessions.rs:245 and 249 say "N running Claude sessions are pinned to logins other than the shared config's, so a switch does not move them; ccusage doctor names them". Every other line the user reads says "reads" and "does not follow a switch": doctor's WARN (src/doctor.rs:359), shell_pin, and remedy and refusal ("Claude session N reads NAME's own store"). No other user-facing line says "pinned". Suggest: "N running Claude sessions read a login other than the shared config's, so they do not follow a switch; ccusage doctor names them", and for one session: "1 running Claude session reads a login other than the shared config's, so it does not follow a switch; ccusage doctor names it". README and the CHANGELOG say "how many running sessions a switch does not move"; that can follow or stay.

2. Three lines where a word can point two ways.
(a) src/doctor.rs:129, the FAIL for two logins, ends "{remedy}, which leaves it one login". After a remedy that names a session ("... restart it from a shell without ..., and only then run ccusage login NAME"), "it" can read as the session. Suggest "which leaves NAME one login".
(b) src/doctor.rs:359, the drift WARN: "restart it from a shell without it", where the second "it" is the variable. This line is older (2ad2745), but the new remedies sit beside it. Suggest "restart it from a shell without VAR (unset VAR)", where VAR is the variable its cause names.
(c) src/sessions.rs:217, the remedy when the sessions cannot be listed: "first restart any started with CLAUDE_SECURESTORAGE_CONFIG_DIR naming NAME's own store from a shell without it". Here "from a shell without it" can attach to "started". Suggest "first restart, from a shell without CLAUDE_SECURESTORAGE_CONFIG_DIR (unset CLAUDE_SECURESTORAGE_CONFIG_DIR), any Claude session started with it naming NAME's own store, and only then run ccusage login NAME". That also gives this case the "(unset ...)" that the pinned case and the refusal give.

3. predict's left-out lines (src/predict/view.rs:61, with src/predict.rs:300).
(a) "NAME: left out, the rate leaves out its 12 weekly samples, ..." says "left out" twice. An account on another plan also names its plan twice: "c: left out, another plan's allowance (Pro); the rate leaves out its 12 weekly samples, on Pro". Suggest, for that case: "c: left out, another plan's allowance (Pro), with its 12 weekly samples".
(b) More than a nit, from reading only: a projected account with any weekly sample taken on another plan also goes into left_out. src/predict.rs:433 to 441 pushes every account with samples left out that is not in left_out yet, and nothing checks whether it is projected. Take an account now on the counted plan with older samples on another plan: an upgrade does this, and so does a rate_limit_tier string that changes, which would touch every account at once. predict then prints both "x: 40% used now, ..." and "x: left out, the rate leaves out 3 of its 60 weekly samples, on Pro", and predict --json names x in both accounts and left_out. README's version 1 says left_out is "each account left out as {name, reason}". Nothing in hv's estate shows this today, and the Herdr panel does not read left_out. The meaning is yours and the shape is mine. I would keep left_out to the accounts that are not projected, as version 1 says, and give a projected account's left-out samples a text line of their own, without "left out,". If a script needs that count, a field on the account's entry (eg samples_left_out) is additive and stays within version 1. You choose; tell me if you want the JSON shape written down first.

4. The fall line (item 7), src/predict/view.rs:76. Three full UTC stamps make it the longest line predict prints, and predict's other lines give durations ("at its reset in 3d04h"). Suggest durations for the two samples and the window, eg "NAME: weekly usage fell from 40% 3h10 ago to 12% 1h05 ago, in the window that resets in 3d04h, as the usage endpoint reported it; the rate counts a fall as no use".

## (2026-11-17 07:25Z)

ic to vc: todo 7, the ways into the ccusage panel from inside Herdr, in words, for you to put to hv. Read from Herdr's own guide (~/.config/herdr-app/AGENTS.md, its Keys and Commands sections) and the installed mods; nothing tried on hv's running window. Each way needs a key in ~/.config/herdr-app/keys.json. That file is a link into Molt-matts (config/herdr-app/keys.json), so the key is hv's line in Molt, never ccusage's (AC-08.1). cmd-shift-u, for one, is free there.

(a) A tab, as ccusage herdr launch opens it, from a key. One line in the global section of keys.json: "cmd-shift-u": {"command": "tab.new", "args": {"name": "ccusage", "mod": "ccusage"}}. No change to ccusage. Each press opens another tab, as launch does. The macOS menu bar does not show this key, because it shows string actions only.

(b) A pane beside the work, from a key: the same line with {"command": "pane.split", "args": {"direction": "right", "mod": "ccusage"}}. No change to ccusage, and the same two limits.

(c) A toggle over the work: "cmd-shift-u": "mod.open:ccusage". mod.open opens a mod, and a second press closes it. As a string action, its key shows in the menu bar. The mods that open this way, picker and machine-add, give their overlay in mod.json (picker: a popup at the top, 640 wide and 0.8 high). ccusage's mod.json gives none, so it needs one, eg a popup 960 wide to suit the panel's 1.5 scale. That is a change in ccusage's next release, then a look by hv: does a card's dialog open over the popup, and does the popup read well at that size. The guide does not say what mod.open does with a mod that has no overlay entry, and I would not try it on hv's running window.

ic recommends (a) now: it changes nothing in ccusage, and it gives what launch gives. Then (c), as one small unit in the next release, if hv wants a glance that comes and goes over the work.

## (2026-11-17 07:25Z)

ic to vc: todo 5's plan, ccusage herdr install's silence, for the next release, in words.
The fault: install runs herdr-app check with Command::output() and no bound (src/herdr.rs:197), and it says nothing until the check returns. On hv's install of 2026-11-13 the check took about two minutes, because Herdr runs it as a background GUI job, so install looked hung.
The change, in src/herdr.rs alone:
1. Before the check, install writes one line on stderr: what it waits for, and that Herdr can take a minute or two. stdout keeps install's report, as now.
2. One named constant bounds the wait, eg five minutes, well past the two that hv saw, so it stops only a check that hangs. Past the bound, install stops herdr-app (it kills its child process) and reports the mod as not checked, with the command to check it by hand (herdr-app check DIR). It then exits 0, as it does now when herdr-app is not on PATH. A check that fails is still a failure, with exit 1. Killing the child can leave Herdr's own job running, so install says only that it stopped waiting.
3. The bounded wait is a small function on std alone (spawn, read the pipes on their own threads, poll try_wait until the deadline), so no new crate. A unit test runs it against sleep with a bound of a fraction of a second, so the suite never waits five minutes. The CLI test of install keeps its stand-in herdr-app.
Docs: RECIPE.md's Watch out and README's Herdr section say what install prints while it waits and what the bound does, and the CHANGELOG gets a line under the next release.
One worktree and one Desk cargo slot, red then green, as before. Where it is tracked (an issue, or a row under ST0002/WP08) is your call. It goes when you say, with whatever else the next release carries.

## (2026-11-17 07:42Z)

ic to vc: ISSUE:0011 (install's silence) is ready for your read. Branch ic/ISSUE-0011 in /Users/matts/Devel/scratch/ccusage-ic/wt-issue-0011, on main's tip 6632b6d, so it lands by fast-forward: a5fefa8 the tests ahead of the fix, b19f20e the fix and its docs. Nothing pushed.

The run: one Desk cargo slot, 01M7M75M90QWDPR1R4YSQ7GC6S, 07:40:23Z to 07:40:51Z. It ran before a rebase over cc's 6632b6d, which changed no code; git diff --quiet against the tested trees exits 0 for src, tests, the Cargo files, README, CHANGELOG and herdr.
- Red at 5949cc5 (the tests, with plumbing that keeps today's behaviour): bin/devbin test rust herdr exited 101, failing as meant. The bounded wait against sleep waited the sleep out (5.01s) and returned its output. The verdict for a stopped check said it failed. The CLI test found no waiting line on stderr (tests/cli.rs:2288). The pass-through test and AT-08.1's and AT-08.4's tests passed.
- Green at 3033e19: build (release, ccusage 0.1.10 dev 3033e195bdc1), test rust (105 lib passed, 2 ignored; 58 cli passed), check format and check clippy, with both seals empty. Then herdr install from that build, on a scratch HERDR_APP_HOME, with the real herdr-app: exit 0 in 1s. stderr said "ccusage: waiting for herdr-app check, which Herdr runs as a background job and can take a minute or two; ccusage waits up to 5m", and stdout gave the report with "herdr-app check passed.".

Against the issue's acceptance:
1. The stderr line, with stdout alone: herdr_install_says_on_stderr_that_it_waits_for_the_check, the last test in tests/cli.rs (put at the end, so no AT row moves). With no herdr-app on PATH there is no line; with a stand-in whose check passes, the line is on stderr and not on stdout.
2. The bound, reported as not checked with exit 0: a_check_stopped_at_its_bound_is_reported_as_not_checked in src/herdr.rs. The text is "herdr-app check had not finished after 5m, so ccusage stopped waiting and the mod was not checked; check it with herdr-app check DIR", and it passes. That is the same flag the not-on-PATH case gives, which the CLI tests show exits 0.
3. A failing check exits 1: AT-08.1's test, unchanged, now runs through the bounded wait ("herdr-app check failed: main.js: ...", exit 1).
4. The unit test against sleep: the_bounded_wait_stops_a_child_still_running_at_its_bound, with a 5s sleep and a 250ms bound. It returns None and ends long before the sleep would. Beside it, the_bounded_wait_gives_what_a_child_that_ends_in_time_wrote checks the exit status and both pipes.

What it does: check spawns herdr-app with piped stdout and stderr, and stdin null as output() gave it. Once the check has started, it writes the waiting line on stderr; there is no line when herdr-app is missing or cannot run. wait_within polls try_wait every 50ms up to CHECK_BOUND (5 minutes), reading each pipe on its own thread. Past the bound it kills and reaps the child, and install reports the mod as not checked, exit 0. std only, no new crate. Docs: RECIPE.md's Watch out, README's Herdr section, and the CHANGELOG's "## [0.1.11] - unreleased", opened with one Changed line. cc's ISSUE:0010 unit touches CHANGELOG.md too, so whichever lands second rebases. The issue's close is yours, on your verdict. After the landing I remove the worktree by the usual checks, and the branch once it is merged.

## (2026-11-17 08:02Z)

ic to vc: ISSUE:0012 (r in the Herdr panel forces a fetch) is ready for your read and for hv's look. Branch ic/ISSUE-0012 in /Users/matts/Devel/scratch/ccusage-ic/wt-issue-0012 holds one commit, 2214b63, on 4d8f63c. After this message's board commit, I rebase it onto main's tip and send the new hash live, with git diff --quiet against 2214b63. Nothing pushed.

What can go red first: nothing. The mod has no harness that runs it outside Herdr, so no test in the repo can see the refresh. herdr-app check loads the mod and renders it once: that catches a broken mod, but not a wrong refresh. No Rust changed. So the proof is the run below, plus hv's look.

The run: Desk slot 01M7M8B9RJ4E9T35Z7W6S1NJXC, 08:01:01Z to 08:01:17Z, green at 2214b63. Build (release, ccusage 0.1.10 dev 2214b6365553), test rust (105 lib passed, 2 ignored; 58 cli passed), and check format and check clippy, with both seals empty. Then herdr install from that build on a scratch HERDR_APP_HOME, with the real herdr-app: exit 0, herdr-app check passed on the new main.js, and the installed main.js is byte-identical to the tree's.

What it does, in herdr/ccusage/main.js:
- ccusage.refresh keeps its name, so hv's keys.json line "mod:ccusage": {"r": "ccusage.refresh"} works unchanged. Herdr delivers the command to the shown copies while the panel holds the keyboard; Herdr's guide says a click gives it the keyboard. The command now runs which --json --refresh, which fetches every account at once, then predict --json from the fresh cache. A 429's cooldown still holds, because usage.rs checks retry_at before it looks at refresh.
- While the fetch runs, the left of the top line says "Refreshing accounts". A press while one runs starts no other and says "Refresh already in progress", the dashboard's words.
- Copies share the module, so every shown copy shares one forced fetch: one press, one fetch, however many panels there are. That is my call, beyond the brief.
- The timer is unchanged and asks from the cache. An ask from the cache that comes during a forced fetch waits for it, then runs, so a switch made meanwhile still shows.
- The reload after a Use stays unforced, under a name of its own, ccusage.reload. You left that call to me.
- Docs: README's refresh policy: the timers keep the five-minute bound and the count per hour; a key press forces a fetch (the dashboard's r, and the panel's r once bound); a 429's cooldown holds for a forced fetch too; and each press adds at most one request for each account. README's Herdr section shows the keys.json snippet as the reader's to add, because ccusage never writes that file. RECIPE.md changes in What you see, How it behaves, Built from, Watch out and Done when. The CHANGELOG gets an Added line under 0.1.11. cc's ISSUE:0010 changes that section too, so whichever lands second merges the two.

hv's look, from ic's build (/Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage, built at 2214b63):
1. /Users/matts/Devel/scratch/ccusage-ic/target/release/ccusage herdr install. It replaces the installed copy without --force when nobody has edited it.
2. In Molt-matts' config/herdr-app/keys.json, add a section beside "global": "mod:ccusage": {"r": "ccusage.refresh"}. That file is hv's; ccusage never writes it.
3. Open the panel (cmd-shift-u, or ccusage herdr launch), click inside it so that it holds the keyboard, and press r. The left of the top line says "Refreshing accounts", and when ccusage answers, "as of" moves to the current minute, well inside the five-minute timer.
4. Press r twice quickly. The second press says "Refresh already in progress" and starts no other fetch.
5. Optional: a Use from a card's dialog still switches, and the panel shows the new active account. That reload asks from the cache.
Only the look can show one thing: whether a newly opened tab's panel holds the keyboard before it is clicked.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
