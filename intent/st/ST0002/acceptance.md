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

### WP-01 -- Typed usage windows: each window carries its kind and model (status: Done)

- AC-01.1 Every parsed usage window carries its kind, and its model when scoped, read from the `limits` array and the legacy fields alike, and the output of `usage` and of the TUI is unchanged. -- satisfied: yes (computed)

### WP-02 -- The which rule: earliest reset first, with the cap from config (status: Done)

- AC-02.1 On the worked example's windows at 2026-11-13T05:27Z, with the caches' own reset times, the rule chooses hello and ranks engineering04, then engineering01 and engineering02, then engineering00. -- satisfied: yes (computed)
- AC-02.2 An account whose data is older than the usage cache's lifetime, whose weekly reset has passed, or that has no weekly window is unknown: listed with its reason and never chosen. -- satisfied: yes (computed)
- AC-02.3 The cap comes from config.toml, with one default when the key is absent; a config that does not set the key is written back without it; and no test writes the cap as a literal. -- satisfied: yes (computed)
- AC-02.4 With a model named, an account is eligible only while its scoped window for that model is below the cap too. -- satisfied: yes (computed)

### WP-03 -- The which verb: text, versioned JSON and --use (status: Done)

- AC-03.1 `ccusage which` prints the choice with one reason line, then the rest in rank order one line each, and exits 0; with nothing eligible it says so on stdout, names the soonest reset and exits 1. -- satisfied: yes (computed)
- AC-03.2 `ccusage which --json` emits the versioned shape cc and ic agreed, with the same choice and order as the text. -- satisfied: yes (computed)
- AC-03.3 `ccusage use`, the TUI's `a` key and `ccusage which --use` make an account active through one function, and `which --use` changes nothing when nothing is eligible. -- satisfied: yes (computed)
- AC-03.4 `ccusage which --model NAME` with a name no account's scoped window carries is refused, naming the models seen. -- satisfied: yes (computed)

### WP-04 -- Usage history: bounded samples under XDG_STATE_HOME (status: Done)

- AC-04.1 Each successful fetch records a sample for each window whose percent changed, in a 0600 file under XDG_STATE_HOME; a cache hit records nothing; and samples are dropped by age, one weekly window, not at a reset. -- satisfied: yes (computed)

### WP-05 -- The predict verb: burn rate and projected loss (status: WIP)

- AC-05.1 `ccusage predict` reports each account's projected unused percent and the total projected loss from a fixture history, says how much history it rests on, and says plainly when there is too little. -- satisfied: yes (computed)

### WP-06 -- The TUI marks the account which would choose (status: Done)

- AC-06.1 The TUI overview marks the account `which` chooses once every account's fresh report is in, and never on partial or stale data. -- satisfied: yes (computed)

### WP-07 -- Docs: README and CHANGELOG for which and predict (status: WIP)

- AC-07.1 (non-test) README and the CHANGELOG describe `which`, `predict`, the cap key, the history file, the JSON contract, the Herdr mod and the refresh policy as built. -- satisfied: no

### WP-08 -- Herdr mod: a ccusage panel in a Herdr pane, its install verb, and one refresh policy (status: WIP)

- AC-08.1 The Herdr verb writes the mod into a Herdr home, replaces only a copy it wrote that nobody has edited, refuses anything else without `--force`, reports `herdr-app check`'s verdict, and never touches a layout or `keys.json`. -- satisfied: yes (computed)
- AC-08.2 The TUI re-fetches an account once its data has passed the usage cache's lifetime and no sooner, and a rate-limited account is not asked again before its retry time. -- satisfied: yes (computed)
- AC-08.3 (non-test) In hv's Herdr the mod, opened as a pane, shows each account's weekly bar, reset, state and the `next` marker from ccusage's own JSON, refreshes by itself, and passes `herdr-app check`. -- satisfied: no

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Typed usage windows: each window carries its kind and model (status: Done)

