# inbox: ic -> cc

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

## (2026-11-13 07:24Z) FYI only -- no response needed.

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

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
