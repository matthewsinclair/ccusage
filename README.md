# Ccusage

A Rust command-line tool for Claude Code subscription usage across multiple named accounts, with explicit account selection and a compact terminal dashboard. It needs `claude` on `PATH`, and the interactive dashboard needs a terminal.

Ccusage is a command-line take on [Claude Usage](https://github.com/richhickson/claudecodeusage). Thanks to [@richhickson](https://github.com/richhickson) for creating Claude Usage and inspiring this project.

## Install

```sh
brew install matthewsinclair/ccusage/ccusage
ccusage doctor
```

On macOS, Homebrew installs the release binary, signed with Geodica's Developer ID and notarized by Apple, and verifies its signing identity. On Linux, the formula builds from a pinned release tag and commit, with Rust as a build dependency. Claude Code must be installed separately for account operations. Prebuilt binaries for macOS (ARM64 and Intel) and Linux (ARM64 and x86-64), with checksums, are available on the [releases page](https://github.com/matthewsinclair/ccusage/releases).

## Build and try

```sh
cargo build --release --locked
./target/release/ccusage tui --demo
```

The demo uses fixture data, reads no credentials, and makes no network requests. `ccusage usage --demo` prints the same layout without entering an interactive screen. An installable binary can be built with `cargo install --path . --locked`; ensure Cargo's binary directory is on `PATH`.

## Check the environment

`ccusage doctor` checks that `claude --version` runs from `PATH`, validates the configuration, inspects XDG/profile directory permissions, and checks existing private files are `0600`. It reads no credential contents, does not invoke browser login or fetch usage, and makes no configuration changes. Missing initial setup is informational and includes the next commands; a missing/broken Claude executable, malformed config, inaccessible path, or unsafe private file exits nonzero. Directory permissions other than `0700` and a stale active profile produce warnings.

## Set up accounts

```sh
ccusage init
ccusage add work --email you@company.com
ccusage add personal --email you@example.com
ccusage login work
ccusage login personal
ccusage use work
ccusage run
```

`login` runs Claude's browser-based subscription login with an account-specific `CLAUDE_CONFIG_DIR`. The email pre-fills the browser login and is checked against `claude auth status` after login. You still choose and authenticate the account in your browser. No passwords or tokens belong in Ccusage's config.

Claude retains ownership of its credentials: macOS Keychain, or its native `.credentials.json` file. Ccusage automatically renews tokens when fetching usage within one minute of expiry, or retries once after an HTTP 401. Renewed credentials stay in the same native store; private files are written atomically with mode `0600`. No tokens enter Ccusage config, usage caches, output, or process arguments. Renewal preserves unrelated credential metadata and coordinates with Claude Code using its refresh and storage locks. Accounts renew on demand; no background daemon or model request is needed. If renewal is rejected, retry usage first; a revoked or expired refresh token requires `ccusage login NAME`. Network failures retain the last successful usage with its original timestamp. An expired login does not prevent launching Claude through `ccusage run`.

```sh
ccusage list
ccusage use personal
ccusage run work -- --continue
ccusage run -- --model sonnet
ccusage logout personal
```

`use` checks that a login is stored, then changes the selection for future launches through Ccusage. It does not change a running Claude session. On Unix, `run` replaces itself with Claude, preserving signals and exit status. Arguments after `--` pass through unchanged. Inherited API-key, OAuth-token, provider, and API-base-URL environment overrides are removed for the launched Claude process so they do not override the named login. Organization-managed Claude settings still apply.

To make plain `claude` use the selected account, review `ccusage shell zsh` (or `bash`) and add the printed function to your shell startup file:

```sh
claude() { command ccusage run -- "$@"; }
```

The function requires `ccusage` on `PATH`. Without it, plain `claude` continues using its normal configuration. The wrapper uses the default Ccusage config or `CCUSAGE_CONFIG`; a one-off `--config` flag is not embedded into the printed function.

### Shared mode

By default each profile is a whole Claude config directory, so a session's settings, history and projects belong to the profile it runs. Shared mode keeps one Claude config directory for every session instead, and gives each session the active profile's login. Each account then has one login, in its profile, and Claude's own config holds none. Turn it on with a top-level key in the config, before any `[profiles]` table:

```toml
claude_config_dir = "~/.config/claude"
```

In shared mode:

- `ccusage use NAME` selects NAME and copies its account record (`oauthAccount`, which Claude Code shows as the session's account) from NAME's own `.claude.json` into the shared one, changing no other key. It refuses a profile that has no record yet; `ccusage login NAME` writes one. On macOS it also sets launchd's `CLAUDE_SECURESTORAGE_CONFIG_DIR`, so Claude Code started from a GUI app follows the active account too. New sessions use NAME, and running ones switch when they are restarted.
- `ccusage run [NAME]` launches Claude with the shared config and NAME's login, switching to NAME first when it is not the active profile.
- `ccusage dir [NAME]` prints the active or named profile's directory, for a launcher to export. When shared mode is off or no profile is active it prints nothing on stdout and fails, so a launcher can use it unconditionally:

```sh
dir=$(ccusage dir 2>/dev/null) && export CLAUDE_CONFIG_DIR=~/.config/claude CLAUDE_SECURESTORAGE_CONFIG_DIR="$dir"
```

`ccusage login NAME` still signs in within the profile's own directory. `ccusage doctor` warns when the shared config still holds a login of its own, when its account record is not the active profile's, when `CLAUDE_SECURESTORAGE_CONFIG_DIR` or launchd's value names no profile or another one, and when a profile directory has no entry in the config. A profile may not use the shared directory as its own.

## Configuration and XDG paths

Ccusage follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/) on macOS and Linux. Empty or relative `XDG_*_HOME` overrides are ignored.

| Purpose                                       | Environment variable | Default                                 |
| --------------------------------------------- | -------------------- | --------------------------------------- |
| Profile configuration                         | `XDG_CONFIG_HOME`    | `~/.config/ccusage/config.toml`         |
| Persistent Claude profile data                | `XDG_DATA_HOME`      | `~/.local/share/ccusage/profiles/NAME/` |
| Active account selection                      | `XDG_STATE_HOME`     | `~/.local/state/ccusage/active.json`    |
| Last successful usage and rate-limit cooldown | `XDG_CACHE_HOME`     | `~/.cache/ccusage/NAME.json`            |

`ccusage paths` shows resolved paths. `CCUSAGE_CONFIG` or a leading `--config PATH` overrides the config file only; the other XDG locations remain independently controlled. Configurations using the same XDG data/state/cache directories share named profile storage and the active selection. To create a fully independent environment, override all four XDG variables.

Example `config.toml`:

```toml
[profiles.work]
email = "you@company.com"

[profiles.personal]
email = "you@example.com"
```

For multiple accounts, keep each profile's default dedicated directory. Each account has its own persistent login, and switching accounts does not require another browser login. Reauthenticate only when the provider rejects renewal or the login has been revoked.

Ccusage verifies configured profile emails against the authenticated account before accepting a new credential. Usage caches are bound to the verified credential and expected email. A different account cannot silently appear under an old profile name, including when the directory path stays the same. Cached usage still requires access to the stored credential; an unknown or mismatched login withholds usage.

On macOS, Ccusage reaches Claude's Keychain items only through `/usr/bin/security`, the way Claude Code does: it reads an item with `find-generic-password` and writes a renewed login with `security -i`, which takes it on stdin, never as a process argument. Every in-place write resets an item's partition list to the writer's own, so a write made by `security` keeps the `apple-tool:` partition that Claude Code and Claude Usage read with, and none of them prompts after a renewal. Ccusage needs no Keychain access of its own, so signed releases, source builds and development builds behave the same. Renewal only updates an existing item and never creates one, and it refuses a credential document too large for one `security -i` line rather than pass it as an argument. A locked Keychain still asks to be unlocked. If an older Ccusage left an item without `apple-tool:`, allow `security` once; the next renewal restores the partition.

To register an existing Claude directory without copying it, and without another login:

```sh
ccusage add existing --dir ~/.claude
```

Share a directory only if every tool using it keeps the same account signed in: a different login there becomes the login Ccusage sees, and Ccusage rejects usage once its configured email no longer matches. To point a registered profile at another directory, set `config_dir` in its entry:

```toml
[profiles.work]
email = "you@company.com"
config_dir = "~/.claude-work"
```

Relative paths resolve against the config file's directory, and `~/` is expanded. Duplicate directories, including symlink aliases of an existing directory, are rejected. Each default profile holds Claude's own settings, history, plugins and authentication state, which Claude manages as a unit; macOS login secrets normally stay in the Keychain, and Ccusage's XDG placement does not move files Claude stores elsewhere.

On macOS, one account signed in from two config directories has two Keychain logins, and tools that list Keychain logins show both. For one login per account under one Claude config, use [shared mode](#shared-mode) rather than pointing a profile at that config.

New directories use mode `0700` and Ccusage files use `0600` on Unix. Existing directory permissions are preserved. Before reading an existing config, state, cache, or fallback credential file, Ccusage requires a regular file with mode exactly `0600` (including no special permission bits); otherwise it refuses with a `chmod 600` instruction before reading contents. It checks the opened file, and does not silently repair user-managed files. Config parser errors redact source contents. These modes do not add encryption to credential files Claude stores on Linux; protect and back up the profile directories accordingly.

## Usage and dashboard

```sh
ccusage
ccusage --all
ccusage --all --json
ccusage usage work
ccusage usage work --refresh
ccusage tui
```

`ccusage` with no command runs `usage`, and `--all`, `--json` or `--refresh` may take the command's place, so `ccusage --all --json` is `ccusage usage --all --json`. `usage` without a name, like bare `ccusage`, queries the selected account. `ccusage help` and `ccusage --help` print the help. Every successful fetch is persisted with its retrieval timestamp. Cached results are reused for five minutes. `--refresh` bypasses that freshness cache, but respects an active rate-limit cooldown. A failed request keeps the last successful data indefinitely and displays `as of TIMESTAMP`, its age, and the error. A reset countdown reaching `due` does not imply the stored percentage has refreshed. A successful newer fetch replaces the old data. Login and logout invalidate that profile's cache to avoid displaying usage from an earlier login.

The undocumented `GET https://api.anthropic.com/api/oauth/usage` endpoint reports subscription percentages and reset times. This integration may need updating when Anthropic changes its API. It performs no model inference. It uses HTTPS with certificate verification, rejects redirects, has a 15-second request timeout, and limits response size. HTTP 429 responses persist a cooldown of at least 30 minutes, extended by a longer numeric `Retry-After`. No retry loop runs automatically.

The dashboard fetches accounts in a background worker, keeps the terminal responsive, and updates reset countdowns while open. Use arrows or `j`/`k` to select an account and Enter to open its detail page. The detail page shows only that account, using the same full formatting as `usage --verbose`. Use arrows or Page Up/Down to scroll; Escape, Backspace, or `b` returns to the overview. Press `a` to activate the selected account from either page. Press `r` to refresh all accounts on the overview or just the current account on a detail page. `q` or Ctrl-C quits from either page; Escape also quits from the overview. It does not switch accounts automatically. The colors, sparse layout, and usage bars are inspired by Claude Code. `NO_COLOR` or a leading `--no-color` disables colors; redirected output is also plain. Small terminals omit bars so percentages and reset times remain visible.

Each failed account gets its own error, so `usage --all` continues through the remaining profiles. Usage commands exit nonzero when any account has an error, even if last-known data is displayed. JSON output includes `name`, `fetched_at` (Unix seconds, UTC), `cached`, `snapshot`, and `error`. Raw usage percentages are retained; the terminal rounds them for display. Optional weekly model windows appear when the API returns them.

Ccusage targets Claude subscription accounts on macOS and Linux. Console/API-billing and cloud-provider accounts are outside its usage model, and native Windows operation has not been validated.

## Development

```sh
bin/devbin check all
bin/devbin test all
bin/devbin build
```

`bin/devbin` is the vendored devbin launcher, and `bin/.devbin/config.yaml` holds the project's checks. `check all` runs `cargo fmt --check`, Clippy with warnings as errors, actionlint on the workflow, shellcheck on the bats suites, `check nopython` and devbin's read-only gates. `test all` runs the Cargo suites and, on macOS, the bats suites for the project commands in `bin/.devbin/cmd/` (`test shell`) and the disposable-keychain renewal test (`test keychain`). `build` makes the release build. Each gate seals its verdict under `tmp/`, and `bin/devbin help --why` says what is active here. devbin needs bash 5 or later, and `bin/devbin doctor` checks the install. [docs/releasing.md](docs/releasing.md#checks) covers the test stand-ins, the keychain test, the maintainer tools and CI.

Tests use local HTTP fixtures and temporary XDG directories. A fake Claude executable verifies login, selection, logout, environment isolation, and argument/exit-code forwarding without using real accounts. Other checks cover stale-cache fallback, cooldown persistence, config validation, and storage permissions. Real browser login and live usage need verification with an account you explicitly configure.

Code is split into configuration, credential access and renewal (the macOS Keychain in `src/keychain.rs` alone), shared mode, storage, usage retrieval, terminal rendering, environment checks and CLI coordination. Dependency purposes are documented beside their entries in `Cargo.toml`. Formatting uses two spaces. Unsafe Rust is forbidden; Clippy denies `unwrap` and `expect`. Cargo.lock is retained for reproducible application builds.

## Detailed output

```sh
ccusage usage hello --verbose
ccusage --verbose usage --all
ccusage usage hello --json
```

The default output shows the compact plan usage bars and retrieval timestamp. `--verbose` adds all fields from the response as readable sections and aligned values: quota windows, exact percentages and reset times, weekly attribution, credits, model shares, optional fields, and retrieval metadata. Long values wrap to the terminal width. Missing values appear as `Not provided`, booleans as Yes/No, and percentages retain their original precision. Unknown future fields are included too. Both leading `--verbose` and `usage ... --verbose` are supported.

JSON is printed only with `--json`, which takes precedence over `--verbose` and emits only the JSON report. Verbose output does not dump network diagnostics or raw JSON to stderr. Credential-bearing response fields and echoed access tokens are redacted before persistence; formatted details also redact older cached response fields. Normal request failures still show an error and the retained data's original `as of` timestamp.

In the TUI, Enter opens the selected login's own detail page. It shares the verbose CLI renderer, with no information from other accounts included. Escape returns to the overview with the previous selection and scroll position intact. Enter does not activate an account; use `a` for that.

The parser uses the canonical `limits` array when present, including model display names such as Fable, and supports older top-level quota fields as a fallback. Attribution objects such as `seven_day_breakdown` are metadata and do not get interpreted as usage windows. Parser errors identify the offending field without quoting its contents.

## Logo

`design/icons/ccusage-logo.svg` contains three plain horizontal bars on a transparent background: long green at the top, medium amber in the middle, and short red at the bottom.

## Maintainer release commands

From a clean `main` checkout on the maintainer's Mac, `bin/devbin release X.Y.Z` is the whole release: the version stamp, the gates and CI, the tag and the push, then `bin/devbin post-cut`, which signs and notarizes the Mac archives, publishes the release and updates the Homebrew tap. It installs and uninstalls no local packages. [docs/releasing.md](docs/releasing.md) covers the preview, the prerequisites and recovery.
