# Releasing Ccusage

The application version is in `Cargo.toml`; keep `Cargo.lock` in sync. Use stable release tags of the form `v0.1.0`. Tags must match the package version. The canonical Homebrew formula template lives at `packaging/homebrew/ccusage.rb`; do not hand-edit version pins in the tap.

## Checks and publication

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
