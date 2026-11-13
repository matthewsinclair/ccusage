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

**Source.** geodica-estate's brief of 2026-11-13, written on hv's ask and sent to vc. Its questions are hv's to answer. hv reviews this thread and its work packages before anyone writes code (hv to vc, 2026-11-13, in the live channel).

**The problem.** hv runs five accounts: engineering00, engineering01, engineering02, engineering04 and hello. Each has a weekly window (`weekly_all`) with its own `resets_at`, and a model-scoped window (`weekly_scoped`; Fable today, with an allowance of 50% of the weekly one). Allowance still unused at a reset is lost. Today hv reads `ccusage --all` and decides by eye.

**The rule.** Work on the account with headroom whose reset is soonest. Accounts with distant resets are the reserve: move to one only when everything resetting sooner is at or near its cap.

**Worked example.** From the caches fetched between 05:23Z and 05:27Z on 2026-11-13 (the brief rounded these times): hello at 73% resets 2026-11-16 09:59Z; engineering04 at 22% resets 2026-11-16 12:00Z; engineering01 and engineering02 at 0% reset 2026-11-19 21:00Z; engineering00 at 0% resets 2026-11-20 04:00Z. The answer is hello until it nears its cap, then engineering04, leaving 00, 01 and 02 until after Monday's resets. WP02 makes this a fixture test.

**What the code gives the work packages.**

- `usage::Snapshot::windows()` returns (label, Window) pairs. The `limits` kinds become display labels ("7 day", or the model's display name), so the kind `which` ranks on is gone before anything can select by it. The legacy fields (`seven_day`, `seven_day_<model>`, `five_hour`) are mapped the same way.
- `usage::Client::report` gives each account a `Report`: the last good snapshot, when it was fetched, whether it came from the cache, and an error when the refresh failed or was rate limited. A report can carry an old snapshot and an error at once. The cache lives 300 s, and a 429 backs off for 1800 s.
- The TUI's worker calls the same `Client::report`, so whatever hooks the fetch path covers the TUI too.
- `config::Config` has `deny_unknown_fields`, so every older binary refuses a config carrying a new key. Once hv sets the cap key, a brew-installed ccusage older than the release refuses the config until it is upgraded.
- `use NAME` lives inline in `cli::run`, and in shared mode it rewrites the shared config's account record through `shared::switch`. Changing hv's real active account is hv's own command (dc's watch-out 1).
- Today the endpoint returns `weekly_all` and a Fable `weekly_scoped` window for all five accounts, and no session window. Reset times carry sub-second jitter: engineering01 resets at 21:00:00.388Z and engineering02 at 21:00:00Z, so ranking on raw timestamps would let the jitter decide and never reach the headroom tie-break.

**Not in scope.** The 5-hour session window: the parser already reads one if the endpoint returns it, and `which` notes it but never ranks on it. Usage credits and extra usage. Switching accounts on a timer.

**Consumers.** hv at the shell. The TUI, with a "next" marker on the chosen account. Later, a status line. geodica-estate will call `ccusage which --json` at its folds, so that JSON shape is a contract: cc is authoritative on what it means and ic on its shape, and a change to it needs both.

**Seats.** Each work package names its proposed builder by remit: cc for the windows, the rule, the history and predict's arithmetic; ic for the verbs, the JSON shape and the TUI; ac for the docs. dc stays on 0.1.9's blockers and runs the cut. hv decides which seats are started.

**Release.** ST0002 joins 0.1.9's one cut with ISSUE:0007 and ISSUE:0003, pending hv's word.

**Order.** WP01 first. Then WP02 and WP04 side by side. WP03 and WP06 follow WP02; WP05 follows WP02 and WP04; WP07 comes last. vc validates each work package against its acceptance criteria on its builder's one measured run.

## Work Packages

| WP    | Title                                                          | Size | Status      |
| ----- | -------------------------------------------------------------- | ---- | ----------- |
| WP-01 | Typed usage windows: each window carries its kind and model    | S    | Not Started |
| WP-02 | The which rule: earliest reset first, with the cap from config | M    | Not Started |
| WP-03 | The which verb: text, versioned JSON and --use                 | M    | Not Started |
| WP-04 | Usage history: bounded samples under XDG_STATE_HOME            | M    | Not Started |
| WP-05 | The predict verb: burn rate and projected loss                 | M    | Not Started |
| WP-06 | The TUI marks the account which would choose                   | S    | Not Started |
| WP-07 | Docs: README and CHANGELOG for which and predict               | S    | Not Started |

## Acceptance

Acceptance Criteria and Acceptance Tests are RENDERED into `acceptance.md`, which is a GENERATED VIEW -- a row authored there is discarded by the next sync. The contract is canon in this thread's model, and the verbs write it: `intent ac new` and `intent at new` mint a row, `intent ac edit` and `intent at edit` reword or re-cite one, and `intent ac satisfy|unsatisfy|descope|rescope|withdraw|reinstate` and `intent at green|red|na` move its state. This cover never restates them.

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
