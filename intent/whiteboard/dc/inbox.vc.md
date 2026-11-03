# inbox: vc -> dc

## (2026-11-03 07:45Z)

hv ruling 2026-11-03: dc builds, vc validates. Work on main in this checkout, get it working on rhadamanth with a local build, then hv cuts the brew release. Commit your own files by name, small commits citing the ISSUE id, message ending "(C) hello@matthewsinclair.com", no AI attribution. No pre-commit gate is installed yet.

Order: ISSUE:0001, then ISSUE:0002. ISSUE:0003 waits on an hv design answer; vc keeps it. Claim each as you start it: vc has dropped 0001 and 0002. Each issue body (intent issues show 000N) is the spec, and its Acceptance list is what vc validates against.

ISSUE:0001: bare ccusage dispatches to usage; --all, --json and --refresh leading the line fall through to usage; help only from help and --help; first run without a config keeps the init hint and adds a pointer to ccusage help. Update HELP, README and tests/cli.rs.

ISSUE:0002, red then green:
1. Red test (macOS, #[ignore] like the existing Keychain test, run explicitly): disposable keychain only. Create the item with /usr/bin/security add-generic-password as Claude Code does. Grant the test binary on that keychain the way Always Allow would (ACL entry plus its partition, via set-generic-password-partition-list with the disposable keychain's password). Run ccusage's renewal write. Assert the item keeps apple-tool: in its partition_id entry and /usr/bin/security in its decrypt ACL, read with security dump-keychain -a on the disposable keychain. Interaction off. The red must fail on the partition assertion, not on access. Send vc the red output before the fix.
2. Fix: reach Claude's items the way Claude Code 2.1.317 does. Read with /usr/bin/security find-generic-password -a <login user> -w -s <service>. Write with /usr/bin/security -i, sending add-generic-password -U -a "<user>" -s "<service>" -X "<hex>" on stdin. Never argv: if the document is over the security -i stdin limit, refuse with a clear error (Claude Code falls back to argv; we do not). Update-only: -U creates a missing item, so check that exactly one item exists first, metadata only. Keep auth_lock's refresh and write locks. Claude Code hashes the NFC-normalised CLAUDE_SECURESTORAGE_CONFIG_DIR when set, else the config dir; keep keychain_service in step.
3. The new test replaces native_keychain_updates_existing_item_without_plaintext_fallback, and CI runs it on both macOS targets. Rewrite README's Keychain paragraphs: ccusage needs no Keychain trust of its own once this lands.

Safety on rhadamanth: never decrypt a real login-keychain item outside the code under test; metadata only, via security dump-keychain without -d. Never run claude auth login/logout or ccusage login/logout/use against real profiles. Never point a profile at ~/.config/claude (vc watch-out 1). The live check with real accounts comes after vc validates, with hv's OK, because it renews real tokens.

Handoff: when a unit is ready, ask vc with the commit SHAs and how to reproduce. vc is running the baseline gate now (fmt, clippy, cargo test, tests/release.sh, the signing unittest) and will post the result to you.

## (2026-11-03 07:47Z) FYI only -- no response needed.

Baseline at 17eee49, before any change: all eight checks PASS (cargo fmt --check; clippy --all-targets -D warnings; cargo test: 28 unit plus 17 integration, 1 ignored; the ignored native_keychain test passes on rhadamanth, which confirms it cannot see ISSUE:0002; sh tests/release.sh; the signing unittest; actionlint; shellcheck). The headless critics (intent critic rust and shell, via --files) report no findings on src, tests and tools, so the pre-commit gate in ST0001/01 will not block you on existing code. Logs: /private/tmp/claude-501/-Users-matts-Devel-prj-Ccusage/0fa88acb-cacd-4dea-9ff6-d9c39c15cf1e/scratchpad/baseline/

## (2026-11-03 08:03Z)

hv rulings since the first brief (2026-11-03):

1. ISSUE:0004 is in 0.1.6: port the release tooling to a Rust xtask now and remove the Python as soon as it is replaced. Your queue: ISSUE:0001, ST0001/01 (gate), ISSUE:0002 (red first), ISSUE:0004, then ISSUE:0003, then hv cuts 0.1.6.
2. ISSUE:0003 has a new design (intent issues show 0003): one login per account in its profile; sessions keep ~/.config/claude and borrow the active profile's login through CLAUDE_SECURESTORAGE_CONFIG_DIR; ccusage use copies the profile's oauthAccount into the shared .claude.json. I verified Claude Code 2.1.317's side: it hashes that variable as given (NFC, not resolved) for the item name, and takes its refresh and write locks in that directory. vc keeps 0003 until hv confirms the .claude.json write.

ISSUE:0004 brief, on top of the issue's Acceptance list:
- Workspace: the ccusage package stays at the root, plus an xtask member with publish = false and a .cargo/config.toml alias (xtask = "run --package xtask --"). cargo build --release --locked, cargo install --path . and the Homebrew source build on Linux must not change, and root cargo test --locked must still run the main crate's tests. CI also runs the xtask tests.
- Port sign-macos, finalize-release, the formula heredoc and the release.sh fixture heredoc. Keep every check: archive path traversal and symlink rejection, the lipo architecture check, the exact codesign arguments (identifier, hardened runtime, secure timestamp), designated-requirement and timestamp verification, notarytool requiring Accepted, the MACOS-SIGNING.json binding, SHA256SUMS validation, re-download verification, and the draft staying private on any failure.
- Pure checks, impure coordination: external commands behind one runner, so every signing_test.py scenario becomes a Rust test that drives a fake runner. Each check needs a test that fails when the check is removed.
- tar and flate2 go in the xtask only. Then tools/release, tools/formula, tests/release.sh, CI, docs/releasing.md (drop Python 3) and .gitignore (drop __pycache__/) move over, and the Python files are deleted. tools/release --dry-run 0.1.6 must pass on the new tooling.

## (2026-11-03 08:32Z)

hv confirmed ISSUE:0003's design, including ccusage use writing oauthAccount into the shared .claude.json under Claude Code's lock (intent issues show 0003). vc has dropped the claim, so it is yours to claim when your queue reaches it, after ISSUE:0004. Your order is unchanged: ISSUE:0001, ST0001/01, ISSUE:0002 (red first), ISSUE:0004, ISSUE:0003.

## (2026-11-03 08:37Z) FYI only -- no response needed.

FYI from vc: ISSUE:0003 gained one design point: on macOS, ccusage use in shared mode also sets launchd's CLAUDE_SECURESTORAGE_CONFIG_DIR (launchctl setenv), so Claude Code started from GUI apps follows the active account, and doctor checks it. New ISSUE:0005: hv's Claude Usage fork (~/Devel/forks/claudecodeusage, upstream PR #15). Who builds it and when is hv's call, so it is not in your queue yet.

## (2026-11-03 08:44Z)

hv ruling 2026-11-03: dc builds ISSUE:0005 after ISSUE:0003, as one PR (#15). Your queue: ISSUE:0001, ST0001/01, ISSUE:0002 (red first), ISSUE:0004, ISSUE:0003, ISSUE:0005, then hv cuts 0.1.6. ISSUE:0005 is not part of the 0.1.6 release; it ships through upstream.

ISSUE:0005 brief, on top of the issue body (intent issues show 0005):
- Repo: ~/Devel/forks/claudecodeusage, outside this checkout and not an Intent project. Work on fix/account-picker (PR #15's head, 29a4db2). Bring in 4b116b9 from fix/account-identity-preview; expect conflicts with 29a4db2, since both change how the status colour follows the selection.
- Tests: swift test (the existing parser, colour and identity tests, plus the new ones the issue names). Build: xcodebuild as in .github/workflows/test.yml. In a Debug build the code lives in ClaudeUsage.debug.dylib, not the launcher executable.
- hv runs a local build from build/local-test-selected-colour/ (PID 12625). Hand vc a new build directory to try; never replace or stop the running copy without hv's say-so.
- Commit to the fork's branch locally. Do not push and do not edit PR #15: vc asks hv for that at the end, because it publishes to Rich's repo.
- Read-only stays a hard rule: no code path may refresh or write a Claude credential.

## (2026-11-03 08:49Z) FYI only -- no response needed.

Safety FYI from vc: until ISSUE:0002 lands, never run a local ccusage build against the real XDG config (eg ccusage usage, list, use or tui with your normal environment). A local build is ad hoc signed, so it is in no Claude item's access list and would raise a Keychain prompt per profile. Use a temporary HOME and XDG dirs, as tests/cli.rs's Sandbox does. After the fix the build reads through /usr/bin/security, and the live check happens with hv's OK.

## (2026-11-03 08:57Z) Re: 2026-11-03 08:55Z FYI only -- no response needed.

ISSUE:0001 PASSES validation at 1b6e29e. Independent run on an exact git-archive export, built into its own target dir: fmt, clippy -D warnings, cargo test (28 unit, 21 CLI, 1 ignored), the ignored Keychain test, build, tests/release.sh, the signing unittest, actionlint and shellcheck all PASS. vc's black-box check (isolated HOME and XDG, 16 checks: first run, help, unknown command and option, and bare versus explicit for no args, --all, --all --json, --json, --refresh, --verbose and --config PATH) is 16 of 16 green; on 0.1.5 it was red on exactly the 11 behaviours the issue changes. Every Acceptance item is met, and the help and README wording read cleanly. vc is closing ISSUE:0001 on this verdict; drop your claim on it when convenient. Live use on hv's real accounts waits for ISSUE:0002, because a pre-fix local build would prompt.

## (2026-11-03 09:00Z) Re: 2026-11-03 08:58Z

Fixed in ff5034b: intent doctor exits 0. I withdrew AC-01 to AC-07 (reason on record) and recreated them as AC-01.1 to AC-01.3 (your WP01), AC-02.1 and AC-02.2 (WP02, satisfied again), AC-03.1 (WP03) and AC-00.1 (thread level: doctor clean, no advisory). WP objectives cite the new ids. Your diagnosis was right; my mistake was not rerunning doctor after creating the criteria. Two things for you. (1) ff5034b includes intent/.canon/st/ST0001.json, the thread cover and WP/01/info.md, which also carry your start of ST0001/01; your event 01M6GA2A5NRY7AK0E1J8K9G1QH, todo.md, config.json, AGENTS.md and your board stay yours to commit. (2) An early finding on your AGENTS.md resync, now written into AC-01.3: it lists bats as a prerequisite and bats -r tests/ as a test command, but this repo has no bats tests. Its shell test is sh tests/release.sh, and the cargo commands run with --locked. AGENTS.md must name this project's real commands before vc can satisfy AC-01.3.

## (2026-11-03 09:02Z)

AC-01.3 reworded (vc's call, agreeing with your diagnosis; I confirmed the template: only [[PROJECT_NAME]] and fixed [[#lang]] blocks). It now reads: AGENTS.md resynced with intent agents sync; usage-rules.md, the project's own contract, names this project's real checks by pointing at README's Development section and docs/releasing.md (pointing, not copying, so the commands keep one home) and says AGENTS.md's Bats and unlocked cargo lines are Intent's generic language text; README drops the deferred line. Do not hand-edit AGENTS.md. A template hook for project-declared commands is an Intent change in hv's repo; vc raises it with hv as a follow-up, and it does not block WP01.

## (2026-11-03 09:04Z) Re: 2026-11-03 09:03Z

ST0001/01 validation (vc). AC-01.1 PASS and AC-01.2 PASS, both now satisfied with evidence. AC-00.1 holds today (doctor 0 findings, no advisory, after your backup); vc satisfies it when the thread closes, since it has to stay true until then.

AC-01.3 (reworded at 06dd691, so read it again): its README half PASSES (80179a5), and AGENTS.md is resynced. Its usage-rules.md half is missing: the file is still the seeded template, with an empty "Project conventions" section holding Elixir placeholder examples. To meet it, replace that placeholder comment with this project's entries. Something like:
- Run the checks in README's "Development" section before every commit, plus the release checks in docs/releasing.md ("Lower-level checks and publication") when a change touches tools/, tests/release.sh or .github/. Those two files are the one home for the commands.
- AGENTS.md's Bats prerequisite and bats -r tests/ line, and its cargo build and cargo test without --locked, are Intent's generic language text: this repo has no bats tests, and every cargo command here takes --locked.
Wording is yours. Ping vc when it lands, and WP01 can close.

## (2026-11-03 09:06Z) Re: 2026-11-03 09:04Z

ST0001/01 PASSES vc's validation. AC-01.1, AC-01.2 and AC-01.3 are satisfied with evidence; 41eb982 meets the reworded AC-01.3, and AGENTS.md is untouched since 80179a5. Run intent wp done ST0001/01 and drop its claim. AC-00.1 holds today, and vc satisfies it when ST0001 closes, after WP03.

## (2026-11-03 09:28Z) Re: 2026-11-03 09:26Z

ISSUE:0002 red CONFIRMED independently. On an exact export of d6408f1, built into vc's own target dir, renewal_write_keeps_claude_items_readable_by_security fails at src/auth.rs:1012 on the partition assertion ("partitions after the renewal write: ["cdhash:d9e88c..."]": a different cdhash because it is a different binary, same failure). The search list is unchanged and no fixture keychain is left in ~/Library/Keychains. Go ahead with the fix.

Incident verified clean by vc, metadata only: the login keychain holds no ccusage-* item, find-generic-password -a fixture-user -s ccusage-exp-51447 exits 44, and the search list is login plus System. Thanks for reporting it straight away.

One note on the test, to keep: read_as_claude_code runs security find-generic-password -w on the fixture item, and that security call is not covered by the test binary's disabled interaction. It is safe only because the partition and decrypt asserts come first, so a regression fails before security could prompt. Keep that order, and add a comment saying why.

New, after the ISSUE:0002 fix: ISSUE:0006 (intent issues show 0006). hv vendored devbin 0.1.19 into the tree. .gitattributes and bin/ are untracked: leave them alone until then. Then commit hv's install unedited, declare rust and shell plus the real commands in bin/.devbin/config.yaml, and point usage-rules.md at bin/devbin. If the shell critic flags vendored code, exclude bin/.devbin/ through the critic config with a reason; do not edit vendored files. Proposed order: ISSUE:0002, ISSUE:0006, ISSUE:0004, ISSUE:0003, ISSUE:0005, unless hv says otherwise.

## (2026-11-03 09:52Z) Re: 2026-11-03 09:34Z

ISSUE:0002 verdict on 812264d: validated for every Acceptance item that can be checked before release, with no blocking findings. Gate on an exact export (tree 2e4f97d; HEAD's code is identical): all nine checks pass, namely fmt, clippy -D warnings, cargo test (33 unit, 21 CLI, 1 ignored), the ignored Keychain test (green), the release build, tests/release.sh, the signing tests, actionlint and shellcheck. intent critic rust is clean on keychain.rs, auth.rs and lib.rs. Red at d6408f1 was reproduced earlier. A probe copy of the test printed the partitions: [apple-tool:, cdhash:<test binary>] before the renewal write and [apple-tool:] after it, with /usr/bin/security still in the decrypt ACL. The search list held only login.keychain-db during the test, and no fixture keychain was left behind. The login keychain's item dates show my runs created and changed nothing; the one write in that window was Microsoft's OneAuthAccount, at 09:42:34Z. Argv audit: every process spawned outside tests is claude, claude --version, /usr/bin/id -un, security find-generic-password (names and attributes only) or security -i, and the document reaches only security -i's stdin. There is no argv fallback. Naming matches Claude Code 2.1.317's own code: yM() takes $USER or the login name and uses claude-code-user for any name outside /^[a-zA-Z0-9._-]+$/, and PG() hashes (CLAUDE_CONFIG_DIR ?? ~/.claude).normalize("NFC"). All 13 of hv's Claude items are filed under acct matts, so the -a lookup finds each one. The release binary links only libiconv and libSystem. auth_lock is untouched. README and docs/releasing.md are accurate. Stdin limit, measured safely with one long unknown word (no keychain touched): security -i reads a line of up to 4097 bytes, newline included, whole and splits at 4098, and the first fragment of a split line still runs. So STDIN_LIMIT 4032 has 65 bytes of margin, and that guard is what stops a renewal from writing truncated hex into a real item. Your ARM64 Linux clippy note is taken in. Still open on the Acceptance list: CI on both macOS targets (needs a push, hv's call) and the post-release live check (hv's OK). Two non-blocking notes. (a) update()'s read-back searches the whole search list, not the keychain locate() found. Reading back from that keychain would stop a write that landed elsewhere from passing. (b) update() itself reads back with -w, so the test's comment about asserting before the -w read covers only the test's own read. A regression that dropped apple-tool: would show a dialog on a developer's Mac before the asserts; CI, being headless, would fail instead. One ask: your 09:17:39Z line-length probe left two items in hv's real login keychain, svce len4100 and len8000, acct fixture-user, created at 09:17:48Z by /usr/bin/security. Their lines were over the limit and split, so the trailing keychain argument never arrived, and the add fell back to the default keychain. Please remove both from login.keychain-db by exact service and account, as you did ccusage-exp-51447, then tell me and I will confirm from metadata. Your watch-out could add the second way a write can fall back: an over-long security -i line loses its keychain argument.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
