---
st_id: ST0003
title: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow
---

# ST0003: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow -- Acceptance

> **THIS FILE IS A GENERATED VIEW, AND A ROW AUTHORED HERE IS DISCARDED BY THE NEXT SYNC.** The acceptance contract is canon in the thread model; this file renders it. Acceptance Criteria (AC) are the ratified completeness boundary; Acceptance Tests (AT) are the small red-to-green tests that prove them.
>
> Done = every AC is covered by a GREEN AT, or (for a non-test AC) its named evidence is satisfied, AND the AC set is the ratified full boundary. Done is read from this map, never from a hand-ticked box.
>
> Test-backed satisfaction is COMPUTED from covering green ATs and never stored -- storing it would be double truth. An AC has four states, not two: beyond satisfied and unsatisfied, a requirement can be **descoped** to a named thread or **withdrawn** with its reason on the record. Both are non-blocking and both are reported separately, so a thread that descoped half its contract looks like one.

## Acceptance Criteria

### WP-01 -- Proof: a running session follows a login changed under it (status: Done)

- AC-01.1 (non-test) hv's trial on the installed Claude Code shows a running session following a login that another process changed, with no restart, and the record names the version, the delay and any prompt. -- evidence: ST0003/WP01's record of hv's trial, 2026-11-13 on Claude Code 2.1.323: session A held the login B stored 73 s earlier, with no restart and no prompt, and answered its next request after one automatic thread replay -- satisfied: yes

### WP-02 -- The login move: claudeAiOauth between stores, locked, journaled, never lossy (status: Done)

- AC-02.1 A move leaves each account with exactly one login, moves only `claudeAiOauth`, and leaves every other key of each document unchanged. -- satisfied: yes (computed)
- AC-02.2 A move interrupted after any step is finished by the next run, and at no point is an account without a login. -- satisfied: yes (computed)
- AC-02.3 Each refusal (no parked login, an only login in the way, an outgoing login that does not verify, a document over the `security -i` limit) happens before any write. -- satisfied: yes (computed)
- AC-02.4 A move waits for a held Claude Code lock, and on a disposable keychain it creates no item and keeps `apple-tool:` on each item it writes. -- satisfied: yes (computed)

### WP-03 -- Shared mode on one live login: use, run, dir, login, logout and adoption (status: Done)

- AC-03.1 In shared mode `ccusage use NAME` puts NAME's login in the shared store and its account record in the shared `.claude.json`, parks the previous account's login in its profile, and unsets launchd's variable. -- satisfied: yes (computed)
- AC-03.2 Usage fetches, renewals and `list` read each account's login from where it lives, before and after a switch. -- satisfied: yes (computed)
- AC-03.3 `ccusage dir` prints no directory for the active account and fails, `ccusage run` gives its child no login directory, `login` of the active profile seats the new login, and `logout` of the active profile is refused. -- satisfied: yes (computed)
- AC-03.4 The first `use` on 0.1.8's layout adopts it, replacing the shared config's login only when its account has another login parked or it can no longer be renewed; parking everything restores that layout; and with shared mode off every existing test passes unchanged. -- satisfied: yes (computed)

### WP-04 -- Doctor for the one live login (status: WIP)

- AC-04.1 `ccusage doctor` reports where each login is, an interrupted move, a shared login that is not the active profile's, and either Claude Code flag that is off its default. -- satisfied: no (computed)
- AC-04.2 `ccusage doctor` lists running Claude sessions whose launch environment names another login directory or another config directory. -- satisfied: no (computed)
- AC-04.3 A usage report says what its account type gives: the plan its login records, as written, and a note in place of figures where the type gives no usage access or no windows. -- satisfied: yes (computed)

### WP-05 -- Docs and the decision record for the one live login (status: Not Started)

- AC-05.1 (non-test) README, ARCHITECTURE.md, RULES.md and the CHANGELOG describe the one live login as built, and ISSUE:0003's record points to this thread. -- satisfied: no

