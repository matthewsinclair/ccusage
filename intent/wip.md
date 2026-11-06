---
verblock: "2026-11-06T13:39:57Z:v0.24: vc - fold for hv's travel; 0.1.9 is one cut"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Ccusage 0.1.9: one cut with every outstanding issue (hv, 2026-11-06)**

dc builds and vc validates.

- ISSUE:0007 (medium): adopt devbin's dev/opt/brew contract through devbin#0326, which devbin-cc is building in devbin's core for devbin's next release. The switch and the publish are `bin/devbin use` and `bin/devbin install`, run in the checkout. `ccusage use NAME` stays the account verb, and ccusage gains no verb, only T4's line on `--version`. Once the devbin release carrying #0326 is vendored (`bin/devbin upgrade`), dc wires the three parts in the issue's Fix outline: the `install:` block in `bin/.devbin/config.yaml`, the `--version` reader the contract spells, and the formula on `@@INSTALL@@` on macOS and Linux with the archive's binary at `bin/ccusage` (dc hold 8, vc hold 4). Then hv runs the full suite, and T10's proof runs with hv.
- ISSUE:0003 (medium): blocked at step 3. 19 of 29 Claude sessions launch with no `CLAUDE_SECURESTORAGE_CONFIG_DIR` and 9 with engineering00's, because each inherits from a herdr shell older than Molt's export or than the last `ccusage use`. So the main config's own login, `ca646b28`, keeps renewing, and step 4 must not run. hv chooses the fix: restart the herdr shells and then the sessions, or have `intent claude start` (or herdr) launch through `ccusage run --`, which reads the active profile at each launch. Then vc re-checks step 3, hv runs step 4, and the live check closes the issue (vc todo 4, dc hold 6).

## Active Steel Threads

None.

## Upcoming Work

- The tap's CI runs `brew style` and `brew audit --strict` again once 0.1.9's formula is published by devbin 0.1.20 or later. The vendored 0.1.20 already carries the fix; the tap's live formula was rendered by 0.1.19 (vc hold 3).

## Notes

- All work happens on `main` in the one checkout. Every commit passes the pre-commit gate (guards, the rust and shell critics, intent doctor). hv pushes, and hv runs a release: one command, `bin/devbin release X.Y.Z` (`docs/releasing.md`).
- Validation is hv's one layer of proof: vc reads the diff, and the author's one measured run is the run. cargo and bats run in a slot from the estate desk. Test only what changed; hv runs the full suite by hand before the cut.
- Never run a local ccusage build against the real XDG config without hv's OK: a renewal spends real tokens. Test in a throwaway HOME and XDG tree.
- What a running Claude session launched with is in its process (`ps -E -ww -p <pid> -o command=`), not in a Bash tool's shell, which re-runs Molt's startup for each command.
