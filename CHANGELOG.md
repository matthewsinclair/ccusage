# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] - unreleased

### Fixed

- Renewing a profile's credentials no longer strips `apple-tool:` from Claude Code's own macOS Keychain items, so `security` stops prompting after every renewal. Ccusage reaches the Keychain only through `/usr/bin/security`, and reads each write back from the keychain it found the item in.

### Changed

- Releases are cut with `bin/devbin release`, their Mac archives signed with `bin/devbin macos finalize`, and the Homebrew formula published with `bin/devbin formula publish`. The repository holds no Python, and `bin/devbin check nopython` keeps it that way.
- On macOS, the Homebrew formula installs the signed archive into the keg's `libexec`, with `ccusage` linked into `bin`.

## [0.1.5] - 2026-10-31

### Fixed

- Signing, verification and the Homebrew formula pass the code requirement to macOS's tools as literal text, so it is never read as a file name.
