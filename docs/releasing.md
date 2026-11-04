# Releasing Ccusage

A release is one devbin command, run from a clean checkout on `main` on the maintainer's Mac:

```sh
bin/devbin release 0.1.7
```

It asks once, and everything after that runs under the one yes: the cut, then its one `after:` step, `bin/devbin post-cut`, which signs and notarizes the Mac archives, publishes the release and the Homebrew formula, and pulls this Mac's clone of the tap.

The version lives in `Cargo.toml`, and `bin/devbin release` is the one step that changes it. Tags are stable and annotated, of the form `v0.1.7`, and must match the package version. The release's notes are its section of `CHANGELOG.md`: open `## [0.1.7] - unreleased` there before the cut, and the cut dates it. `bin/.devbin/config.yaml` declares the release in its `release:` and `formula:` blocks, and `bin/devbin release check` prints that declaration.

## 1. The cut

`bin/devbin release 0.1.7` (or `--patch`, `--minor`, `--major`) asks once, then does the rest under that one yes. It stamps the version into `Cargo.toml`, runs `bin/devbin lock-sync` to bring `Cargo.lock` along, dates the note and makes the release commit. Then it runs every leg together: the gates (`check all`, `test all`, `build` and `doctor`) on the release commit here, and the Build and release workflow on the release commit pushed to `candidate/v0.1.7`. Once every leg is green it makes the annotated tag, pushes the commit and the tag to `local` and `upstream`, and deletes the candidate branch. Nothing is tagged while any leg is red.

CI's run on the tag then builds the four archives (Apple silicon and Intel macOS, x86-64 and ARM64 Linux) and makes the release object: a private draft holding them and `SHA256SUMS.ci`. The cut waits for that draft (`object: ci`), then runs `bin/devbin post-cut`.

Preview a cut without stamping, committing, tagging, pushing, or running a gate or `post-cut`:

```sh
bin/devbin release 0.1.7 --dry-run
```

A cut that stops before its tag is continued by running the same command again: each step checks whether it is done already, and a stamp, commit or gate already done reads `already`. `bin/devbin release status v0.1.7` shows where a release stands. A red candidate leaves nothing tagged. Fix what the run names, then run it again. A code defect after the tag needs a new patch release, since a tag is never moved.

At its end devbin prints a note to run `brew upgrade ccusage`, then `ccusage use brew`. Ignore the second half: `ccusage use` selects an account profile, and Ccusage has no dev, opt and brew switch.

## 2. After the tag: `bin/devbin post-cut`

`bin/devbin post-cut` is the cut's one `after:` line. In order, stopping at the first failure, it runs `bin/devbin macos finalize <tag>`, then `bin/devbin formula publish`, then pulls this Mac's clone of the tap, when Homebrew has one, so that `brew upgrade ccusage` sees the release. It is one line, not three, because the release core runs every `after:` line even when one before it failed, and `formula publish` reads the `SHA256SUMS` that only a finished `macos finalize` writes.

It takes its tag from the cut, else `v` and the version in `Cargo.toml`, and refuses a tag that is not an annotated tag here or is not the newest release, since `formula publish` publishes the newest. When a step fails it says so and names the command that resumes the release:

```sh
bin/devbin post-cut v0.1.7
```

A re-run skips what is done: `macos finalize` verifies a release that is public already and changes nothing, and `formula publish` leaves a tap that carries the render as it is. Run by the cut, `formula publish` takes the cut's yes; run by hand, it asks before it pushes.

## 3. Mac signing, notarization and publication

`bin/devbin macos finalize v0.1.7` signs the draft's two Mac archives with the existing `Developer ID Application: Geodica Pty Ltd (76BQL8L47U)` identity in the maintainer's Keychain, and notarizes each signed binary. The private key is never exported or uploaded to GitHub. The command refuses on CI, off macOS, and without that identity in the keychain.

Before it downloads anything for a draft, it reads the notary credentials from the login keychain, under the `keychain` prefix in `packaging/macos/signing`: `geodica.apple.APPLE_ID`, `geodica.apple.APPLE_APP_PASSWORD` and `geodica.apple.APPLE_TEAM_ID`, the credentials Geodica's other releases use. It refuses when one is missing or empty, or when their team is not the signing team. They reach `notarytool` as its arguments, and the command never prints them.