### WP-06 -- Rollout on rhadamanth, closing ISSUE:0003 (status: Not Started)

- AC-06.1 (non-test) After the rollout on rhadamanth no Claude session carries `CLAUDE_SECURESTORAGE_CONFIG_DIR`, each account has exactly one login, and Claude Usage lists each account once with nothing prompting. -- satisfied: no
- AC-06.2 (non-test) With sessions running, `ccusage use` and `ccusage which --use` each reach a session that was already open at its next request, with no restart. -- satisfied: no

## Acceptance Tests

### WP-01 -- Proof: a running session follows a login changed under it (status: Done)

_(no tests in this group)_

### WP-02 -- The login move: claudeAiOauth between stores, locked, journaled, never lossy (status: Done)

- AT-02.1 `src/seat.rs:725` -- covers AC-02.1 -- status: green -- A move leaves each account one login, and every other key of each document, in its order, as it was; red under the swap_remove mutant in WP02's mutant run at ffc95a1, whose move code 90bd7ea keeps, once the test's rest() stopped reading through without_login; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.2 `src/seat.rs:756` -- covers AC-02.1 -- status: green -- Parking the seated login leaves the shared store holding none, its other keys as they were; red before WP02: written against seat::run, it could not compile before it; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.3 `src/seat.rs:774` -- covers AC-02.1 -- status: green -- A parked login is seated in a shared store that holds none; red before WP02: written against seat::run, it could not compile before it; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.4 `src/seat.rs:793` -- covers AC-02.2 -- status: green -- A move interrupted at each point the clean run passes keeps every login somewhere, and the next run finishes it as the clean run did; red under the mutant that stops finishing a seat written from Parked, in WP02's mutant run at ffc95a1, whose move code 90bd7ea keeps; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.5 `src/seat.rs:841` -- covers AC-02.2 -- status: green -- Another move is refused, before any write, while an interrupted one is unfinished; red under the mutant that lets any journal resume, in WP02's mutant run at ffc95a1, whose move code 90bd7ea keeps; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.6 `src/seat.rs:889` -- covers AC-02.3 -- status: green -- Each refusal on file stores (nothing to seat, an occupied park, an outgoing login that does not verify, an empty shared store, an only login) comes before any write, journal or selection, and its message carries no token; red under the mutant that drops the verification, in WP02's mutant run at ffc95a1, whose move code 90bd7ea keeps, and in WP02's first run, where its own occupied case was wrong; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.7 `src/seat.rs:958` -- covers AC-02.3 -- status: green -- The only-login refusal gives way to a move that says replace, which WP03's adoption decides; red before WP02: written against seat::run, it could not compile before it; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.8 `src/seat.rs:1039` -- covers AC-02.4 -- status: green -- A move waits while Claude Code holds the shared store's refresh or write lock, or a profile's refresh lock; red under the mutant that takes only the shared store's locks, in WP02's mutant run at ffc95a1, whose move code 90bd7ea keeps; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86
- AT-02.9 `src/seat.rs:1074` -- covers AC-02.4, AC-02.3 -- status: green -- On a disposable keychain, a document over the security -i line is refused before any write, and a move creates no item and keeps apple-tool: and /usr/bin/security on each item it writes; red under the mutant that checks each write only as it is made, in WP02's mutant run at ffc95a1, whose move code 90bd7ea keeps; green in WP02's measured run at 90bd7ea, Desk slot 01M79YXEJ5ZXWANGXNGA1BTBV5, landed as 9afdf86, and in bin/devbin test keychain at 26da10a, Desk slot 01M79ZAX3CB02ZEKV3RSC1JJ5G
- AT-02.10 `src/seat.rs:864` -- covers AC-02.2 -- status: green -- An interrupted move whose parked login has gone from its profile's store is refused as changed and writes nothing more; red under the mutant that takes a lost park for no change, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea

### WP-03 -- Shared mode on one live login: use, run, dir, login, logout and adoption (status: Done)

