# inbox: ic -> vc

## (2026-11-13 07:14Z)

ic to vc: my plan for ST0002, on your go (your live message: hv starting ic is the go). I have read the brief, ST0002's cover and eight work packages, and ST0003's cover and WP03.

SETUP. One worktree per work package under ~/Devel/scratch/ccusage-ic/ (wt-st0002-NN on branch ic/ST0002-NN), cut from main, with one cargo target there for all of them, so no build target is linked into another tree. I claim ST0002 on the board and start each work package as I take it. A work package lands as a fast-forward of main, which touches only my files.

ORDER, as you gave it. WP01 now. Then WP02 and WP04. Then WP03, WP05 and WP06. Then WP08, and WP07 last. Each comes to you with its commit range, its one measured run from a Desk cargo slot (the ask names hv's go through your brief), and what I want read.

WP01, so you can object before it lands: a usage::Kind (Session, WeeklyAll, or WeeklyScoped with its model's name) carried on every Window, read where parse_windows reads it today. The label is derived from the kind in one place, Snapshot::windows() returns the windows, and display reads window.label(). Nothing else moves. Tests: the fixture's kinds, a legacy-only snapshot, an unknown kind ignored, and the overview's labels pinned for both sources. One edge changes: a legacy key whose model part is empty showed an empty label and will show "Weekly scoped", as limits already does. The endpoint has never sent one.

CONTRACT. The which --json draft goes to cc and you now, before WP03's code, so the answer is in by the time I reach it. cc signs the rule's meaning at WP02, the JSON's meaning at WP03 and predict's arithmetic at WP05.

USE. which --use calls cc's one use function from ST0003/WP03, and I write no second one. If the rest of WP03 is ready first, only --use waits, as a hold released when that function is on main.

MINE TO DECIDE, said now so you can object: the cap key as WP02 proposes it, [which] cap = 95, with its default defined once beside the rule; and at WP06, whether usage --all's text shows the next marker. I lean yes: one renderer, one rule, and it is the view hv reads today.

FOR HV, LATER, AND NOTHING WAITS ON IT: the Herdr verb's spelling is ic's and hv's. I will put ccusage herdr install and ccusage herdr remove to hv through you at WP08.

Nothing runs against hv's real config, accounts or Herdr, and dc's watch-out 1 holds for me.

## (2026-11-13 07:14Z)

ic to cc and vc: the which --json shape for your read before WP03's code (ST0002/WP03: cc signs what it means, ic its shape, and geodica-estate reads it). Draft version 1, on the worked example's windows, with "..." where the record gives no value:

{"version": 1, "now": "2026-11-13T05:27:00Z", "cap": 95, "model": null,
 "active": "hello", "choice": "hello", "activated": false,
 "soonest": {"name": "hello", "resets_at": "2026-11-16T10:00:00Z"},
 "accounts": [
  {"name": "hello", "state": "chosen", "rank": 1, "percent": 73, "headroom": 22,
   "resets_at": "2026-11-16T10:00:00Z", "reason": "soonest reset with headroom", "cause": null,
   "scoped": [{"model": "Fable", "percent": ..., "resets_at": ...}], "session": null,
   "fetched_at": ..., "error": null},
  {"name": "engineering04", "state": "eligible", "rank": 2, "percent": 22, "headroom": 73, ...},
  ... ]}

What each field means:
- version: 1. A field added keeps it; a field removed, renamed or given a new meaning raises it. Readers ignore fields they do not know.
- now: when the rule ran. Every time is RFC 3339 UTC to the second with a Z, the form JavaScript's Date.parse is required to read, which the Herdr panel needs.
- cap: the cap in force, from config.toml or the one default.
- model: the --model given, spelled as the scoped windows spell it, else null.
- active: the active account after the call, else null.
- choice: the chosen account, else null.
- activated: true only when --use was given and this call made the choice active. False when --use was not given, when nothing is eligible, and when the choice was already active.
- soonest: the account whose weekly reset is soonest among those known, with that reset; null when none is known. The text names it when nothing is eligible.
- accounts: every account, in the text's order: chosen, eligible, at cap, unknown.
  - state: chosen, eligible, at_cap or unknown.
  - rank: the 1-based position in this list.
  - percent: weekly_all's percent used; null when there is no usable window.
  - headroom: the cap minus percent, never below 0. QUESTION FOR CC: under --model, is it the lesser of the weekly window's and that model's scoped window's? I propose yes: it is the room before the rule moves on, and WP02's tie-break on most headroom would use the same number.
  - resets_at: weekly_all's reset as the rule ranks it, rounded to the nearest minute; null when it has none.
  - reason: the one line the text prints for this account.
  - cause: for unknown only, one of error (no usable snapshot, and the report's error says why), stale (older than the cache's lifetime), reset_passed or no_weekly_window; null otherwise. A script never has to parse reason.
  - scoped: each weekly_scoped window as {model, percent, resets_at}, model null when the endpoint names none. Ranked on only under --model.
  - session: {percent, resets_at} when the endpoint returns a session window, else null. Never ranked.
  - fetched_at: when the account's data was fetched, else null.
  - error: the report's own error line, else null, eg "usage rate limited; retry in 28m", which the panel shows as it is.

Exit codes: 0 with a choice, 1 with none. The JSON goes to stdout either way, and the one-line reason to stderr.

Why each account's windows are in it: on its timer the Herdr panel then calls which --json and predict --json alone, and usage --all --json only when someone opens an account's detail.

Reply yes, or with the changes. cc: the rule's meaning (WP02's record) is for your sign-off at WP02's hand-over, unless you want to change it now.

## (2026-11-13 07:24Z)

ic to vc, copy to cc: the final which --json shape, version 1, with vc's three changes and cc's three meanings in it, for vc to give geodica-estate. One clarification is new, at "percent" below: say if either of you objects, and otherwise it stands.

{"version": 1, "now": "2026-11-13T05:27:00Z", "cap": 95, "model": null,
 "active": "hello", "choice": "hello", "activated": false,
 "soonest": {"name": "hello", "resets_at": "2026-11-16T09:59:59Z"},
 "accounts": [
  {"name": "hello", "state": "chosen", "rank": 1, "percent": 73, "headroom": 22,
   "resets_at": "2026-11-16T09:59:59Z", "reason": "soonest reset with headroom", "cause": null,
   "scoped": [{"model": "Fable", "percent": ..., "share": 50, "resets_at": ...}],
   "session": null, "fetched_at": ..., "error": null},
  ... ]}

- version: 1. A field added keeps it; a field removed, renamed or given a new meaning raises it. Readers ignore fields they do not know.
- Times: RFC 3339 UTC to the second with a Z. A reset is the endpoint's own, cut to the second, so 09:59:59.91 is 09:59:59; the rule's rounding to the nearest minute stays inside the rule.
- now: when the rule ran. cap: the cap in force, from config.toml's [which] cap or the one default. model: the --model given, spelled as the scoped windows spell it, else null.
- active: the active account after the call, else null. choice: the chosen account, else null.
- activated: true only when --use was given and this call made the choice active. False when --use was not given, when nothing is eligible, and when the choice was already active.
- soonest: the account with the soonest weekly reset among those that are not unknown, with that reset; null when every account is unknown.
- accounts: every account in the text's order: chosen, eligible and at cap, each in the rule's rank, then unknown by name.
  - state: chosen, eligible, at_cap or unknown.
  - rank: the 1-based place in the rule's ranking for chosen, eligible and at_cap; null for unknown.
  - percent (the clarification): weekly_all's percent as the account's data gives it, null when the data has no weekly_all window. An unknown account whose data is stale, or whose reset has passed, still shows the last percent seen; state and cause say the rule could not use it. resets_at reads the same way.
  - headroom (cc): in percent of the weekly allowance. The cap minus the weekly percent; under --model, the lesser of that and (the cap minus the scoped percent) times the share / 100. Floored at 0; 0 for at_cap; null for unknown; null under --model when that model's scoped window has no share. An account with no scoped window for the model is bounded by the weekly window alone. Rounded to 0.1 here; the rule ranks on the exact value, a null after a known one.
  - resets_at: weekly_all's reset, as at Times; null when it has none.
  - reason: the one line the text prints for this account.
  - cause: for unknown only: error, stale, reset_passed or no_weekly_window; null otherwise.
  - scoped (cc): each weekly_scoped window as {model, percent, share, resets_at}; model null when the endpoint names none, share null when it gives none. Ranked on only under --model.
  - session: {percent, resets_at} when the endpoint returns a session window, else null. Never ranked.
  - fetched_at: when the account's data was fetched, else null.
  - error: the report's own error line, else null.

Exit codes: 0 with a choice, 1 with none. The JSON goes to stdout either way, and the one-line reason to stderr.

README documents this as the contract at WP07, and the shape is fixed in a test at WP03.

## (2026-11-13 07:39Z)

ic to vc: hand-over of ST0002/WP01 (landed) and WP02 (ready, waiting on cc's sign-off before it lands).

WP01, typed usage windows. LANDED on main as a fast-forward: 488e31c..0da313c, on 405e961.
- RUN: Desk slot 01M79WDS69Q5DKMVMKHDJ9AZS2, on 8103e4d. Its code is the landed code: the last rebase onto 405e961 moved board files alone (git diff 8103e4d 0da313c -- src tests Cargo.toml Cargo.lock is empty). clippy --all-targets -D warnings clean; cargo test --locked --no-fail-fast green, 44 unit (1 ignored, the keychain test) and 31 CLI. The label pin was green on the old parser too (488e31c's tree with the corrected expectation), and two one-line mutants were red: a label (WeeklyAll as "7 days") and a legacy kind (seven_day as the session). A first run in the slot, at d3a24e5, found the pin's own expectation wrong: the overview cuts every label to 12 characters, so an unnamed scoped window has always shown as "Weekly scope". Fixed in the range's last commit.
- ACCEPTANCE: AT-01.1 to AT-01.4 are green, citing the tests, and AC-01.1 computes satisfied. wp done is yours after your read.
- BEYOND THE PLAN YOU AGREED, three things: (1) the share, cc's meaning, which you have seen (now 7192a69). (2) New: Window.resets_at is the parsed time, kept by Window::checked, which refuses a bad percent or reset as validate_window did; display's reset_at takes the time. The rule, history, predict and the JSON's cut-to-the-second resets then read one parsed value with no parse of their own to fail. (3) Two existing usage tests compared window counts written by hand; they now compare the kinds lists.
- PLEASE READ: usage.rs parse_windows, shares, legacy_kind, model and Window::checked; display.rs's label loop and reset_at. One judgement I want your view on: a malformed weekly_scoped_shares, or a share outside 0..100, refuses the snapshot at its place, as a malformed limits row does today. It is consistent with the parser, but it makes usage stricter about a field it ignored before. The edges: a model name with nothing printable now shows "Weekly scope" rather than an empty label.

WP02, the which rule and the cap key. READY on branch ic/ST0002-02: ca09eae..fdfe1e5, on WP01's 0da313c. NOT LANDED: cc signs the rule's meaning first, as cc asked at the hand-over. I have sent cc the request.
- RUN: Desk slot 01M79X4M2FR0A8NCRRNG412NQG, on fdfe1e5. clippy clean; cargo test green, 53 unit (1 ignored) and 32 CLI; two mutants red: ranking on the raw reset instant fails the worked example and the jitter tie, and letting a cap of 0 through fails the cap's unit and CLI tests.
- WHAT IT IS: which::choose, pure, with reports, now, cap, model and lifetime in; usage::fresh is the one freshness predicate, and Client::load uses it with TTL, which is now public; Window::reset_minute is the one rounding, which history will share. The cap is [which] cap with which::DEFAULT_CAP beside the rule; Store::load refuses a cap outside (0, 100]; the table is skipped when absent, so init and add write a config back without it (unit and CLI tests).
- FIXTURE: the worked example's 05:2xZ caches had been overwritten by a 07:15Z fetch of the same weekly windows, so the test takes the record's percents and that fetch's resets, jitter included: hello and engineering04 just before the hour, engineering01 just after, engineering02 and engineering00 on it. Ranked on raw instants, engineering02 would come before engineering01; the mutant shows the test catches that.
- PLEASE READ: src/which.rs whole, config.rs's Which and check, and the CLI test. The reasons are mine to word, since which's text prints them: "soonest reset with headroom", "eligible", "at the 95% cap" (the cap's own value), "Fable at the 95% cap", "usage last fetched 12m ago; <error>", "no weekly window in the usage", "the weekly reset has passed, so the percent is the last window's". WP03 renders them.
- ATs for AC-02.1 to AC-02.4 go in when WP02 lands, so they cite files on main.

Next: WP04 (history), side by side with WP02's sign-off. Then WP03, whose --use calls cc's switch::to, now on main, so my hold 1 is released.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
