---
st_id: ST0002
title: Add 'which' verb to indicate the best account to use to maximise token usage across accounts
---

# ST0002: Add 'which' verb to indicate the best account to use to maximise token usage across accounts -- Acceptance

> **THIS FILE IS A GENERATED VIEW, AND A ROW AUTHORED HERE IS DISCARDED BY THE NEXT SYNC.** The acceptance contract is canon in the thread model; this file renders it. Acceptance Criteria (AC) are the ratified completeness boundary; Acceptance Tests (AT) are the small red-to-green tests that prove them.
>
> Done = every AC is covered by a GREEN AT, or (for a non-test AC) its named evidence is satisfied, AND the AC set is the ratified full boundary. Done is read from this map, never from a hand-ticked box.
>
> Test-backed satisfaction is COMPUTED from covering green ATs and never stored -- storing it would be double truth. An AC has four states, not two: beyond satisfied and unsatisfied, a requirement can be **descoped** to a named thread or **withdrawn** with its reason on the record. Both are non-blocking and both are reported separately, so a thread that descoped half its contract looks like one.

## Acceptance Criteria

### ST-level

- AC-00.1 (non-test) On hv's five accounts, hv's own `ccusage which` names the account the rule picks from `ccusage --all`, and `ccusage predict` runs without error. -- satisfied: no

### WP-01 -- Typed usage windows: each window carries its kind and model (status: Not Started)

- AC-01.1 Every parsed usage window carries its kind, and its model when scoped, read from the `limits` array and the legacy fields alike, and the output of `usage` and of the TUI is unchanged. -- satisfied: no (computed)

### WP-02 -- The which rule: earliest reset first, with the cap from config (status: Not Started)

- AC-02.1 On the worked example's windows at 2026-11-13T05:27Z, with the caches' own reset times, the rule chooses hello and ranks engineering04, then engineering01 and engineering02, then engineering00. -- satisfied: no (computed)
- AC-02.2 An account whose data is older than the usage cache's lifetime, whose weekly reset has passed, or that has no weekly window is unknown: listed with its reason and never chosen. -- satisfied: no (computed)
- AC-02.3 The cap comes from config.toml, with one default when the key is absent; a config that does not set the key is written back without it; and no test writes the cap as a literal. -- satisfied: no (computed)
- AC-02.4 With a model named, an account is eligible only while its scoped window for that model is below the cap too. -- satisfied: no (computed)

### WP-03 -- The which verb: text, versioned JSON and --use (status: Not Started)

- AC-03.1 `ccusage which` prints the choice with one reason line, then the rest in rank order one line each, and exits 0; with nothing eligible it says so on stdout, names the soonest reset and exits 1. -- satisfied: no (computed)
- AC-03.2 `ccusage which --json` emits the versioned shape cc and ic agreed, with the same choice and order as the text. -- satisfied: no (computed)
- AC-03.3 `ccusage use`, the TUI's `a` key and `ccusage which --use` make an account active through one function, and `which --use` changes nothing when nothing is eligible. -- satisfied: no (computed)
- AC-03.4 `ccusage which --model NAME` with a name no account's scoped window carries is refused, naming the models seen. -- satisfied: no (computed)

### WP-04 -- Usage history: bounded samples under XDG_STATE_HOME (status: Not Started)

- AC-04.1 Each successful fetch records a sample for each window whose percent changed, in a 0600 file under XDG_STATE_HOME; a cache hit records nothing; and samples are dropped by age, one weekly window, not at a reset. -- satisfied: no (computed)

### WP-05 -- The predict verb: burn rate and projected loss (status: Not Started)

- AC-05.1 `ccusage predict` reports each account's projected unused percent and the total projected loss from a fixture history, says how much history it rests on, and says plainly when there is too little. -- satisfied: no (computed)

### WP-06 -- The TUI marks the account which would choose (status: Not Started)

- AC-06.1 The TUI overview marks the account `which` chooses once every account's fresh report is in, and never on partial or stale data. -- satisfied: no (computed)

### WP-07 -- Docs: README and CHANGELOG for which and predict (status: Not Started)

- AC-07.1 (non-test) README and the CHANGELOG describe `which`, `predict`, the cap key, the history file and the JSON contract as built. -- satisfied: no

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Typed usage windows: each window carries its kind and model (status: Not Started)

_(no tests in this group)_

### WP-02 -- The which rule: earliest reset first, with the cap from config (status: Not Started)

_(no tests in this group)_

### WP-03 -- The which verb: text, versioned JSON and --use (status: Not Started)

_(no tests in this group)_

### WP-04 -- Usage history: bounded samples under XDG_STATE_HOME (status: Not Started)

_(no tests in this group)_

### WP-05 -- The predict verb: burn rate and projected loss (status: Not Started)

_(no tests in this group)_

### WP-06 -- The TUI marks the account which would choose (status: Not Started)

_(no tests in this group)_

### WP-07 -- Docs: README and CHANGELOG for which and predict (status: Not Started)

_(no tests in this group)_

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
