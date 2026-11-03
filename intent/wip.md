---
verblock: "2026-11-03T08:44:16Z:v0.6: vc - ISSUE:0005 goes to dc after ISSUE:0003, as one PR"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: Bring Ccusage up to Intent spec, while dc fixes ISSUE:0001 and ISSUE:0002 for vc to validate**

- ISSUE:0001 (low): bare `ccusage` shows the active account's usage and `ccusage --all` every account; help only from `help` and `--help`. dc builds first.
- ISSUE:0002 (high): ccusage's renewal writes go through Security.framework and strip `apple-tool:` from Claude's Keychain items, so `/usr/bin/security` (Claude Code, Claude Usage) prompts after every renewal. dc writes the red test on a disposable keychain, then moves reads and writes to `/usr/bin/security` as Claude Code does. It is checked live on rhadamanth with a local build, then hv cuts the brew release.
- ISSUE:0004 (medium): the release tooling ships Python. hv chose a Rust `xtask`, built now and shipped in 0.1.6, with the Python removed as soon as it is replaced. dc builds it after ISSUE:0002.
- ISSUE:0003 (medium): engineering.01 appears twice in Claude Usage, because the main config holds its own login of a profile's account. The design is one login per account: sessions keep `~/.config/claude`, borrow the active profile's login through `CLAUDE_SECURESTORAGE_CONFIG_DIR`, and `ccusage use` copies the profile's `oauthAccount` into the shared `.claude.json`. hv confirmed that write; dc builds it last.
- ISSUE:0005 (medium): hv's Claude Usage fork (`~/Devel/forks/claudecodeusage`, upstream PR #15) folds in duplicate grouping, follows the account Claude Code is using, names the preferred item as Claude Code does, and stays read-only. dc builds it after ISSUE:0003, as one PR (#15, retitled); it ships through upstream, not in 0.1.6. The push and the PR edit need hv's OK.

## Active Steel Threads

- ST0001: Bring Ccusage up to Intent spec. WP01 (dc): languages, Intent's Claude canon and git gate, AGENTS.md. WP02 (vc): whiteboard roster and these records. WP03 (vc): the Keychain decision in `intent/llm/`, once ISSUE:0002 lands.

## Upcoming Work

- hv cuts 0.1.6 once ISSUE:0001 to ISSUE:0004 work on rhadamanth, the release running on the new `xtask` tooling.
- After 0.1.6: the ISSUE:0003 rollout (shared mode on, Molt's launcher export, restart the sessions, retire `ca646b28`).

## Notes

- All work happens on `main` in the one checkout (hv, 2026-11-03).
- Never point a ccusage profile at `~/.config/claude` before ISSUE:0002 ships: a renewal would strip `apple-tool:` from the main Claude Code item (vc watch-out 1).
