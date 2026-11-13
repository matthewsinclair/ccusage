---
st_id: ST0002
title: Add 'which' verb to indicate the best account to use to maximise token usage across accounts
status: WIP
created: 2026-11-13
completed:
---

# ST0002: Add 'which' verb to indicate the best account to use to maximise token usage across accounts

## Objective

Give hv one command that names the account to work on now, so that no weekly allowance goes unused at its reset, and a companion that projects how much allowance will be lost if hv follows it. `ccusage which` applies earliest reset first, "use it or lose it", to every account's weekly window. `ccusage predict` does arithmetic on the usage samples ccusage records from its own fetches. Both read what `usage --all` reads, through the same five-minute cache, and "No model inference is made" stays true.

## Context

**Source.** geodica-estate's brief of 2026-11-13, written on hv's ask and sent to vc. vc reviewed and corrected this elaboration the same day, on hv's ask. hv's word on it, 2026-11-13 in the live channel: all five accounts are on one plan, and engineering04 returns to the free plan, which leaves four; one cut carries everything; hv starts ic and cc once the facts of ST0003/WP01's trial are in; and a Herdr mod is added, WP08. No seat writes code before that trial's facts are in.

**The problem.** hv runs five accounts: engineering00, engineering01, engineering02, engineering04 and hello. Each has a weekly window (`weekly_all`) with its own `resets_at`, and a model-scoped window (`weekly_scoped`; Fable today, with an allowance of 50% of the weekly one). Allowance still unused at a reset is lost. Today hv reads `ccusage --all` and decides by eye.

**The rule.** Work on the account with headroom whose reset is soonest. Accounts with distant resets are the reserve: move to one only when everything resetting sooner is at or near its cap.

**Worked example.** From the caches fetched between 05:23Z and 05:27Z on 2026-11-13 (the brief rounded these times): hello at 73% resets 2026-11-16 09:59Z; engineering04 at 22% resets 2026-11-16 12:00Z; engineering01 and engineering02 at 0% reset 2026-11-19 21:00Z; engineering00 at 0% resets 2026-11-20 04:00Z. The answer is hello until it nears its cap, then engineering04, leaving 00, 01 and 02 until after Monday's resets. WP02 makes this a fixture test, and it stays one after engineering04 leaves the plan.

**What the code gives the work packages.**

- `usage::Snapshot::windows()` returns (label, Window) pairs. The `limits` kinds become display labels ("7 day", or the model's display name), so the kind `which` ranks on is gone before anything can select by it. The legacy fields (`seven_day`, `seven_day_<model>`, `five_hour`) are mapped the same way.
- `usage::Client::report` gives each account a `Report`: the last good snapshot, when it was fetched, whether it came from the cache, and an error when the refresh failed or was rate limited. A report can carry an old snapshot and an error at once, and a fresh snapshot with an error when only the cache save failed. So the age of a report's data, not whether it carries an error, says whether the rule may use it. The cache lives 300 s, and a 429 backs off for 1800 s.
- The TUI fetches at open and on `r`, never by itself, through the same `Client::report`. So a hook in the fetch path covers it, and what it shows can be older than the cache's lifetime. WP08 gives it the same refresh policy as the Herdr panel.
- `config::Config` has `deny_unknown_fields`, so every older binary refuses a config carrying a new key, and `ccusage add` rewrites the whole config. A key written back unasked would lock a brew-installed ccusage out of a config hv never changed, and a launcher's `ccusage dir` would then fail without a word.
- `use NAME` lives inline in `cli::run`, and the TUI's `a` key is a second copy of it that writes `active.json` alone and skips shared mode's switch. One function replaces both. Changing hv's real active account is hv's own command (dc's watch-out 1).
- Today the endpoint returns `weekly_all` and a Fable `weekly_scoped` window for all five accounts, and no session window. Reset times jitter by under a second either side of the hour: hello's is 09:59:59.53Z and engineering01's 21:00:00.39Z. Ranking on raw timestamps would let the jitter decide, and cutting them to the minute would put one reset in two different minutes, so the rule compares resets rounded to the nearest minute.

**Departures from the brief.**

- History is bounded by age, one weekly window, not by reset. Dropping a window's samples at its reset erases what hv burned on it while the other accounts' older samples still stretch the span, so the burn rate would read low after every reset. hv has read this and not objected.
- The burn rate `predict` projects with is percent per elapsed hour over that history, idle hours included. The brief's rate per hour of actual use needs a guess at hv's working hours before it can project days ahead, and sparse samples cannot tell working hours from idle ones. hv has read this and not objected.
- `predict` adds percents across accounts, which holds because the accounts are on one plan (hv, 2026-11-13). A profile left on another plan, as engineering04 would be on the free one, is outside that arithmetic until hv removes it.

**Not in scope.** The 5-hour session window: the parser already reads one if the endpoint returns it, and `which` notes it but never ranks on it. Usage credits and extra usage. Switching accounts on a timer. A longer cache lifetime for accounts nothing is running on.

**Consumers.** hv at the shell. The TUI, with a "next" marker on the chosen account. The Herdr panel (WP08), which draws from the same JSON. Later, a status line. geodica-estate will call `ccusage which --json` at its folds, so that JSON shape is a contract: cc is authoritative on what it means and ic on its shape, and a change to it needs both.

**With ST0003.** `which --use` calls the one `use` function. Once ST0003 lands, that switches every running session, so reaching an account's limit costs one command and a cap nearer 100 wastes less. The default stays 95 until hv says otherwise.

**Seats.** ic builds this thread, and cc signs the rule, predict's arithmetic and what the JSON means. cc builds ST0003. dc wires ISSUE:0007, briefed on 2026-11-13, and runs the cut. hv starts ic and cc once the facts of ST0003/WP01's trial are in.

**Release.** hv, 2026-11-13: "one cut, everything in". It carries this thread, ST0003 (whose rollout closes ISSUE:0003) and ISSUE:0007.

**Order.** WP01 first. Then WP02 and WP04 side by side. WP03 and WP06 follow WP02; WP05 follows WP02 and WP04; WP08 follows WP03, WP05 and WP06; WP07 comes last. vc validates each work package against its acceptance criteria on its builder's one measured run.

## Work Packages

| WP    | Title                                                                                | Size | Status      |
| ----- | ------------------------------------------------------------------------------------ | ---- | ----------- |
| WP-01 | Typed usage windows: each window carries its kind and model                          | S    | Done        |
| WP-02 | The which rule: earliest reset first, with the cap from config                       | M    | WIP         |
| WP-03 | The which verb: text, versioned JSON and --use                                       | M    | Not Started |
| WP-04 | Usage history: bounded samples under XDG_STATE_HOME                                  | M    | WIP         |
| WP-05 | The predict verb: burn rate and projected loss                                       | M    | Not Started |
| WP-06 | The TUI marks the account which would choose                                         | S    | Not Started |
| WP-07 | Docs: README and CHANGELOG for which and predict                                     | S    | Not Started |
| WP-08 | Herdr mod: a ccusage panel in a Herdr pane, its install verb, and one refresh policy | M    | Not Started |

## Acceptance

Acceptance Criteria and Acceptance Tests are RENDERED into `acceptance.md`, which is a GENERATED VIEW -- a row authored there is discarded by the next sync. The contract is canon in this thread's model, and the verbs write it: `intent ac new` and `intent at new` mint a row, `intent ac edit` and `intent at edit` reword or re-cite one, and `intent ac satisfy|unsatisfy|descope|rescope|withdraw|reinstate` and `intent at green|red|na` move its state. This cover never restates them.

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