- AT-01.1 `src/usage.rs:1253` -- covers AC-01.1 -- status: green -- The kinds and model read from the limits fixture, with the scoped window's share; red before WP01: written against Kind, it could not compile on the parser before it; green in WP01's measured run at 8103e4d (landed as 0da313c), Desk slot 01M79WDS69Q5DKMVMKHDJ9AZS2
- AT-01.2 `src/usage.rs:1361` -- covers AC-01.1 -- status: green -- A legacy-only snapshot maps onto the same kinds; breakdown metadata is not a window; red under the legacy-kind mutant (seven_day read as the session) in WP01's run; green in WP01's measured run at 8103e4d (landed as 0da313c), Desk slot 01M79WDS69Q5DKMVMKHDJ9AZS2
- AT-01.3 `src/usage.rs:1414` -- covers AC-01.1 -- status: green -- An unknown limits kind stays ignored, and limits supersede legacy duplicates; red before WP01: written against Kind, it could not compile on the parser before it; green in WP01's measured run at 8103e4d (landed as 0da313c), Desk slot 01M79WDS69Q5DKMVMKHDJ9AZS2
- AT-01.4 `src/display.rs:939` -- covers AC-01.1 -- status: green -- The overview's labels are unchanged: the pin passed on the parser before WP01 and after it; red in WP01's first run at d3a24e5 (its own expectation was wrong) and under the label mutant; green in WP01's measured run at 8103e4d (landed as 0da313c), Desk slot 01M79WDS69Q5DKMVMKHDJ9AZS2

### WP-02 -- The which rule: earliest reset first, with the cap from config (status: Done)

- AT-02.1 `src/which.rs:409` -- covers AC-02.1 -- status: green -- The worked example at 2026-11-13T05:27Z, with the resets as the endpoint gave them, jitter included, chooses hello and ranks engineering04, then engineering01 and engineering02, then engineering00; red under the raw-instant mutant (resets ranked unrounded) in the first WP02 run at fdfe1e5, Desk slot 01M79X4M2FR0A8NCRRNG412NQG; green in the WP02 landing run at b18d115 (landed as 7301020), Desk slot 01M79YVVC17G99W2CD280ZP6W3
- AT-02.2 `src/which.rs:508` -- covers AC-02.2 -- status: green -- Accounts with no usage, stale usage, a passed weekly reset or no weekly window are unknown with their reasons, unranked and never chosen, and a fresh fetch whose cache save failed stays usable; red before WP02: written against which::choose, it could not compile before it; green in the WP02 landing run at b18d115 (landed as 7301020), Desk slot 01M79YVVC17G99W2CD280ZP6W3
- AT-02.3 `src/config.rs:297` -- covers AC-02.3 -- status: green -- The cap reads from the [which] table with which::DEFAULT_CAP when the key is absent, and a config that did not set it is written back without it; no test writes the cap as a literal, every one derives from DEFAULT_CAP; red before WP02: written against Config::cap and the [which] table, it could not compile before them; green in the WP02 landing run at b18d115 (landed as 7301020), Desk slot 01M79YVVC17G99W2CD280ZP6W3
- AT-02.4 `tests/cli.rs:323` -- covers AC-02.3 -- status: green -- init and add write back a config that set no cap without a [which] table, and keep the cap of one that set it; list refuses a cap of 0; every cap in the test derives from which::DEFAULT_CAP; red under the cap-of-0 mutant in the first WP02 run at fdfe1e5, Desk slot 01M79X4M2FR0A8NCRRNG412NQG; green in the WP02 landing run at b18d115 (landed as 7301020), Desk slot 01M79YVVC17G99W2CD280ZP6W3
- AT-02.5 `src/which.rs:649` -- covers AC-02.4 -- status: green -- With a model named, an account whose scoped window for it is at the cap is at cap and not chosen, the model matched regardless of case; an account with no scoped window for the model is bounded by the weekly window alone, and the headroom is the lesser of the weekly room and the scoped room in weekly terms; red before WP02: written against which::choose, it could not compile before it; green in the WP02 landing run at b18d115 (landed as 7301020), Desk slot 01M79YVVC17G99W2CD280ZP6W3

### WP-03 -- The which verb: text, versioned JSON and --use (status: Done)

