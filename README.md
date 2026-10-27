# Ccusage

Named Claude Code accounts, explicit account selection, and a compact terminal dashboard for subscription usage. Written in Rust, with seven direct dependencies and no async runtime. Requires `claude` on `PATH`; the interactive dashboard requires a terminal.

## Install

```sh
brew install matthewsinclair/ccusage/ccusage
ccusage doctor
```

The Homebrew formula builds from a pinned release tag and commit on macOS and Linux. Rust is a build dependency; Claude Code must be installed separately for account operations. Prebuilt binaries for macOS (ARM64 and Intel) and Linux (ARM64 and x86-64), with checksums, are available on the [releases page](https://github.com/matthewsinclair/ccusage/releases).

## Build and try


```sh
cargo build --release --locked
./target/release/ccusage tui --demo
```

The demo uses fixture data, reads no credentials, and makes no network requests. `ccusage usage --demo` prints the same layout without entering an interactive screen. An installable binary can be built with `cargo install --path . --locked`; ensure Cargo's binary directory is on `PATH`.

## Check the environment

`ccusage doctor` checks that `claude --version` runs from `PATH`, validates the configuration, inspects XDG/profile directory permissions, and checks existing private files are `0600`. It reads no credential contents, does not invoke browser login or fetch usage, and makes no configuration changes. Missing initial setup is informational and includes the next commands; a missing/broken Claude executable, malformed config, inaccessible path, or unsafe private file exits nonzero. Directory permissions other than `0700` and a stale active profile produce warnings. Native macOS/Linux builds run in CI for ARM64 and x86-64; see [releasing](docs/releasing.md) for the release and tap-update process.

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

Claude retains ownership of its credentials: macOS Keychain, or its native `.credentials.json` file. Ccusage reads the access token for usage requests, never prints it, and never copies it into its config or usage cache. It does not perform OAuth refresh or rewrite Claude's tokens. If a token expires, launch that profile in Claude so Claude can refresh it, or use `ccusage login NAME` again. An expired login does not prevent launching Claude through `ccusage run`.

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

## Configuration and XDG paths

Ccusage follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest/) on macOS and Linux. Empty or relative `XDG_*_HOME` overrides are ignored.

| Purpose | Environment variable | Default |
| --- | --- | --- |
| Profile configuration | `XDG_CONFIG_HOME` | `~/.config/ccusage/config.toml` |
| Persistent Claude profile data | `XDG_DATA_HOME` | `~/.local/share/ccusage/profiles/NAME/` |
| Active account selection | `XDG_STATE_HOME` | `~/.local/state/ccusage/active.json` |
| Last successful usage and rate-limit cooldown | `XDG_CACHE_HOME` | `~/.cache/ccusage/NAME.json` |

`ccusage paths` shows resolved paths. `CCUSAGE_CONFIG` or a leading `--config PATH` overrides the config file only; the other XDG locations remain independently controlled. Configurations using the same XDG data/state/cache directories share named profile storage and the active selection. To create a fully independent environment, override all four XDG variables.

Example `config.toml`:

```toml
[profiles.work]
email = "you@company.com"

[profiles.personal]
email = "you@example.com"
```

To register an existing Claude directory without copying it:

```sh
ccusage add existing --dir ~/.claude
```

Or set `config_dir` in a profile. Relative paths resolve against the config file's directory; `~/` is expanded. Duplicate directories, including existing symlink aliases, are rejected. Each default profile contains Claude's own settings, history, plugins, and authentication state; these are kept together because Claude manages that directory as a unit. macOS login secrets normally remain in Keychain. Ccusage's XDG placement does not relocate files Claude itself chooses to store elsewhere.

New directories use mode `0700` and Ccusage files use `0600` on Unix. Existing directory permissions are preserved. Before reading an existing config, state, cache, or fallback credential file, Ccusage requires a regular file with mode exactly `0600` (including no special permission bits); otherwise it refuses with a `chmod 600` instruction before reading contents. It checks the opened file, and does not silently repair user-managed files. Config parser errors redact source contents. These modes do not add encryption to credential files Claude stores on Linux; protect and back up the profile directories accordingly.

## Usage and dashboard

```sh
ccusage usage work
ccusage usage --all
ccusage usage --all --json
ccusage usage work --refresh
ccusage tui
```

`usage` without a name queries the selected account. Every successful fetch is persisted with its retrieval timestamp. Cached results are reused for five minutes. `--refresh` bypasses that freshness cache, but respects an active rate-limit cooldown. A failed request keeps the last successful data indefinitely and displays `as of TIMESTAMP`, its age, and the error. A reset countdown reaching `due` does not imply the stored percentage has refreshed. A successful newer fetch replaces the old data. Login and logout invalidate that profile's cache to avoid displaying usage from an earlier login.

The undocumented `GET https://api.anthropic.com/api/oauth/usage` endpoint reports subscription percentages and reset times. This integration may need updating when Anthropic changes its API. It performs no model inference. It uses HTTPS with certificate verification, rejects redirects, has a 15-second request timeout, and limits response size. HTTP 429 responses persist a cooldown of at least 30 minutes, extended by a longer numeric `Retry-After`. No retry loop runs automatically.

The dashboard fetches accounts in a background worker, keeps the terminal responsive, and updates reset countdowns while open. Use arrows or `j`/`k` to select an account and Enter to open its detail page. The detail page shows only that account, using the same full formatting as `usage --verbose`. Use arrows or Page Up/Down to scroll; Escape, Backspace, or `b` returns to the overview. Press `a` to activate the selected account from either page. Press `r` to refresh all accounts on the overview or just the current account on a detail page. `q` or Ctrl-C quits from either page; Escape also quits from the overview. It does not switch accounts automatically. The colors, sparse layout, and usage bars are inspired by Claude Code. `NO_COLOR` or a leading `--no-color` disables colors; redirected output is also plain. Small terminals omit bars so percentages and reset times remain visible.

Each failed account gets its own error, so `usage --all` continues through the remaining profiles. Usage commands exit nonzero when any account has an error, even if last-known data is displayed. JSON output includes `name`, `fetched_at` (Unix seconds, UTC), `cached`, `snapshot`, and `error`. Raw usage percentages are retained; the terminal rounds them for display. Optional weekly model windows appear when the API returns them.

This initial version targets Claude subscription accounts on macOS and Linux. Console/API-billing and cloud-provider accounts are outside its usage model. Native Windows operation has not been validated. Native macOS and Linux tests and release builds run in the four-platform GitHub Actions matrix.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo build --release --locked
```

Tests use local HTTP fixtures and temporary XDG directories. A fake Claude executable verifies login, selection, logout, environment isolation, and argument/exit-code forwarding without using real accounts. Other checks cover stale-cache fallback, cooldown persistence, config validation, and storage permissions. Real browser login and live usage need verification with an account you explicitly configure.

Code is split into configuration, credential access, storage, usage retrieval, terminal rendering, and CLI coordination. Dependency purposes are documented beside their entries in `Cargo.toml`. Formatting uses two spaces. Unsafe Rust is forbidden; Clippy denies `unwrap` and `expect`. Cargo.lock is retained for reproducible application builds. Intent integration is intentionally deferred.

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