- AT-03.1 `src/seat.rs:1000` -- covers AC-03.2 -- status: green -- A login is read from the shared store while its profile is seated, from its own store otherwise and with shared mode off, and neither of an unfinished move's profiles is read; red under the mutant whose home ignores the seat, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.2 `tests/cli.rs:900` -- covers AC-03.1 -- status: green -- use seats the profile's login in the shared store, copies only its account record into the shared .claude.json, and clears launchd's login directory; red under the mutant that leaves launchd set, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.3 `tests/cli.rs:936` -- covers AC-03.1, AC-03.2 -- status: green -- A switch parks the previous account's login and seats the next one's, and list and usage read each login where it lives, before and after; red under the mutant whose usage reads the profile's own store, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.4 `tests/cli.rs:979` -- covers AC-03.4 -- status: green -- use --none returns every login to its profile and leaves the shared store with none, the layout an older ccusage reads, with launchd pointed at the active profile on macOS, and use seats it again; refused beside a name and with shared mode off; red under the mutant whose none moves nothing, in WP03's mutant run at 350e689, and under the mutant whose none clears launchd, in the follow-up's mutant run at 46295e6; green in the follow-up's measured run at 46295e6, Desk slot 01M7A1SFMQY40KBJ95VXWDZBT8, landed as 2019b93
- AT-03.5 `tests/cli.rs:870` -- covers AC-03.3 -- status: green -- dir prints no directory for the active account and fails saying why, and names a parked profile's; red under the mutant that names the active one, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.6 `tests/cli.rs:1025` -- covers AC-03.3 -- status: green -- login of the active profile seats its fresh login in place of the old one, and its logout is refused until another is active; red under the mutant that keeps the old login, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.7 `tests/cli.rs:1066` -- covers AC-03.3 -- status: green -- run gives Claude the shared config and no login directory, whatever the shell exported, and switches first to a profile that is not active; red under the mutant that does not switch, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.8 `src/usage.rs:895` -- covers AC-03.1 -- status: green -- A usage cache verifies an outgoing login only for the exact token it checked against the profile's email, where the login lives now, and the profile endpoint's account matches a record by its uuid; red under the mutant that ignores where the login lives, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.9 `src/switch.rs:237` -- covers AC-03.4 -- status: green -- Adoption replaces the shared config's own login only when Claude can no longer renew it, or when its account keeps another login in a profile, and otherwise refuses saying how to keep it; red under the mutant that always replaces, in WP03's mutant run at 350e689; green in WP03's measured run at 350e689, Desk slot 01M7A10VT8E47A13AR5BYK4RK6, landed as eea2dea
- AT-03.10 `src/usage.rs:938` -- covers AC-03.4 -- status: green -- A login is dead only when the token endpoint refuses its refresh with invalid_grant; another 400, a 401 or a 5xx is an error, so adoption stops instead of replacing; red under the mutant that takes any failure for dead, in WP03's mutant run at 350e689, and under the mutant that takes any 400 for dead, in the follow-up's mutant run at 46295e6; green in the follow-up's measured run at 46295e6, Desk slot 01M7A1SFMQY40KBJ95VXWDZBT8, landed as 2019b93

### WP-04 -- Doctor for the one live login (status: WIP)

- AT-04.9 `tests/cli.rs:1633` -- covers AC-04.3 -- status: green -- Each usage report gives the plan its login records, exactly as Claude Code wrote it, and null where it records none; red under the mutant whose report carries no plan, in the follow-up's mutant run at 46295e6; green in the follow-up's measured run at 46295e6, Desk slot 01M7A1SFMQY40KBJ95VXWDZBT8, landed as ed77832

### WP-05 -- Docs and the decision record for the one live login (status: Not Started)

_(no tests in this group)_

### WP-06 -- Rollout on rhadamanth, closing ISSUE:0003 (status: Not Started)

_(no tests in this group)_

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
