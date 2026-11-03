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

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