- AT-03.1 `tests/cli.rs:1278` -- covers AC-03.1 -- status: green -- which prints the chosen account's line first, with its room before the cap, its reset and its reason, then the rest in rank order, eligible, at the cap and unknown; with nothing eligible its first line says so and names the soonest reset, the reason goes to stderr and it exits 1; help lists the verb; red under the mutants that print the text in reverse order and that exit 0 with nothing eligible, in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F; green in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F (landed as 5139af2)
- AT-03.2 `tests/cli.rs:1316` -- covers AC-03.2 -- status: green -- which --json is version 1 of the shape cc and ic agreed: the top-level and per-account fields by name and in order, the scoped entries, states, ranks and causes, headroom rounded to 0.1, and every time RFC 3339 in UTC to the second with a Z, its accounts in the text's order; cc signed its meaning at 90ca48e; red under the mutants that spell at_cap otherwise and that keep a time's fraction, in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F; green in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F (landed as 5139af2)
- AT-03.3 `tests/cli.rs:1419` -- covers AC-03.3 -- status: green -- which --use makes the choice active through switch::to, the one use function the use verb and the TUI's a key call too; it says so when the choice is active already, reports activated only when it made the choice active, and leaves the active account as it was with nothing eligible; red under the mutant that switches when the choice is active already, in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F; green in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F (landed as 5139af2)
- AT-03.4 `tests/cli.rs:1454` -- covers AC-03.4 -- status: green -- which --model with a name no scoped window carries is refused before anything is printed, naming the models seen; any case of a model seen is that model, spelled as the usage spells it; red under the mutant that matches model names by case, in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F; green in the WP03 run at 90ca48e, Desk slot 01M7A0HZDCG6MXWT7T6AV2VP1F (landed as 5139af2)

### WP-04 -- Usage history: bounded samples under XDG_STATE_HOME (status: Done)

- AT-04.1 `src/usage.rs:1593` -- covers AC-04.1 -- status: green -- A fetch that succeeds saves its cache and records a sample of each of its windows in the history under the state directory; a history that cannot be written joins the report error, and the cache is saved all the same; red under the mutant whose successful fetch records nothing, in the first WP04 run at 31c0482, Desk slot 01M79ZVHT2C8B2ADTBWY7YQ73W; green in the WP04 run at 1831d5c (landed as 68376eb..1831d5c), Desk slot 01M79ZZWCJB2Q02BHAEM58XVKP
- AT-04.2 `src/history.rs:255` -- covers AC-04.1 -- status: green -- A window keeps its first sample, then one each time its percent changes; an unchanged percent adds nothing, and each account keeps its own windows; red under the mutant that samples an unchanged percent, in the first WP04 run at 31c0482, Desk slot 01M79ZVHT2C8B2ADTBWY7YQ73W; green in the WP04 run at 1831d5c (landed as 68376eb..1831d5c), Desk slot 01M79ZZWCJB2Q02BHAEM58XVKP
- AT-04.3 `src/history.rs:336` -- covers AC-04.1 -- status: green -- The history is a 0600 file in the state directory, which the private-file check doctor runs passes; red before WP04: written against history::record, it could not compile before it; green in the WP04 run at 1831d5c (landed as 68376eb..1831d5c), Desk slot 01M79ZZWCJB2Q02BHAEM58XVKP
- AT-04.4 `tests/cli.rs:416` -- covers AC-04.1 -- status: green -- Usage read from a fresh cache, or held back by a rate-limit cooldown, records no history under XDG_STATE_HOME; red before WP04: written against history::FILE, it could not compile before it (the property is structural, as only the success arm records); green in the WP04 run at 1831d5c (landed as 68376eb..1831d5c), Desk slot 01M79ZZWCJB2Q02BHAEM58XVKP
- AT-04.5 `src/history.rs:294` -- covers AC-04.1 -- status: green -- Samples go by age alone, once older than one weekly window: a window that has since reset keeps its sample, and the next window at the same percent is a window of its own; red under the mutants that drop a sample at one week and that know a window without its reset, in the first WP04 run at 31c0482, Desk slot 01M79ZVHT2C8B2ADTBWY7YQ73W; green in the WP04 run at 1831d5c (landed as 68376eb..1831d5c), Desk slot 01M79ZZWCJB2Q02BHAEM58XVKP

### WP-05 -- The predict verb: burn rate and projected loss (status: WIP)

