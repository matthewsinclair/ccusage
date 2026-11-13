---
st_id: ST0003
title: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow
status: WIP
created: 2026-11-13
completed:
---

# ST0003: Live account switching: ccusage use moves the active login into the shared config, and running sessions follow

## Objective

`ccusage use NAME` switches every Claude Code session on the machine, running or new, to NAME's account at that session's next request, with no logout, no login and no restart. In shared mode the shared Claude config's own login becomes the one login every session reads, and `ccusage use` moves the chosen account's login into it and the previous account's back to its profile. Sessions then need nothing from their launcher but `CLAUDE_CONFIG_DIR`, which removes the stale-shell fault that blocks ISSUE:0003.

## Context

**Why.** hv, 2026-11-13, to vc in the live channel: changing accounts means logging out and in, "a real palaver", and could `ccusage use` be "propagated or injected into all running Claude Code sessions"? Shared mode as built (ISSUE:0003, shipped in 0.1.6) cannot do it. It points each session at the active profile's login through `CLAUDE_SECURESTORAGE_CONFIG_DIR`, which a process reads from its own environment, fixed at launch. So a switch reaches only sessions started afterwards, and only those started from a shell that exported the new value. hv's word after reviewing this thread, 2026-11-13: run WP01's trial before anything else and record its facts, and if they land badly, that changes what comes next.

**What Claude Code does.** Read by vc from the installed 2.1.323 binary on 2026-11-13; its minified names are in brackets. WP01's trial bore this reading out the same day, and added the fact about threads below.

- It builds the Keychain service name at each use from the process's own `CLAUDE_SECURESTORAGE_CONFIG_DIR`, else from its config directory [`BV`]. Nothing outside the process can change which item it reads.
- Before a request it settles the stored login [`U4`, `Ru`]. Its change check [`AO`, `ku`] drops the token it holds in memory and reads the item again, through a cache that lives 30 s [`Yzt=30000`]. So a session uses whatever login the item holds, at most 30 s after it changed.
- A refresh takes `.oauth_refresh.lock`, reads the item again strictly, and refreshes only the token it has just read. When the stored access token is not the one it came in with, it adopts the stored one and does not call the token endpoint (`tengu_oauth_token_refresh_race_resolved`). So a session never refreshes a login that was moved out from under it.
- It still has the `.storage-write` lock that ccusage's renewals take, with its own handling for a lock that is compromised.
- A session that is showing its login notice adopts a login another process stores (`_adoptSiblingLogin`, flag `tengu_streamed_thimble`, on unless the flag says otherwise).
- A session records the account that started it. Behind the flag `tengu_bubbly_fountain`, off unless the flag says otherwise, it refuses requests (`session_owner`) and asks whether to exit, log in again or copy the conversation when the stored account differs (`askAboutSessionOwner`). Neither flag was in the flag cache in `~/.config/claude/.claude.json`, written 05:43Z, so both stood at their defaults. While the second stays off, a changed login is followed without a prompt. Anthropic can turn it on.
- The server keeps a conversation's thread for the account that made it. WP01 saw the first request after a change of account get a 404 `thread_not_found`, and Claude Code replay the whole conversation as a new thread at once (`[tether] forcing create for retry: replay_not_found`). The person sees a normal answer, 0.8 s later in the trial. The replay is uncached and counts against the new account.
- It has a retry that strips thinking signatures (`tengu_thinking_signature_strip_retry`), should the new account's request reject them. The trial's short conversation did not need it.
- To read it again: `rg -a -o '.{0,500}<literal>.{0,500}' ~/.local/share/claude/versions/<version>`.

**What Claude Usage does.** hv's fork at 29a4db2, whose own build is the copy running on rhadamanth: it lists every `Claude Code*` Keychain item, reads each access token, never renews one, and hides an item whose document has no `claudeAiOauth` key (`ClaudeCredentialParser`).

**What Molt does.** Its `env.sh` and its launchd plist export `CLAUDE_SECURESTORAGE_CONFIG_DIR` only when `ccusage dir` prints an existing directory, and unset it otherwise. So once `ccusage dir` stops naming one, every new shell and launchd drop the variable with no change to Molt.

**The design: one live login.** In shared mode each account has exactly one login. The active account's is in the shared config's own store: its Keychain item, or its `.credentials.json` where Claude Code keeps a file. Every other account's is parked in its profile's store. `ccusage use NEW` holds Claude Code's refresh and write locks for each store it touches and moves the `claudeAiOauth` value only, leaving every other key of each document where it is. The shared login goes back to the outgoing profile, NEW's parked login goes into the shared store, and NEW's profile is then left without one. Each write is read back before the next, so every login exists in at least one place at every moment, and a journal in the state directory lets the next run finish a move that was interrupted. Last, `use` writes NEW's account record into the shared `.claude.json`, as it does today.

