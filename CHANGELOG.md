# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.9] - 2026-11-13

### Added

- In a checkout, `bin/devbin install` publishes ccusage to an opt prefix, and `bin/devbin use dev`, `use opt` and `use brew` switch the `ccusage` link between the checkout's build, the opt install and Homebrew's, as for every tool released through Homebrew. These are devbin's commands: `ccusage use NAME` still selects an account (ISSUE:0007).
- `ccusage which` names the account to work on now, so no weekly allowance goes unused at its reset: of the accounts below the cap, the one whose weekly window resets soonest, with the rest in rank order and why. `--model NAME` bounds each account by that model's weekly window too, and `--use` makes the choice active. `which --json` is version 1 of a contract other programs read (ST0002).
- `[which] cap` in `config.toml` sets the cap, a percent of the weekly allowance, 95 when absent. Every ccusage before 0.1.9 refuses a config that sets it (ST0002).
- Every successful fetch records each window's changed percent in `history.jsonl` under `XDG_STATE_HOME`, kept for one weekly window, and `doctor` checks it (ST0002).
- `ccusage predict` estimates your burn rate from that history and projects how much of each account's weekly allowance goes unused at its reset if you follow `which`, counting the accounts of the plan most of them record, and says when there is too little history to tell (ST0002).
- The dashboard marks the account `which` would choose with `· next`, on whole, fresh data alone, and `usage --all` marks it too. The dashboard asks again for each account whose usage has passed five minutes, at most once in five minutes, so left open it stays current (ST0002).
- A Herdr mod ships inside ccusage: a panel of each account's weekly use, the next account and predict's projection, from ccusage's own JSON. `ccusage herdr install` puts it into Herdr's mods and `ccusage herdr remove` takes it out, each touching only its own unedited copy without `--force` (ST0002).
- In shared mode, `ccusage use NAME` switches every Claude session, running or new, to NAME from its next request, with no logout, login or restart. The shared config holds one live login, the active profile's, and a switch moves logins between it and the profiles' own stores, so each account keeps one login and Claude Usage lists each once. A switch that was interrupted is finished by the next `use` (ST0003).
- `ccusage use --none` puts the active login back in its profile and leaves the shared config with none, the layout an older ccusage expects (ST0003).
- In shared mode, `ccusage doctor` says where each login is, and fails on a profile with no login or two, a switch left unfinished, or a shared login that is another account's or too long for the Keychain to take in one write. It warns of either Claude Code flag a switch relies on that is off its default, and of each running session launched into another login or config directory (ST0003).
- `usage --json` gives each account's plan as its login records it, in `subscription_type` and `rate_limit_tier` (ST0003).
- An account whose type gives no usage figures, or no limits, shows a note saying so in place of its windows, not an error (ST0003).

### Changed

- In shared mode, a session needs only `CLAUDE_CONFIG_DIR`. `ccusage run` gives Claude no login directory, `ccusage dir` prints none for the profile whose login is in the shared config, and on macOS `use` clears launchd's `CLAUDE_SECURESTORAGE_CONFIG_DIR`, so the launcher line that exported it goes. The first `use` on 0.1.8's layout replaces the shared config's own login only when Claude can no longer renew it, or when its account keeps another login in a profile (ST0003).
- In shared mode, `ccusage login NAME` on the active profile seats its fresh login in the shared config, and `ccusage logout NAME` refuses the profile whose login is in the shared config until a switch moves it out (ST0003).
- In shared mode, `ccusage doctor` reads each credential store, and asks Claude's profile endpoint whose the shared login is unless a usage fetch already verified it (ST0003).
- `ccusage --version` names the tree it runs from and the commit it was built from: `ccusage X.Y.Z (brew, <sha12>)` from Homebrew, `(opt, <sha12>)` from an opt install, and `(dev, <sha12>)` from `bin/devbin build`, with `+dirty` for a build from uncommitted inputs (ISSUE:0007).
- The Homebrew formula installs one tree on macOS and Linux: `bin/ccusage` in the keg's `libexec`, with a manifest. On Linux it builds with `bin/devbin install` instead of `cargo install`, and it no longer installs the logo (ISSUE:0007).
- Release archives hold `bin/ccusage` alone, rooted at the install root, and no longer carry `LICENSE`, `README.md` or the logo (ISSUE:0007).

## [0.1.8] - 2026-11-04

### Changed

- `ccusage help` lists `usage --demo [--json] [--verbose]`, and its text is plain ASCII.
- The Homebrew formula's caveats no longer say macOS may ask for Keychain access when moving to signed releases: since 0.1.6 Ccusage needs no Keychain access of its own.
- README says the macOS release is notarized, and points one login per account under one Claude config at shared mode.

## [0.1.7] - 2026-11-04

### Changed

- A release is one command, `bin/devbin release`. Its one `after:` step, `bin/devbin post-cut`, signs and notarizes the Mac archives, publishes the release and the Homebrew formula, and brings this Mac's clone of the tap up to date; `bin/devbin post-cut <tag>` resumes a release that stopped after its tag.
- Both Mac binaries are notarized as well as signed, with the notary credentials in the maintainer's login keychain, so `CCUSAGE_NOTARY_PROFILE` is gone.

## [0.1.6] - 2026-11-03

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
