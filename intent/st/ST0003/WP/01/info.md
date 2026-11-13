---
wp_id: WP-01
title: Proof: a running session follows a login changed under it
scope: XS
status: Done
---

# WP-01: Proof: a running session follows a login changed under it

## Objective

On the installed Claude Code, show that a session already running picks up a login that another process stores in the same place, with no restart, and record how long it takes and whether anything prompts. Nothing else in this thread is built until it passes.

- **Who:** vc writes the steps and records the result here. hv runs them: two browser logins on a scratch config, about ten minutes, with no ccusage code involved. hv's word, 2026-11-13: this runs before anything else, and its facts decide what comes next.
- **Set up:** a scratch config, `/tmp/ccusage-trial`, and a scratch working directory, so no real login, session or project is touched. Its Keychain item would be `Claude Code-credentials-b029a948`, or `-aec1db1a` if Claude Code resolves `/tmp` first.
- **Terminal A:** `mkdir -p /tmp/ccusage-trial-work && cd /tmp/ccusage-trial-work`, then `env -u CLAUDE_SECURESTORAGE_CONFIG_DIR CLAUDE_CONFIG_DIR=/tmp/ccusage-trial claude --debug-file /tmp/ccusage-trial-a.log`. Log in as hello, run `/usage` and note the weekly figure (about 73%), send a prompt, and leave it open.
- **Terminal B:** the same directory and the same command with `--debug-file /tmp/ccusage-trial-b.log`. Run `/login` as engineering00 (0% used), and send a prompt.
- **Back in A, about a minute later:** send a prompt, then `/status` and `/usage`.
- **Why two accounts with different usage:** `/status` reads the account record in `.claude.json`, which B's login rewrites, so it cannot show whose token A sends. `/usage` asks with A's own token: a weekly figure near 0% means A now holds engineering00's login, and one near 73% means it still holds hello's.
- **Pass:** A answers with no restart, and its `/usage` shows engineering00's figure.
- **Record:** whether A followed and after how long; any prompt or notice A showed; what its debug log says of the change; Claude Code's version; the two flags (`tengu_bubbly_fountain`, `tengu_streamed_thimble`) in the scratch `.claude.json`'s cache; and that no real Keychain item is new or gone.
- **Clean up (hv, after vc has read the logs):** `/logout` in one trial session, close both, and remove `/tmp/ccusage-trial`, `/tmp/ccusage-trial-work` and the two logs. vc then checks from Keychain metadata that the trial's item holds no login or is gone, and gives hv the delete command if it is still there.
- **Shortcut:** if hv already sees, on the current Claude Code, that sessions sharing one config follow a `/login` made in one of them, hv says so and that is the record.
- **If it fails:** this thread stops, and ISSUE:0003 closes by its launcher fix instead.
- **Done when:** AC-01.1 is satisfied by the record.

**The record: passed.** hv ran it on 2026-11-13 on rhadamanth, on Claude Code 2.1.323, and vc read the two debug logs, the scratch config and Keychain metadata. Times are UTC, from the logs and the Keychain.

- **06:52:50 and 06:53:41:** sessions A and B started on the scratch config. Read from the processes, each carried `CLAUDE_CONFIG_DIR=/tmp/ccusage-trial` and no `CLAUDE_SECURESTORAGE_CONFIG_DIR`.
- **06:55:33:** A logged in as hello. The trial's item, `Claude Code-credentials-b029a948`, was created: the hash is of `/tmp/ccusage-trial` as given, not resolved. A showed 77% of its weekly limit, resetting 2026-11-16 10:00.
- **B on the first-run screen did not follow:** it stayed on "Select login method" for the three minutes before its own login, and its log shows nothing of A's. The first-run screen does not adopt a sibling's login.
- **06:57:32:** A answered a prompt as hello.
- **06:58:43:** B logged in as engineering.01. The item was rewritten and the account record became engineering.01@geodica.com. hv meant engineering00, and engineering01 served as well, being at 0% that morning.
- **06:59:56:** A, not restarted and with no request since 06:57:32, fetched its usage with its own token: 1% used, resetting 2026-11-19 21:00. That is engineering01's window, and what B showed. So A held the new login 73 s after it was stored. That is an upper bound, since A was not asked sooner.
- **07:01:08:** A's next prompt continued the conversation begun as hello. The API answered 404 `thread_not_found`: "No thread state was found for the requested `previous_message_id`. Replay the full conversation with `thread: {"type": "create"}` to start a new Thread." Claude Code retried at once (`[tether] forcing create for retry: replay_not_found`), and the answer streamed 0.8 s later. hv saw a normal answer, "Worked for 1s".
- **After it:** `/status` in A named engineering.01@geodica.com, and the 77% warning line was gone.
- **No prompt:** A showed no notice, refusal or session-owner question, and its log has no auth error. Neither `tengu_bubbly_fountain` nor `tengu_streamed_thimble` was among the 1283 flags in the scratch config's cache, so both stood at their defaults.
- **What stayed the old account's:** A still listed the MCP servers it loaded at start, 2 connected and 3 needing auth, and its log has no MCP line after the switch.
- **Nothing real was touched:** 13 `Claude Code-credentials*` items before, and those 13 and the trial's one during.
- **What it adds to the design:** a session's first request after a switch costs one 404 and a replay of the whole conversation, uncached, on the new account. Claude Code does that itself, within its own retry allowance (attempt 1 of 11).

## Acceptance

Acceptance Criteria for this work package are RENDERED into `ST0003/acceptance.md`, under the `WP-01` heading. THAT FILE IS A GENERATED VIEW -- a row authored there is discarded by the next sync. The contract is canon in the thread's model, and the verbs write it: `intent ac new` and `intent at new` mint a row, `intent ac edit` and `intent at edit` reword or re-cite one, and `intent ac satisfy|unsatisfy|descope|rescope|withdraw|reinstate` and `intent at green|red|na` move its state. This cover never restates them.

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
