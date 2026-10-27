# Releasing Ccusage

The application version is in `Cargo.toml`; keep `Cargo.lock` in sync. Use stable release tags of the form `v0.1.0`. Tags must match the package version. The canonical Homebrew formula template lives at `packaging/homebrew/ccusage.rb`; do not hand-edit version pins in the tap.

## One command

From a clean Ccusage checkout on `main`, with the sibling `homebrew-ccusage` checkout also clean:

```sh
tools/release 0.1.1
```

Pass the new stable version without a leading `v`. The command updates Cargo.toml and Cargo.lock, runs fmt, Clippy, tests, the release build, actionlint, shellcheck, and release-helper integration tests, then commits the version bump. It creates an annotated tag and atomically pushes the source `main` branch and tag. It waits for the exact tag/commit's GitHub Actions run to succeed, checks all four release archives and SHA256SUMS are published, updates/pushes the pinned Homebrew formula, and waits for the tap’s macOS/Linux Homebrew tests. It does not install or uninstall anything locally.

Required maintainer tools: Git, Cargo/Rust, authenticated `gh`, `actionlint`, and `shellcheck`. The active GitHub account needs push access to both repositories. The source remote defaults to `upstream` when present, otherwise `origin`. The tap defaults to the sibling `../homebrew-ccusage` directory. To use different local names/locations:

```sh
tools/release 0.1.1 --remote origin --tap /path/to/homebrew-ccusage
```

`CCUSAGE_REMOTE` and `CCUSAGE_TAP` provide the same defaults. The command verifies the source remote and tap repository identities, fetches both repositories, refuses a source branch that is behind/diverged, and fast-forwards the tap when possible. It permits local source commits ahead of GitHub, which are included in the atomic push. It never force-pushes or rewrites an existing tag. A per-checkout release lock prevents concurrent invocations.

Preview without editing versions, running validation, contacting GitHub, or publishing:

```sh
tools/release --dry-run 0.1.1
```

The preview still requires clean local checkouts and installed maintainer tools. It describes the intended steps; it does not assert that remote permissions or CI will pass.

If publication was interrupted after the version commit, resolve the network/CI/tap failure and continue from the same release commit:

```sh
tools/release --resume 0.1.1
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
actionlint .github/workflows/build.yml
```

Commit and push the release changes to `main`, and wait for the Build and release workflow to pass on all four platforms. Then create and push an annotated release tag:

```sh
git tag -a v0.1.0 -m 'Ccusage v0.1.0'
git push upstream v0.1.0
```

The tagged workflow validates the version, tests and builds native binaries for Apple Silicon macOS, Intel macOS, x86-64 Linux, and ARM64 Linux. It publishes four `.tar.gz` archives and `SHA256SUMS` to a GitHub release only after every build succeeds. It needs only the repository-scoped GitHub Actions token; third-party actions are pinned to commit SHAs. An existing release is not overwritten automatically.

Linux release binaries use glibc: x86-64 is built on Ubuntu 22.04, ARM64 on Ubuntu 24.04. For other compatible Linux distributions, building locally or using the source-building Homebrew formula avoids depending on the release binary's glibc baseline. Windows is not a release target.

## Homebrew tap

Once the tag is public, publish the exact tag and commit to the existing tap checkout:

```sh
tools/formula render v0.1.0 /tmp/ccusage.rb
tools/formula publish v0.1.0 /path/to/homebrew-ccusage
```

`render` reads the template and package version from the tag itself. `publish` checks the clean tap checkout is on main and points to `matthewsinclair/homebrew-ccusage`, verifies the public source tag's commit, renders the formula, and commits/pushes only that formula. It uses the maintainer's existing `gh` and Git authentication; no cross-repository workflow secret is needed. It never force-pushes or updates a tag. Tap README and CI changes are ordinary reviewed Git changes.

The formula builds from source using Homebrew's Rust build dependency and Cargo.lock, following the same release tag plus commit pinning as the Utilz tap. It supports Homebrew on macOS and Linux. Users install with:

```sh
brew install matthewsinclair/ccusage/ccusage
ccusage doctor
```

The formula installs the binary and logo, does not require Claude during installation, and does not change account files or shell startup files. Install Claude Code separately for live account operations. Homebrew tests use fixture data and a temporary configuration directory, without credentials or live usage requests.