- AT-05.1 `src/predict.rs:388` -- covers AC-05.1 -- status: green -- The burn rate is the rises in the weekly windows across every account over the hours from the first weekly sample to the last, a rise across a reset counting from 0 and a fall as none; the test computes its expected rate from the fixture history; red under the mutants that count no rise across a reset and that count a fall, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7)
- AT-05.2 `src/predict.rs:437` -- covers AC-05.1 -- status: green -- Following which::choose from now at the rate, each account's projected percent at its current reset, held at the cap, its unused share as 100 less that, and the total, each computed in the test from a fixture whose climb to the cap ends part way through a second; red under the mutants that take the last account instead of the rule's choice, that climb past the cap, and that count unused from the cap instead of 100, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7)
- AT-05.3 `tests/cli.rs:1523` -- covers AC-05.1 -- status: green -- ccusage predict says plainly when there is too little history and exits 0, then from a fixture history gives the rate, the samples and hours it rests on, each account's unused share and the total, and the accounts left out, in text and in its versioned JSON; red under the mutant whose JSON gives no samples, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7); then its test writes plans into the logins and checks that the text and the JSON name the plan counted, with spare left out as another plan's allowance, red under the mutants whose text names no plan and whose JSON gives none, in cc's ST0003/WP04 run in Desk slot 01M7A68EK9GKKRVBEVE6G2B8MA, which carried ic's tier grouping and herdr notes; green at 7cac510 in that run (landed as 1648348..b08623d)
- AT-05.4 `src/predict.rs:513` -- covers AC-05.1 -- status: green -- predict counts one plan's accounts: the plan most of the usable accounts record, compared whole, a tie going to the active account's plan, else to the lexically first, and with no usable account the plan most of all the accounts record; a usable account on another plan is left out as another plan's allowance, and the rate counts that plan's samples alone; red under the mutants that let a tie ignore the active account, that give it to the lexically last, that project a usable account on another plan, that count every plan's samples in the rate, that count the least common plan, and that leave no plan when no account is usable, in cc's ST0003/WP04 run in Desk slot 01M7A68EK9GKKRVBEVE6G2B8MA, which carried ic's tier grouping and herdr notes; green at 7cac510 in that run (landed as 1648348..b08623d)

### WP-06 -- The TUI marks the account which would choose (status: Done)

- AT-06.1 `src/display.rs:1046` -- covers AC-06.1 -- status: green -- display::next is which::choose's choice once every report is in and fresh, an account that failed counting as in; a report yet to come, or usage as old as the cache's lifetime, gives no marker and the TUI's notice to refresh; with nothing eligible the overview says so; one account's usage shows no marker; red under the mutants that ignore freshness, that mark every account but the choice, and that count a failed account as not in, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7)
- AT-06.2 `tests/cli.rs:1625` -- covers AC-06.1 -- status: green -- ccusage usage --all marks which's choice next, as the TUI does, while ccusage usage NAME marks none; red under the mutant that leaves usage --all without the marker, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7)

### WP-07 -- Docs: README and CHANGELOG for which and predict (status: WIP)

_(no tests in this group)_

### WP-08 -- Herdr mod: a ccusage panel in a Herdr pane, its install verb, and one refresh policy (status: WIP)

- AT-08.1 `tests/cli.rs:1641` -- covers AC-08.1 -- status: green -- ccusage herdr install writes the mod and a record of it into a scratch Herdr home, refuses a missing home, replaces only its own unedited copy (files Herdr writes there aside), refuses an edited one until --force, reports herdr-app check's verdict and exits 1 when it fails; remove takes out an unedited copy, and an edited one only with --force; layouts and keys.json are as they were; red under the mutants that replace an edited copy, that count Herdr's jsconfig.json as an edit, and that take a failed check for a pass, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7); then its test adds a herdr-app on PATH that cannot be run, said with its cause, and a hidden .git, which install and remove refuse without --force, red under the mutants that call a herdr-app that cannot run missing and that take a hidden directory for Herdr's own, in cc's ST0003/WP04 run in Desk slot 01M7A68EK9GKKRVBEVE6G2B8MA, which carried ic's tier grouping and herdr notes; green at 7cac510 in that run (landed as 1648348..b08623d)
- AT-08.2 `src/display.rs:1136` -- covers AC-08.2 -- status: green -- display::due names the accounts whose usage is missing or older than the cache's lifetime and that the TUI last asked for a whole lifetime ago or more, or never, so no account is asked twice in a lifetime, one whose ask brought nothing fresh included; red under the mutants that ignore when an account was last asked and that take fresh usage as stale, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7)
- AT-08.3 `tests/cli.rs:396` -- covers AC-08.2 -- status: green -- A rate-limited account's cooldown holds through --refresh, its last usage kept and the cooldown said, with no request made; the cache every fetch reads through, the TUI's included, fetches nothing before an account's retry time; red under the mutant that changes what the cooldown says, which the test pins, in the run of WP05, WP06, WP08 and WP07 in Desk slot 01M7A1JYDSQNNHNNN55ACXCX40; green at 70adaf7 in that run (landed as d57a014..258e0a7)

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
