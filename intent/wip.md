---
verblock: "2026-11-03T07:48:41Z:v0.2: vc - Focus after triaging ISSUE:0001 to ISSUE:0004"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: Bring Ccusage up to Intent spec, while dc fixes ISSUE:0001 and ISSUE:0002 for vc to validate**

- ISSUE:0001 (low): bare `ccusage` shows the active account's usage and `ccusage --all` every account; help only from `help` and `--help`. dc builds first.
- ISSUE:0002 (high): ccusage's renewal writes go through Security.framework and strip `apple-tool:` from Claude's Keychain items, so `/usr/bin/security` (Claude Code, Claude Usage) prompts after every renewal. dc writes the red test on a disposable keychain, then moves reads and writes to `/usr/bin/security` as Claude Code does. It is checked live on rhadamanth with a local build, then hv cuts the brew release.
- ISSUE:0003 (medium): engineering.01 appears twice in Claude Usage, from two logins of one account. The design (one login per account, sessions selecting it with `CLAUDE_SECURESTORAGE_CONFIG_DIR`) waits on hv.
- ISSUE:0004 (medium): the release tooling ships Python. hv picks the replacement: a Rust `xtask` or zsh.

## Active Steel Threads

- ST0001: Bring Ccusage up to Intent spec. WP01 (dc): languages, Intent's Claude canon and git gate, AGENTS.md. WP02 (vc): whiteboard roster and these records. WP03 (vc): the Keychain decision in `intent/llm/`, once ISSUE:0002 lands.

## Upcoming Work

- hv cuts the brew release once ISSUE:0002 works on rhadamanth.
- Port the release tooling off Python (ISSUE:0004), on the option hv picks.

## Notes

- All work happens on `main` in the one checkout (hv, 2026-11-03).
- Never point a ccusage profile at `~/.config/claude` before ISSUE:0002 ships: a renewal would strip `apple-tool:` from the main Claude Code item (vc watch-out 1).