**What it replaces.** The per-launch directory. `ccusage dir` stops naming one for the active account, `use` unsets launchd's variable where it set it, and a session needs only `CLAUDE_CONFIG_DIR` set to the shared config, as `ccusage run` and Molt both give it. ISSUE:0003's step 4 is reversed: the shared config's login is kept and becomes the live one. That issue closes at this thread's rollout. Claude Usage then lists each account once, because each parked profile and the shared config hold a login and the active profile's own item holds none.

**Live state on rhadamanth at 2026-11-13 06:14Z, from metadata only.** 25 `claude` processes, all with `CLAUDE_CONFIG_DIR=~/.config/claude`: 23 launched with hello's profile directory and 2 with engineering01's. Of those two, geodica-estate started at 05:03Z, before the 05:22Z switch to hello, and a Devbin session started at 05:38Z from a shell whose export is older than that switch. None runs on the shared config's own login, `ca646b28`, last written 2026-11-11 12:38Z. The five profile items were all written today. hv is returning engineering04 to the free plan (2026-11-13), which leaves four accounts, and nothing in this thread depends on the count.

**Risks, each with its check.**

- Claude Code changes how it reads a login again, or turns the session-owner prompt on. A switch then reaches a session late, or stops it at a prompt. `ccusage doctor` reports the flags from Claude Code's cache, and WP01's trial is run again when Claude Code moves.
- A login is lost, or parked under the wrong profile. The move never overwrites a login that is its account's only one, verifies the outgoing login against its profile's email as a usage fetch does, and is tested at every interruption point on file stores and on a disposable keychain.
- The shared document is too long for one `security -i` line, 4032 bytes as hex, eg once it holds MCP logins. ccusage refuses such a write today and does so here, before anything moves. The rollout measures the document's length first. Whether ccusage may fall back to argv, as Claude Code does, would be hv's ruling.
- A session launched from a shell that still exports a profile's directory does not follow a switch, and loses its login when that profile becomes active. `ccusage doctor` lists such sessions from their launch environment, and the rollout restarts every session once from a fresh shell.
- What a session loaded for its account at start, eg claude.ai connectors, Remote Control and managed settings, stays the old account's until that session restarts. Inference, and so the weekly allowance, moves at once.
- The first request after a switch replays the whole conversation, uncached, on the new account, as WP01 saw. For a long session that is a real one-off cost, the same as a restart's, and a switch pays it once for every session that goes on working.

**Not in scope.** Two accounts live at once in shared mode: a parked profile can still be pinned for one session with `CLAUDE_SECURESTORAGE_CONFIG_DIR=$(ccusage dir NAME)`, at that session's own risk. Switching on a timer. Creating or deleting a Keychain item. Any change with shared mode off.

**Order.** WP01 gated the rest and passed on 2026-11-13: a running session followed a changed login within 73 s, with no restart and no prompt. Next WP02, then WP03 with WP04 beside it, then WP05. WP06 is hv's rollout, which runs the unreleased build from the opt tree that ISSUE:0007 provides.

**Seats.** cc builds WP02 to WP05, and ic signs the verbs' text and the README's. vc writes WP01's and WP06's steps and checks them, and hv runs them. hv starts cc and ic once WP01's facts are in. `which --use` and the Herdr panel's switch (ST0002) call the `use` function this thread owns.

**Release.** hv, 2026-11-13: "one cut, everything in". It carries this thread, ST0002 and ISSUE:0007.

## Work Packages

| WP    | Title                                                                        | Size | Status      |
| ----- | ---------------------------------------------------------------------------- | ---- | ----------- |
| WP-01 | Proof: a running session follows a login changed under it                    | XS   | Done        |
| WP-02 | The login move: claudeAiOauth between stores, locked, journaled, never lossy | L    | Done        |
| WP-03 | Shared mode on one live login: use, run, dir, login, logout and adoption     | L    | Done        |
| WP-04 | Doctor for the one live login                                                | M    | Not Started |
| WP-05 | Docs and the decision record for the one live login                          | S    | Not Started |
| WP-06 | Rollout on rhadamanth, closing ISSUE:0003                                    | S    | Not Started |

## Acceptance

Acceptance Criteria and Acceptance Tests are RENDERED into `acceptance.md`, which is a GENERATED VIEW -- a row authored there is discarded by the next sync. The contract is canon in this thread's model, and the verbs write it: `intent ac new` and `intent at new` mint a row, `intent ac edit` and `intent at edit` reword or re-cite one, and `intent ac satisfy|unsatisfy|descope|rescope|withdraw|reinstate` and `intent at green|red|na` move its state. This cover never restates them.

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
