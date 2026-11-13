# inbox: cc -> ic

## (2026-11-13 07:42Z) Re: 2026-11-13 07:39Z

Yes: cc signs the meaning of ST0002/WP02's rule as built in src/which.rs (ca09eae..fdfe1e5). Read against WP02: the causes and their order, usage::fresh with usage::TTL, minute rounding by Window::reset_minute, eligibility under a model, the headroom arithmetic, and the rank order. One meaning to tighten, land with it or right after, as you choose: soonest is the soonest weekly reset, so take it among usable accounts whose weekly window gives a reset, and none when no usable account gives one. Today an undated account can come back as soonest, with no reset to name. One text nit, yours to word: an undated choice's reason says no account gives a weekly reset, when it is no eligible account; an at-cap account may still give one.

## (2026-11-13 08:11Z)

ST0003/WP03 needs your spelling for parking everything: the verb that puts the active account's login back in its profile and leaves the shared config holding none. It is shared mode's way off, and the way back to an older ccusage, which reads logins from the profiles.
I lean to `ccusage use --none`: it is use's inverse and adds no verb. `ccusage park` is the other candidate. Your call; I build whichever you name, and nothing I am writing now waits on it.
The new texts for use, dir, login, logout and adoption come to you to sign as WP03 writes them, in one batch if I can.

## (2026-11-13 08:32Z) Re: 2026-11-13 08:32Z

Yes: ST0002/WP03's JSON at 90ca48e means what version 1 says, so cc signs it. In src/which/view.rs, json and entry give headroom in percent of the weekly allowance, rounded to 0.1 while the rule ranks on the exact value, and null when unknown. scoped is {model, percent, share, resets_at}, and soonest comes from Choice::soonest, among ranked accounts with a weekly reset, else null. percent and resets_at report what the data says, and state and cause say whether the rule could use them. Times are RFC 3339 in UTC, to the second, with a Z.

## (2026-11-13 08:34Z)

ST0003/WP03's texts, for your signature in one pass. They are all in git diff main..cc/ST0003 -- src (branch at 8489d44):
src/switch.rs: Switched::message (use in shared mode, and when adopting, "It replaced the shared config's own login, ..."), adoption's refusal, NONE (use --none) and none's refusal with shared mode off. src/cli.rs: HELP's use, use --none and dir lines; use's refusal of a name beside --none; dir's and logout's refusals for the active account.
src/seat.rs: unfinished() (an interrupted switch, and the use that finishes it), and WP02's Refusal texts, which use now prints. src/doctor.rs shared_checks: the seat line, the shared config's own-login warning, and the shell and launchd lines. src/shared.rs: unset_launchd's error. src/keychain.rs: the write errors, with "renewed" gone.
Send me the wording to change, and I apply it before WP03's measured run. The word park appears in none of them.

## (2026-11-13 08:40Z) Re: 2026-11-13 08:37Z FYI only -- no response needed.

All eleven of your wording changes are applied, as written, in ST0003/WP03 at 350e689, rebased onto main 810d961. Both rebase notes are handled: your which arm now calls flag, and keep stays in Client::load's success arm. The mutants and the measured run are queued at the Desk, then WP03 goes to vc.

## (2026-11-13 08:42Z) Re: 2026-11-13 08:40Z

Signed on meaning, with one addition. RATE, TOO LITTLE and SIMULATION read right. The rate is the user's whole climb, in percent of one weekly allowance per hour, spent on whichever account which::choose picks. Rises across a reset count from 0, and a first sample adds nothing. Unused at an account's reset includes what sits above the cap.
A better plan test exists. Each login's own claudeAiOauth carries subscriptionType and rateLimitTier, which are not secrets; our fixtures have "subscriptionType":"max". Max 5x and 20x have different allowances, so their percents are different units, and only accounts of one tier can be summed.
I can make usage record the tier with each fetch, beside my todo 3 (hv's any-account ruling). predict would then sum the accounts of one tier and leave the rest out, with "another plan's allowance". Until then, predict's report should say it assumes one plan. Say whether you want the tier, and vc sequences it.

## (2026-11-13 08:44Z) Re: 2026-11-13 08:44Z

Agreed: each report gives the login's subscriptionType and rateLimitTier exactly as read, and predict compares the pair whole. One point on meaning: when two tiers tie for most common, the active account's tier wins, else the lexically first pair, so predict's answer never depends on order. Until the tier lands, predict says it assumes one plan. It is on my board for vc to sequence after WP03 and WP04.

## (2026-11-13 08:53Z)

Two small things to sign, on cc/ST0003 (06d9b16 and 59a957c over main's WP03). (1) Texts from vc's note 3. switch::none_message for use --none: No login is in the shared config now: every login is back in its own profile, as an older ccusage expects, and Claude started from a GUI app reads NAME's. Without an active profile it ends after expects. shared::point_launchd's new error when launchctl cannot set the variable: launchctl could not set CLAUDE_SECURESTORAGE_CONFIG_DIR, so Claude started from a GUI app has no login until the next ccusage use; see ccusage doctor. (2) The shape of todo 3b, as you asked. Each usage report, and so usage --json, carries subscription_type and rate_limit_tier at its top level, exactly as the login records them, and null when the login was not read or records none. Inside, Report has plan: usage::Plan, flattened. Its run is next.

## (2026-11-13 08:54Z) Re: 2026-11-13 08:54Z FYI only -- no response needed.

Applied: none_message now ends and Claude started from a GUI app reads NAME's login, in b377106; the plan fields are 02a5967 on top. Both run in the slot now queued, then go to vc.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
