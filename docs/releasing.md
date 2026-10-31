# Releasing Ccusage

The application version is in `Cargo.toml`; keep `Cargo.lock` in sync. Use stable release tags of the form `v0.1.0`. Tags must match the package version. The canonical Homebrew formula template lives at `packaging/homebrew/ccusage.rb`; do not hand-edit version pins in the tap.

## One command

From a clean Ccusage checkout on `main`, with the `../Brew/homebrew-ccusage` checkout also clean:

```sh
tools/release 0.1.2
```

Pass the new stable version without a leading `v`. The command updates Cargo.toml and Cargo.lock, runs fmt, Clippy, tests, the release build, actionlint, shellcheck, and release-helper integration tests, then commits the version bump. It creates an annotated tag and atomically pushes the source `main` branch and tag. It waits for the exact tag/commit's GitHub Actions run to succeed, downloads the draft archives, signs both Mac architectures locally, verifies the uploaded signatures and checksums, publishes the draft, updates/pushes the pinned Homebrew formula, and waits for the tap’s macOS/Linux Homebrew tests. It does not install or uninstall anything locally.

Required maintainer tools: Git, Cargo/Rust, authenticated `gh`, `actionlint`, `shellcheck`, and Python 3. Final publication runs on macOS using the existing `Developer ID Application: Geodica Pty Ltd (76BQL8L47U)` identity in the maintainer's Keychain. The private key is never exported or uploaded to GitHub. CI creates a draft; a signing or verification failure leaves it private and blocks the tap update.

Before the first signed release, run `tools/test-keychain-upgrade` from a normal terminal. It signs two different fixture builds, creates a uniquely named disposable record in the login Keychain with the first, and reads it from the second at a new path with interaction disabled. It fails rather than opening an access approval dialog for the upgraded build. It deletes that fixture record on exit and never uses Claude credentials. Both Swift builds treat warnings as errors. This requires access to the macOS Security service and Apple's secure timestamp service.

`tools/sign-macos` uses identifier `com.geodica.ccusage`, hardened runtime, a secure timestamp, and the fixed Geodica Developer ID team. It rejects ad hoc signatures, unexpected archives, wrong architectures, differing designated requirements, or missing timestamps. `MACOS-SIGNING.json` binds the signed Mac archives to their checksums and signing identity. Homebrew installs those bytes on macOS and verifies the certificate requirement during install and test. Linux retains source builds.

Optional notarization: set `CCUSAGE_NOTARY_PROFILE` to an existing local `notarytool` credential profile before release. The helper submits each signed Mac binary and requires Accepted status. No notarization credentials are exported. The manifest records whether notarization was performed. Code signing and notarization are distinct; signed-only releases must not be described as notarized.

 The active GitHub account needs push access to both repositories. The source remote defaults to `upstream` when present, otherwise `origin`. The tap defaults to the `../Brew/homebrew-ccusage` directory. To use different local names/locations:

```sh
tools/release 0.1.2 --remote origin --tap /path/to/homebrew-ccusage
```

`CCUSAGE_REMOTE` and `CCUSAGE_TAP` provide the same defaults. The command verifies the source remote and tap repository identities, fetches both repositories, refuses a source branch that is behind/diverged, and fast-forwards the tap when possible. It permits local source commits ahead of GitHub, which are included in the atomic push. It never force-pushes or rewrites an existing tag. A per-checkout release lock prevents concurrent invocations.

Preview without editing versions, running validation, contacting GitHub, or publishing:

```sh
tools/release --dry-run 0.1.2
```

The preview still requires clean local checkouts and installed maintainer tools. It describes the intended steps; it does not assert that remote permissions or CI will pass.

If publication was interrupted after the version commit, resolve the network/CI/tap failure and continue from the same release commit:

```sh
tools/release --resume 0.1.2
```

Resume does not bump the version or create another source commit. It retries the atomic push, waits for the existing workflow, verifies assets, retries the tap update, including a tap commit whose earlier push failed, and waits for Homebrew CI. It does not automatically rerun failed CI: inspect the failure and rerun the job in GitHub when appropriate. A code defect requires a new patch release, not moving a published tag. If the checkout has advanced beyond the tagged release commit, use the lower-level formula publisher below for a tap-only repair.

A local validation failure occurs before any release commit/tag/push, and leaves version edits for inspection. Review and fix them, or restore only Cargo.toml/Cargo.lock before retrying; the helper does not discard changes automatically. Normal exits and signals remove its lock. After a forced kill/crash, check that no release process is running before removing a stale lock at the path reported by the command.

The integration suite uses temporary Git repositories and real atomic/local pushes, with mocked GitHub and validation commands. It tests successful releases, failures, read-only previews, version validation, branch/dirty-tree guards, asset checks, locking, and recovery without contacting GitHub:

```sh
sh tests/release.sh
```

## Lower-level checks and publication

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
python3 -m unittest discover -s tests -p 'signing_test.py'
actionlint .github/workflows/build.yml
```

On macOS, also test updates against a disposable Keychain (no real account credentials are used):

```sh
cargo test --locked native_keychain_updates_existing_item_without_plaintext_fallback -- --ignored
```

This test needs access to the macOS Security service and may fail to create its disposable Keychain inside an agent sandbox. Both macOS CI targets run it explicitly.

Commit and push the release changes to `main`, and wait for the Build and release workflow to pass on all four platforms. Then create and push an annotated release tag:

```sh
git tag -a v0.1.0 -m 'Ccusage v0.1.0'
git push upstream v0.1.0
```

The tagged workflow validates the version, tests and builds native binaries for Apple Silicon macOS, Intel macOS, x86-64 Linux, and ARM64 Linux. It uploads four `.tar.gz` archives and initial `SHA256SUMS` to a private draft only after every build succeeds. Run `tools/finalize-release vX.Y.Z` locally (normally invoked by `tools/release`) to sign both Mac archives, regenerate checksums, verify the uploaded bytes, and publish. Resuming an already public release verifies its existing signatures without replacing assets. It needs only the repository-scoped GitHub Actions token; third-party actions are pinned to commit SHAs. An existing release is not overwritten automatically.

Linux release binaries use glibc: x86-64 is built on Ubuntu 22.04, ARM64 on Ubuntu 24.04. For other compatible Linux distributions, building locally or using the source-building Homebrew formula avoids depending on the release binary's glibc baseline. Windows is not a release target.

## Homebrew tap

Once the tag is public, publish the exact tag and commit to the existing tap checkout:

```sh
tools/formula render v0.1.0 /tmp/ccusage.rb
tools/formula publish v0.1.0 /path/to/homebrew-ccusage
```

`render` reads the template and package version from the tag itself. `publish` checks the clean tap checkout is on main and points to `matthewsinclair/homebrew-ccusage`, verifies the public source tag's commit, renders the formula, and commits/pushes only that formula. It uses the maintainer's existing `gh` and Git authentication; no cross-repository workflow secret is needed. It never force-pushes or updates a tag. Tap README and CI changes are ordinary reviewed Git changes.

The formula installs the signed architecture-specific release archive on macOS, pinned by SHA256 from the verified signing manifest. It builds from the pinned tag and commit using Rust on Linux. It supports Homebrew on macOS and Linux. Users install with:

```sh
brew install matthewsinclair/ccusage/ccusage
ccusage doctor
```

The formula installs the binary and logo, does not require Claude during installation, and does not change account files or shell startup files. Install Claude Code separately for live account operations. Homebrew tests use fixture data and a temporary configuration directory, without credentials or live usage requests.
