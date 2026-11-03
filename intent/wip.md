---
verblock: "2026-11-03T09:34:23Z:v0.8: vc - localfold: ISSUE:0002's fix awaits validation; ISSUE:0006 queued"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: Bring Ccusage up to Intent spec; only WP03 is left, after ISSUE:0002 is validated**

dc builds and vc validates, in hv's order: ISSUE:0002, ISSUE:0006, ISSUE:0004, ISSUE:0003, ISSUE:0005.

- ISSUE:0002 (high): dc's fix is in at `812264d`, and it moves ccusage's Keychain reads and writes to `/usr/bin/security`, as Claude Code does. Next, vc validates it on an exact export: the red test at `d6408f1` going green, the full gate, an audit that no credential bytes reach a process argument, and update-only behaviour. Then comes the live check on hv's real accounts, with hv's OK. Caution for that check: once the fixed build has renewed an item through `security`, the item's partition list will probably drop ccusage's team ID, so brew's 0.1.5 would prompt. Use only the fixed build until 0.1.6 is installed.
- ISSUE:0006 (medium): adopt devbin. Commit hv's vendored install unedited, declare `rust` and `shell` and the real commands in `bin/.devbin/config.yaml`, and point `usage-rules.md` at `bin/devbin`. `.gitattributes` and `bin/` stay untracked until dc reaches it.
- ISSUE:0004 (medium): replace the release tooling's Python with a Rust `xtask`, shipped in 0.1.6, and remove the Python as soon as it is replaced.
- ISSUE:0003 (medium): one login per account. Sessions keep `~/.config/claude` and borrow the active profile's login through `CLAUDE_SECURESTORAGE_CONFIG_DIR`; `ccusage use` copies the profile's `oauthAccount` into the shared `.claude.json` and sets launchd's copy of the variable.
- ISSUE:0005 (medium): hv's Claude Usage fork (`~/Devel/forks/claudecodeusage`, upstream PR #15) folds in duplicate grouping, follows the account Claude Code is using, names the preferred item as Claude Code does, and stays read-only. It ships as one PR through upstream, not in 0.1.6. The push and the PR edit need hv's OK.

## Active Steel Threads

- ST0001: Bring Ccusage up to Intent spec. WP03 (vc) records the Keychain decision in `intent/llm/` once ISSUE:0002 is validated. AC-00.1 (doctor clean, no advisory) is satisfied when the thread closes.

## Upcoming Work

- hv cuts 0.1.6 once ISSUE:0002, ISSUE:0004 and ISSUE:0003 work on rhadamanth, with the release running on the new `xtask` tooling.
- After 0.1.6: the ISSUE:0003 rollout (shared mode on, Molt's launcher export, restart the sessions, retire `ca646b28`).

## Notes

- All work happens on `main` in the one checkout (hv, 2026-11-03). Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor).
- Never point a ccusage profile at `~/.config/claude` before ISSUE:0002 ships: a renewal would strip `apple-tool:` from the main Claude Code item (vc watch-out 1).
- Never run a pre-fix local ccusage build against the real XDG config: it is ad hoc signed and in no item's access list, so it prompts once per profile. Test in a throwaway HOME and XDG tree.
- `security -i` falls back to the login keychain when the keychain it names does not exist. The ISSUE:0002 test refuses to run without its fixture keychain.
- AGENTS.md's build and test lines are Intent's generic language text (Bats, cargo without `--locked`), and Intent offers no project hook. `usage-rules.md` points at the real checks instead, and at `bin/devbin` once ISSUE:0006 lands. intent-vc files the template gap as an Intent issue on hv's go.