It downloads the draft's archives and `SHA256SUMS.ci`, and checks each archive against those sums. It unpacks a Mac archive only when its listing is exactly `LICENSE`, `README.md`, `ccusage` and `ccusage-logo.svg`, each once and each a regular file, named bare or under a `./` root. Each binary must be its target's architecture. It signs with identifier `com.geodica.ccusage`, the hardened runtime and a secure timestamp. Each signature must then meet the Geodica requirement in `packaging/macos/signing`, carry the timestamp and the runtime flag, and have a designated requirement with no cdhash, and both architectures must share it. Each signed binary is then submitted to Apple's notary service with `notarytool`, and the run stops unless Apple answers Accepted; a refusal names its submission id and nothing else Apple said. It repacks each Mac archive with no owner in its headers, and copies the Linux archives as they are. It writes `SHA256SUMS` over all four and `MACOS-SIGNING.json`, which binds each Mac archive's checksum and designated requirement to the release's tag, identifier and team, and records it as notarized. Then it uploads them, downloads everything again and verifies it all, the notarized record included, removes `SHA256SUMS.ci` and publishes the draft. Any failure leaves the draft private. A bare Mac binary cannot carry a stapled ticket, so Gatekeeper finds the notarization online.

The signing facts (identity, team, identifier and requirement) and the notary credentials' prefix live once, in `packaging/macos/signing`. The formula repeats the requirement, and `tests/macos.bats` fails when the two differ.

Run against a release that is public already, `macos finalize` verifies its checksums, its manifest and both Mac signatures, reads no notary credential, and changes nothing. A run whose publish did not finish is finished by running it again: it takes the draft's own `SHA256SUMS` in place of CI's.

## 4. The Homebrew tap

`bin/devbin formula publish`, the second step of `post-cut`, renders the formula for the newest release from `packaging/homebrew/ccusage.rb`, clones the tap (`matthewsinclair/homebrew-ccusage`) fresh, and commits and pushes the formula once it has a yes. It reads each Mac archive's checksum from the release's `SHA256SUMS`, which only `macos finalize` writes. So a publish run before finalizing has no sums to read, and refuses. A tap that carries the render already is left as it is. Preview a render with `bin/devbin formula render v0.1.7 /tmp/ccusage.rb`.

On macOS the formula installs the signed architecture-specific archive into the keg's `libexec`, links `ccusage` into `bin`, and checks the signing requirement during install and test. On Linux it builds from the release tag and its commit, with Rust as a build dependency. It does not require Claude during installation, and changes no account files or shell startup files. Homebrew tests use fixture data and a temporary configuration directory, without credentials or live usage requests. Users install with:

```sh
brew install matthewsinclair/ccusage/ccusage
ccusage doctor
```

The tap's CI runs `brew install` and `brew test` on macOS and Linux. Its `brew style` and `brew audit --strict` steps are off until devbin's formula render stops writing a header line longer than brew style allows (devbin `lib/cmd/formula:424`); they come back with that fix.

## Checks

```sh
bin/devbin check all
bin/devbin test all
bin/devbin build
```

These run every check `bin/.devbin/config.yaml` holds, as README's Development section describes, and the cut runs them as its gates. `check all` includes actionlint on the workflow, shellcheck on the bats suites, and `check nopython`, which refuses any Python the repository tracks. On macOS, `test all` also runs `test shell`, the bats suites for the project commands in `bin/.devbin/cmd/` (`macos`, `post-cut`, `lock-sync` and `nopython`) against stand-ins for `gh`, `codesign`, `lipo`, `security`, `notarytool`, `brew` and `bin/devbin` itself, so they sign, notarize, upload and publish nothing, and read no real notary credential.

On macOS, `test all` runs `test keychain` too, which tests renewal against a disposable keychain (no real account credentials are used). It creates a keychain under `~/Library/Keychains`, the only place macOS gives one the login keychain's partitioned format, adds an item there as Claude Code does, and checks that ccusage's renewal write keeps `apple-tool:` and `/usr/bin/security` on it and never creates a missing item. The keychain is never added to the search list and is deleted when the test ends. This test needs access to the macOS Security service and may fail to create its disposable keychain inside an agent sandbox. A Keychain dialog while it runs means `security` has lost access to the fixture item, which is the fault this test exists to catch: deny it so the test fails, because Always Allow would change the item's access and could hide the fault. Both macOS CI targets run it explicitly.

Required maintainer tools: Git, Cargo/Rust, authenticated `gh` with push access to both repositories, bash 5 or later for `bin/devbin`, zsh, `jq`, bats, `actionlint`, `shellcheck`, Xcode's `notarytool`, and `intent` for `check critic`. The notary credentials are in the login keychain, as above.

CI runs the same Rust checks per target on all four platforms, on pushes to `main` and `candidate/**`, on tags and on pull requests. It needs only the repository-scoped GitHub Actions token, and its third-party actions are pinned to commit SHAs. Linux release binaries use glibc: x86-64 is built on Ubuntu 22.04, ARM64 on Ubuntu 24.04. For other compatible Linux distributions, building locally or using the source-building Homebrew formula avoids depending on the release binary's glibc baseline. Windows is not a release target.
