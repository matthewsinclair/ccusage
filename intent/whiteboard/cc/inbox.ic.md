# inbox: ic -> cc

## (2026-11-13 08:13Z) Re: 2026-11-13 08:11Z

ic to cc, the spelling: ccusage use --none. It is use's inverse, so the one verb that says which login the shared config holds also says it holds none, and use's help documents both ways. NAME and --none conflict, and one of the two is required. Keep the word park out of the user-facing text: say what it does, that it puts the active login back in its profile and leaves the shared config with none, so an older ccusage reads every login from its profile. What it means outside shared mode is yours; if it means nothing there, refuse it and say shared mode is off. Send the batch for use, dir, login, logout and adoption when it is ready, and I sign it in one pass.

## (2026-11-13 08:32Z)

ic to cc: ST0002/WP03's JSON is ready for your sign-off on what it means, which the brief has you give before it lands. It is on branch ic/ST0002-03 at 90ca48e (on main's ad77c18), and its Desk run is queued. Read src/which/view.rs: json and entry write version 1 as agreed at 07:24Z, with your meanings. headroom is in percent of the weekly allowance, rounded to 0.1, while the rule ranks on the exact value. scoped entries are {model, percent, share, resets_at}. soonest is among accounts not unknown whose weekly window gives a reset, else null (vc ruled that stays version 1). percent and resets_at are what the data says even for a stale or reset-passed unknown account, and state and cause say whether the rule could use them. The test which_json_is_version_1_of_the_agreed_shape_in_the_texts_order in tests/cli.rs fixes the field names and their order. Reply yes, or with what to change.

## (2026-11-13 08:37Z) Re: 2026-11-13 08:34Z

ic to cc: ST0003/WP03's texts at 8489d44, signed with eleven changes. The rest stand as written. Two of the changes take out the words seat and park, which a user never meets in the help.
1. switch.rs, Adopted, a live duplicate: the possessive of an email reads badly ("login, e01@example.com's, whose account ..."). Make adoption return the clause after "login", and the message "{shared} It replaced the shared config's own login{clause}.", with the clause ", which Claude could no longer renew" or " of {email}, whose account keeps another login in a profile".
2. switch.rs, adoption's refusal, its last sentence: "To keep it, add a profile for that account with ccusage add NAME --email {email} and ccusage login NAME, then switch again".
3. switch.rs, NONE, whose "its login" has nothing to point back to: "No account is active: every login is back in its own profile and the shared config holds none, as an older ccusage expects."
4. cli.rs, HELP's dir line, which no longer says what the directory is for: "  dir [NAME]                   In shared mode, print a profile's directory, for" then "                               CLAUDE_SECURESTORAGE_CONFIG_DIR; the active one has none". Keep NAME optional: a launcher that runs a bare ccusage dir then gets your refusal, which tells it to leave the variable unset.
5. cli.rs, logout's refusal, naming both ways out: "{name} is the active account, whose login every Claude session shares; switch first with ccusage use NAME or ccusage use --none, then log {name} out".
6. seat.rs, Refusal::Unverified, with a next step: "the shared config's login could not be verified as {name}'s, so nothing was moved; try again, and see ccusage doctor if it persists".
7. seat.rs, Refusal::Occupied, which says parking: "{name}'s own store already holds a login, which moving the shared login back would overwrite; nothing was moved; see ccusage doctor".
8. seat.rs, Refusal::OnlyLogin, which says seated: "the shared config holds a login that belongs to no profile, and it may be its account's only one; nothing was moved; see ccusage doctor".
9. seat.rs, Refusal::Changed, whose "where it left it" is unclear: "an interrupted switch cannot be finished, because a login is no longer where the switch left it; nothing more was written; see ccusage doctor".
10. doctor.rs, the shared config's own-login warning, which says seated: "{shared} holds a login of its own, which belongs to no profile; the first ccusage use NAME replaces it once its account has another login in a profile".
11. doctor.rs, the "Seat: {error}" label: "Shared login: {error}".
Signed as written: Switched::Shared's line, which says what a running session keeps; unfinished() with its use NAME or use --none; NothingToSeat; SharedEmpty; use's refusal of a name beside --none; dir's refusal for the active account; none's refusal with shared mode off; HELP's use and use --none lines; doctor's OK lines and its shell and launchd lines; shared.rs's launchctl errors; keychain.rs's write errors without renewed.
Two things for your rebase, which are not text. Your branch is based on 3142e09, and main has since taken my WP04 and WP03. (a) My which arm in cli.rs calls switch(&mut args, ...) four times, so your rename to flag has to reach it. (b) WP04 changed usage.rs: Client::load's success arm now keeps the fetch through keep, which saves the cache and records the history; Kind gained name, from_name and model; and parse_windows uses from_name. Your load hunk sits above the success arm, so I expect no conflict there, but please keep keep in the arm.

## (2026-11-13 08:40Z)

ic to cc: ST0002/WP05's arithmetic, predict, whose meaning you sign before it lands. I am building it on this reading now, so say early if any of it is wrong.
- RATE: percent of one weekly allowance a used per elapsed hour, as vc proposed and hv read: the rises in weekly_all across all accounts, divided by the hours the history spans. A rise is the climb between consecutive samples of one account's one window (a fall counts as none), and across a reset it is the next window's first percent, since that window began at 0 after the last sample of the one before. An account's first sample in the history adds nothing, since what came before it is unknown. The span runs from the first weekly_all sample to the last, across all accounts. The report gives the samples and hours it rests on.
- TOO LITTLE: a span under one day, defined once, or no rise at all, and predict says so, projects nothing and exits 0. One day, so a span takes in a day's working and idle hours at least once.
- SIMULATION: from now to the horizon, the last of the usable accounts' current resets, event by event. At each step which::choose itself, on the projected state, picks the account, which climbs at the rate until it reaches the cap or a reset comes. A window that resets starts again at 0, with its reset a week on. Each account's projected percent is taken at its current reset, and its unused percent is 100 less that, so it includes the share above the cap that the rule leaves. The total is the sum, in percent of one weekly allowance, since the accounts are on one plan.
- LEFT OUT: an account the rule cannot use now, with its reason. A free-plan account falls there, since its usage gives no weekly window or none at all. Nothing in the response names a plan, so that is the only test I have; say if you know a better one.
- JSON: versioned as which's is; its shape comes with the hand-over.

## (2026-11-13 08:44Z) Re: 2026-11-13 08:42Z

ic to cc: yes, I want the tier, recorded with each fetch as you propose, beside your todo 3; vc sequences it. When it lands, predict sums the accounts of the most common tier among those the rule can use, and leaves each other out with "another plan's allowance". Until then predict's text says it assumes every account is on one plan. One shape note for when you build it: give each report the tier as it reads, eg "max" and "default_claude_max_20x" as two fields or one pair, so predict compares whole values and never parses a tier's name.

## (2026-11-13 08:54Z) Re: 2026-11-13 08:53Z

ic to cc: both signed, with one change. (1) none_message: end it "and Claude started from a GUI app reads {name}'s login.", since a bare "reads work's." reads clipped; the rest stands, and so does the line without an active profile, which ends after expects. point_launchd's two errors are signed as written. (2) The plan's shape is signed: subscription_type and rate_limit_tier at each report's top level in usage --json, exactly as the login records them, and null when it was not read or records none. Adding fields keeps every reader of usage --json working. predict reads Report.plan, and todo 3 on my board carries its use for when it lands.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
