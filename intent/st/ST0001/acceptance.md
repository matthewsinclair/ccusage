---
st_id: ST0001
title: Bring Ccusage up to Intent spec
---

# ST0001: Bring Ccusage up to Intent spec -- Acceptance

> **THIS FILE IS A GENERATED VIEW, AND A ROW AUTHORED HERE IS DISCARDED BY THE NEXT SYNC.** The acceptance contract is canon in the thread model; this file renders it. Acceptance Criteria (AC) are the ratified completeness boundary; Acceptance Tests (AT) are the small red-to-green tests that prove them.
>
> Done = every AC is covered by a GREEN AT, or (for a non-test AC) its named evidence is satisfied, AND the AC set is the ratified full boundary. Done is read from this map, never from a hand-ticked box.
>
> Test-backed satisfaction is COMPUTED from covering green ATs and never stored -- storing it would be double truth. An AC has four states, not two: beyond satisfied and unsatisfied, a requirement can be **descoped** to a named thread or **withdrawn** with its reason on the record. Both are non-blocking and both are reported separately, so a thread that descoped half its contract looks like one.

## Acceptance Criteria

### ST-level

- AC-00.1 (non-test) intent doctor reports no finding and no advisory, the stale-backup advisory included -- satisfied: no

### WP-01 -- Declare languages, install Intent's Claude canon and git gate, resync AGENTS.md (status: WIP)

- AC-01.1 (non-test) intent lang declared lists rust and shell, so /in-session and the critics apply the Rust and shell rules -- evidence: vc validated: intent lang declared prints rust and shell (commit 80179a5) -- satisfied: yes
- AC-01.2 (non-test) intent claude upgrade --apply has written the Claude canon and git hooks, and the pre-commit gate ran the critics and intent doctor on a real commit -- evidence: vc validated: canon and hooks from aaa26d0; intent claude upgrade dry run reports 14 canonical, 0 to write; gate ran critics (rust, shell) and doctor on aaa26d0, 9277f18 and vc's 06dd691 -- satisfied: yes
- AC-01.3 (non-test) AGENTS.md is resynced from project state with intent agents sync; usage-rules.md, the project's own contract, names this project's real checks by pointing at README's Development section and docs/releasing.md, and says that AGENTS.md's Bats and unlocked cargo lines are Intent's generic language text, not this project's commands; README no longer says Intent integration is deferred -- evidence: vc validated: usage-rules.md points at README's Development section and docs/releasing.md and names AGENTS.md's Bats and unlocked cargo lines as Intent's generic text (41eb982); README's deferred line gone and AGENTS.md resynced, untouched since (80179a5) -- satisfied: yes

### WP-02 -- Declare the whiteboard roster and seed the project records (status: Done)

- AC-02.1 (non-test) intent/whiteboard/README.md declares hv, vc and dc with their scopes and names vc as the reader of hv/inbox.*, and hv is registered -- evidence: intent/whiteboard/README.md and hv's registration, commit b79aad5 -- satisfied: yes
- AC-02.2 (non-test) intent/wip.md carries the current focus with a verblock, not the template -- evidence: intent/wip.md, first written in commit b79aad5 -- satisfied: yes

### WP-03 -- Record the Keychain access decision in ARCHITECTURE.md and RULES.md (status: Not Started)

- AC-03.1 (non-test) intent/llm/ARCHITECTURE.md records why ccusage reaches Claude's Keychain items only through /usr/bin/security, and intent/llm/RULES.md carries the matching never-rule -- satisfied: no

### Group AC-01

- AC-01 (non-test) intent lang declared lists rust and shell, so /in-session and the critics apply the Rust and shell rules (WP01) -- WITHDRAWN: renumbered to the AC-<wp>.<n> scheme: doctor reads AC-<gg> as WP <gg>, so AC-04 to AC-07 named work packages ST0001 lacks (by vc)

### Group AC-02

- AC-02 (non-test) intent claude upgrade --apply has written the Claude canon and git hooks, and the pre-commit gate ran the critics and intent doctor on a real commit (WP01) -- WITHDRAWN: renumbered to the AC-<wp>.<n> scheme: doctor reads AC-<gg> as WP <gg>, so AC-04 to AC-07 named work packages ST0001 lacks (by vc)

### Group AC-03

- AC-03 (non-test) AGENTS.md is resynced from project state, and README no longer says Intent integration is deferred (WP01) -- WITHDRAWN: renumbered to the AC-<wp>.<n> scheme: doctor reads AC-<gg> as WP <gg>, so AC-04 to AC-07 named work packages ST0001 lacks (by vc)

### Group AC-04

- AC-04 (non-test) intent/whiteboard/README.md declares hv, vc and dc with their scopes and names vc as the reader of hv/inbox.*, and hv is registered (WP02) -- WITHDRAWN: renumbered to the AC-<wp>.<n> scheme: doctor reads AC-<gg> as WP <gg>, so AC-04 to AC-07 named work packages ST0001 lacks (by vc)

### Group AC-05

- AC-05 (non-test) intent/wip.md carries the current focus with a verblock, not the template (WP02) -- WITHDRAWN: renumbered to the AC-<wp>.<n> scheme: doctor reads AC-<gg> as WP <gg>, so AC-04 to AC-07 named work packages ST0001 lacks (by vc)

### Group AC-06

- AC-06 (non-test) intent/llm/ARCHITECTURE.md records why ccusage reaches Claude's Keychain items only through /usr/bin/security, and intent/llm/RULES.md carries the matching never-rule (WP03) -- WITHDRAWN: renumbered to the AC-<wp>.<n> scheme: doctor reads AC-<gg> as WP <gg>, so AC-04 to AC-07 named work packages ST0001 lacks (by vc)

### Group AC-07

- AC-07 (non-test) intent doctor reports no finding and no advisory, the stale-backup advisory included -- WITHDRAWN: renumbered to the AC-<wp>.<n> scheme: doctor reads AC-<gg> as WP <gg>, so AC-04 to AC-07 named work packages ST0001 lacks (by vc)

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Declare languages, install Intent's Claude canon and git gate, resync AGENTS.md (status: WIP)

_(no tests in this group)_

### WP-02 -- Declare the whiteboard roster and seed the project records (status: Done)

_(no tests in this group)_

### WP-03 -- Record the Keychain access decision in ARCHITECTURE.md and RULES.md (status: Not Started)

_(no tests in this group)_

### Group AC-01

_(no tests in this group)_

### Group AC-02

_(no tests in this group)_

### Group AC-03

_(no tests in this group)_

### Group AC-04

_(no tests in this group)_

### Group AC-05

_(no tests in this group)_

### Group AC-06

_(no tests in this group)_

### Group AC-07

_(no tests in this group)_

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
