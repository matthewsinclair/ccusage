---
verblock: "2026-11-03T09:12:09Z:v0.7: vc - ST0001/01 validated; AGENTS.md template gap raised with intent-vc"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: Bring Ccusage up to Intent spec; only WP03 is left, after dc's ISSUE:0002 fix**

- ISSUE:0002 (high, dc building now): ccusage's renewal writes go through Security.framework and strip `apple-tool:` from Claude's Keychain items, so `/usr/bin/security` (Claude Code, Claude Usage) prompts after every renewal. dc writes the red test on a disposable keychain, then moves reads and writes to `/usr/bin/security` as Claude Code does. It is checked live on rhadamanth with a local build, then hv cuts the brew release.
- ISSUE:0004 (medium): the release tooling ships Python. hv chose a Rust `xtask`, built now and shipped in 0.1.6, with the Python removed as soon as it is replaced. dc builds it after ISSUE:0002.
- ISSUE:0003 (medium): engineering.01 appears twice in Claude Usage, because the main config holds its own login of a profile's account. The design is one login per account: sessions keep `~/.config/claude`, borrow the active profile's login through `CLAUDE_SECURESTORAGE_CONFIG_DIR`, and `ccusage use` copies the profile's `oauthAccount` into the shared `.claude.json`. hv confirmed that write; dc builds it last.
- ISSUE:0005 (medium): hv's Claude Usage fork (`~/Devel/forks/claudecodeusage`, upstream PR #15) folds in duplicate grouping, follows the account Claude Code is using, names the preferred item as Claude Code does, and stays read-only. dc builds it after ISSUE:0003, as one PR (#15, retitled); it ships through upstream, not in 0.1.6. The push and the PR edit need hv's OK.

## Active Steel Threads

- ST0001: Bring Ccusage up to Intent spec. WP01 and WP02 are done and validated. WP03 (vc) records the Keychain decision in `intent/llm/` once ISSUE:0002 lands, and AC-00.1 (doctor clean, no advisory) is satisfied when the thread closes.

## Upcoming Work

- hv cuts 0.1.6 once ISSUE:0002 to ISSUE:0004 work on rhadamanth (ISSUE:0001 is closed), the release running on the new `xtask` tooling.
- After 0.1.6: the ISSUE:0003 rollout (shared mode on, Molt's launcher export, restart the sessions, retire `ca646b28`).

## Notes

- All work happens on `main` in the one checkout (hv, 2026-11-03).
- Never point a ccusage profile at `~/.config/claude` before ISSUE:0002 ships: a renewal would strip `apple-tool:` from the main Claude Code item (vc watch-out 1).
- AGENTS.md's build and test lines are Intent's generic language text (Bats, cargo without `--locked`), and Intent offers no project hook. `usage-rules.md` points at the real checks instead. vc raised the template gap with intent-vc at hv's request on 2026-11-03; it is not being fixed today, and in the estate devbin owns building.
