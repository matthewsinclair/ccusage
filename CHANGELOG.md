# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] - unreleased

### Added

- Shared mode, turned on by a top-level `claude_config_dir` in the config: sessions keep one Claude config directory and borrow the active profile's login, so each account holds one login and Claude's own config needs none. `ccusage use NAME` copies NAME's account record into the shared `.claude.json` and, on macOS, points launchd at its login; `ccusage run` launches with the shared config and the profile's login; the new `ccusage dir [NAME]` prints a profile's directory for a launcher to export as `CLAUDE_SECURESTORAGE_CONFIG_DIR`; and `ccusage doctor` warns of each way a session can drift from the active profile. With the key unset, nothing changes.

### Fixed

- Renewing a profile's credentials no longer strips `apple-tool:` from Claude Code's own macOS Keychain items, so `security` stops prompting after every renewal. Ccusage reaches the Keychain only through `/usr/bin/security`, and reads each write back from the keychain it found the item in.

### Changed

- Releases are cut with `bin/devbin release`, their Mac archives signed with `bin/devbin macos finalize`, and the Homebrew formula published with `bin/devbin formula publish`. The repository holds no Python, and `bin/devbin check nopython` keeps it that way.
- On macOS, the Homebrew formula installs the signed archive into the keg's `libexec`, with `ccusage` linked into `bin`.

## [0.1.5] - 2026-10-31

### Fixed

- Signing, verification and the Homebrew formula pass the code requirement to macOS's tools as literal text, so it is never read as a file name.
