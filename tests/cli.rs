#![cfg(unix)]

use ccusage::{Result, storage};
use std::{
  fs,
  os::unix::fs::PermissionsExt,
  path::{Path, PathBuf},
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
/// A shared config's .claude.json, as Claude Code lays one out, recording the main
/// config's own account.
const SHARED_SETTINGS: &str = "{\n  \"numStartups\": 3,\n  \"oauthAccount\": {\n    \"emailAddress\": \"main@example.com\"\n  },\n  \"theme\": \"dark\"\n}\n";
struct Sandbox {
  root: PathBuf,
}
impl Sandbox {
  fn new() -> Result<Self> {
    let root = std::env::temp_dir().join(format!(
      "ccusage-test-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    storage::private_dir(&root)?;
    let root = root.canonicalize()?;
    fs::create_dir_all(root.join("bin"))?;
    fs::create_dir_all(root.join("home"))?;
    // Claude fixture: browser login and auth status without touching real accounts.
    let script = r##"#!/bin/sh
umask 077
case "$1 $2" in
  '--version ') /bin/echo 'Claude Code (test fixture)' ;;
  'auth login')
    /bin/mkdir -p "$CLAUDE_CONFIG_DIR"
    # Each profile's login is its own, so a test can tell where each one went.
    /usr/bin/printf '{"claudeAiOauth":{"accessToken":"TEST_TOKEN_%s","expiresAt":9999999999999}}\n' \
      "$(/usr/bin/basename "$CLAUDE_CONFIG_DIR")" > "$CLAUDE_CONFIG_DIR/.credentials.json"
    ;;
  'auth status') /bin/echo '{"email":"test@example.com"}' ;;
  'auth logout') /bin/rm -f "$CLAUDE_CONFIG_DIR/.credentials.json" ;;
  *)
    /bin/echo "dir=$CLAUDE_CONFIG_DIR"
    /bin/echo "api=${ANTHROPIC_API_KEY-unset}"
    /bin/echo "oauth=${CLAUDE_CODE_OAUTH_TOKEN-unset}"
    /bin/echo "storage=${CLAUDE_SECURESTORAGE_CONFIG_DIR-unset}"
    for arg in "$@"; do /bin/echo "arg=$arg"; done
    exit 23 ;;
esac
"##;
    fs::write(root.join("bin/claude"), script)?;
    fs::set_permissions(root.join("bin/claude"), fs::Permissions::from_mode(0o700))?;
    // launchctl stand-in: no test may reach launchd, so every command names this one.
    let launchctl = r#"#!/bin/sh
here=$(/usr/bin/dirname "$0")
/bin/echo "$*" >> "$here/../launchctl.log"
case "$1" in
  setenv) /usr/bin/printf '%s' "$3" > "$here/../launchd-$2" ;;
  unsetenv) /bin/rm -f "$here/../launchd-$2" ;;
  getenv) /bin/cat "$here/../launchd-$2" 2>/dev/null; /bin/echo ;;
esac
"#;
    fs::write(root.join("bin/launchctl"), launchctl)?;
    fs::set_permissions(
      root.join("bin/launchctl"),
      fs::Permissions::from_mode(0o700),
    )?;
    Ok(Self { root })
  }
  fn command(&self) -> Command {
    self.command_at(Path::new(env!("CARGO_BIN_EXE_ccusage")))
  }
  /// The sandbox's environment for `program`, such as a copy of ccusage in an install tree.
  fn command_at(&self, program: &Path) -> Command {
    let mut command = Command::new(program);
    command
      .env("HOME", self.root.join("home"))
      .env("XDG_CONFIG_HOME", self.root.join("config"))
      .env("XDG_DATA_HOME", self.root.join("data"))
      .env("XDG_STATE_HOME", self.root.join("state"))
      .env("XDG_CACHE_HOME", self.root.join("cache"))
      .env("PATH", self.root.join("bin"))
      .env("CCUSAGE_LAUNCHCTL", self.root.join("bin/launchctl"))
      .env_remove("CCUSAGE_CONFIG")
      // A launcher exports these into the shell the tests run from, as Molt does for
      // shared mode, and doctor reports the second; a test sets them itself.
      .env_remove("CLAUDE_CONFIG_DIR")
      .env_remove("CLAUDE_SECURESTORAGE_CONFIG_DIR")
      .env("NO_COLOR", "1");
    command
  }
  /// Shared mode on, at a scratch shared config whose .claude.json has Claude Code's
  /// shape and records another account, and whose credential store holds no login yet;
  /// `work` logged in with its own record.
  fn shared(&self) -> Result<PathBuf> {
    self.setup()?;
    let shared = self.root.join("home/.config/claude");
    fs::create_dir_all(&shared)?;
    fs::write(shared.join(".claude.json"), SHARED_SETTINGS)?;
    storage::write_atomic(&shared.join(".credentials.json"), b"{}")?;
    let config = self.root.join("config/ccusage/config.toml");
    let text = fs::read_to_string(&config)?;
    storage::write_atomic(
      &config,
      format!("claude_config_dir = \"{}\"\n{text}", shared.display()).as_bytes(),
    )?;
    self.login_with_record("work", "u-work", "test@example.com")?;
    Ok(shared)
  }
  /// Log `name` in, as Claude Code would, with an oauthAccount record of its own.
  fn login_with_record(&self, name: &str, uuid: &str, email: &str) -> Result<()> {
    self.ok(&["login", name])?;
    let record = serde_json::json!({"oauthAccount": {"accountUuid": uuid, "emailAddress": email}});
    fs::write(
      self.profile(name).join(".claude.json"),
      serde_json::to_vec(&record)?,
    )?;
    Ok(())
  }
  fn profile(&self, name: &str) -> PathBuf {
    self.root.join("data/ccusage/profiles").join(name)
  }
  fn launchctl(&self) -> Result<String> {
    Ok(fs::read_to_string(self.root.join("launchctl.log")).unwrap_or_default())
  }
  fn run(&self, args: &[&str]) -> Result<Output> {
    Ok(self.command().args(args).output()?)
  }
  fn ok(&self, args: &[&str]) -> Result<String> {
    let output = self.run(args)?;
    if !output.status.success() {
      return Err(String::from_utf8(output.stderr)?.into());
    }
    Ok(String::from_utf8(output.stdout)?)
  }
  fn setup(&self) -> Result<()> {
    self.ok(&["init"])?;
    self.ok(&["add", "work", "--email", "test@example.com"])?;
    Ok(())
  }
  fn cached(&self, name: &str, timestamp: i64, retry_at: i64) -> Result<()> {
    self.cached_usage(
      name,
      timestamp,
      retry_at,
      serde_json::json!({"five_hour":{"utilization":62,"resets_at":null}}),
    )
  }
  /// `name` logged in with the fixture's token, and its usage cached at `timestamp` as
  /// `snapshot`, rate limited until `retry_at`. The profile's email is the fixture's.
  fn cached_usage(
    &self,
    name: &str,
    timestamp: i64,
    retry_at: i64,
    snapshot: serde_json::Value,
  ) -> Result<()> {
    let source = self.root.join("data/ccusage/profiles").join(name);
    use sha2::{Digest, Sha256};
    storage::write_atomic(
      &source.join(".credentials.json"),
      br#"{"claudeAiOauth":{"accessToken":"TEST_TOKEN","expiresAt":9999999999999}}"#,
    )?;
    let credential_id = format!("{:x}", Sha256::digest(b"TEST_TOKEN\0test@example.com"));
    let cache = serde_json::json!({"fetched_at":timestamp,"retry_at":retry_at,"source":source,"credential_id":credential_id,
      "snapshot":snapshot});
    storage::write_atomic(
      &self.root.join(format!("cache/ccusage/{name}.json")),
      &serde_json::to_vec(&cache)?,
    )
  }
  /// The which tests' accounts, with usage cached now and resets relative to it: work,
  /// the soonest reset with headroom, with a Fable window; personal, eligible with a
  /// later reset; spare, at the cap with the soonest reset of all; and broken, with no
  /// login, so unknown.
  fn which_accounts(&self) -> Result<()> {
    self.setup()?;
    for name in ["personal", "spare"] {
      self.ok(&["add", name, "--email", "test@example.com"])?;
    }
    self.ok(&["add", "broken"])?;
    let (now, cap) = (ccusage::now(), ccusage::which::DEFAULT_CAP);
    self.cached_usage("work", now, 0, weekly(cap / 2.0, 2 * DAY, Some(FABLE))?)?;
    self.cached_usage("personal", now, 0, weekly(0.0, 5 * DAY, None)?)?;
    self.cached_usage("spare", now, 0, weekly(cap, DAY, None)?)
  }
}

const DAY: i64 = 86_400;
/// The Fable window's percent in the which tests.
const FABLE: f64 = 10.0;

/// The time `in_seconds` from now, in RFC 3339.
fn reset_in(in_seconds: i64) -> Result<String> {
  Ok(
    (time::OffsetDateTime::now_utc() + time::Duration::seconds(in_seconds))
      .format(&time::format_description::well_known::Rfc3339)?,
  )
}

/// Usage whose weekly window is at `percent` and resets `in_seconds` from now, with a
/// Fable window at `fable`, whose share is half the weekly allowance, when one is given.
fn weekly(percent: f64, in_seconds: i64, fable: Option<f64>) -> Result<serde_json::Value> {
  Ok(resetting(percent, &reset_in(in_seconds)?, fable))
}

/// `weekly`'s usage, resetting at `reset`, so that usage built more than once is on the
/// one window.
fn resetting(percent: f64, reset: &str, fable: Option<f64>) -> serde_json::Value {
  let mut limits =
    vec![serde_json::json!({"kind": "weekly_all", "percent": percent, "resets_at": reset})];
  if let Some(fable) = fable {
    limits.push(
      serde_json::json!({"kind": "weekly_scoped", "percent": fable, "resets_at": reset,
      "scope": {"model": {"display_name": "Fable"}}}),
    );
  }
  serde_json::json!({"limits": limits,
    "weekly_scoped_shares": [{"limit_index": 1, "allowance_percent_of_weekly": 50}]})
}

/// The account each line of the which text is about, in order: the name before its
/// colon, for every line but a "Nothing eligible" one.
fn line_names(text: &str) -> Vec<&str> {
  text
    .lines()
    .filter(|line| !line.starts_with("Nothing eligible"))
    .filter_map(|line| line.split_once(':').map(|(name, _)| name))
    .collect()
}
impl Sandbox {
  /// A usage cache for `name`, as a fetch leaves one: `token` verified as
  /// test@example.com while the login lived in `source`, with a snapshot fetched now.
  /// It lets a switch verify the outgoing login, and usage answer, with no network.
  fn verified(&self, name: &str, source: &Path, token: &str) -> Result<()> {
    use sha2::{Digest, Sha256};
    let credential_id = format!("{:x}", Sha256::digest(format!("{token}\0test@example.com")));
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)?
      .as_secs();
    let cache = serde_json::json!({"fetched_at": now, "retry_at": 0, "source": source,
      "credential_id": credential_id, "verified_email": "test@example.com",
      "snapshot": {"five_hour": {"utilization": 62, "resets_at": null}}});
    storage::write_atomic(
      &self.root.join(format!("cache/ccusage/{name}.json")),
      &serde_json::to_vec(&cache)?,
    )
  }
}

/// The access token of the login a file store in `dir` holds, if any.
fn login_of(dir: &Path) -> Result<Option<String>> {
  let document: serde_json::Value =
    serde_json::from_slice(&fs::read(dir.join(".credentials.json"))?)?;
  Ok(
    document
      .pointer("/claudeAiOauth/accessToken")
      .and_then(serde_json::Value::as_str)
      .map(str::to_owned),
  )
}

impl Drop for Sandbox {
  fn drop(&mut self) {
    if let Err(error) = fs::remove_dir_all(&self.root) {
      eprintln!("could not clean test sandbox: {error}");
    }
  }
}

/// The account names a JSON usage report covers, in report order.
fn names(reports: &serde_json::Value) -> Result<Vec<&str>> {
  reports
    .as_array()
    .ok_or("expected a JSON report array")?
    .iter()
    .map(|report| {
      report["name"]
        .as_str()
        .ok_or_else(|| "report without a name".into())
    })
    .collect()
}

#[test]
fn xdg_paths_and_no_overwrite() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  let config = sandbox.root.join("config/ccusage/config.toml");
  let before = fs::read(&config)?;
  assert!(!sandbox.run(&["init"])?.status.success());
  assert_eq!(fs::read(&config)?, before);
  assert!(!sandbox.root.join("home/.claude").exists());
  let paths = sandbox.ok(&["paths"])?;
  assert!(paths.contains(&sandbox.root.join("state/ccusage").display().to_string()));
  assert_eq!(fs::metadata(config)?.permissions().mode() & 0o777, 0o600);
  let output = sandbox
    .command()
    .args(["paths"])
    .env("XDG_CONFIG_HOME", "relative")
    .env("XDG_DATA_HOME", "")
    .output()?;
  let text = String::from_utf8(output.stdout)?;
  assert!(text.contains("home/.config/ccusage/config.toml"));
  assert!(text.contains("home/.local/share/ccusage"));
  Ok(())
}

// ST0002 AT-02.4: init and add keep the cap out of a config that did not set it.
#[test]
fn init_and_add_write_the_which_cap_only_into_a_config_that_set_it() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  let path = sandbox.root.join("config/ccusage/config.toml");
  let text = fs::read_to_string(&path)?;
  assert!(!text.contains("[which]") && !text.contains("cap"));
  let cap = ccusage::which::DEFAULT_CAP / 2.0;
  storage::write_atomic(&path, format!("{text}\n[which]\ncap = {cap}\n").as_bytes())?;
  sandbox.ok(&["add", "personal"])?;
  let config: ccusage::config::Config = toml::from_str(&fs::read_to_string(&path)?)?;
  assert_eq!(config.cap(), cap);
  assert!(config.profiles.contains_key("personal"));
  // A cap no rule could use is refused by every verb that loads the config.
  storage::write_atomic(&path, format!("{text}\n[which]\ncap = 0\n").as_bytes())?;
  let output = sandbox.run(&["list"])?;
  assert!(!output.status.success());
  assert!(String::from_utf8(output.stderr)?.contains("[which] cap"));
  Ok(())
}

#[test]
fn login_select_run_logout_and_argument_isolation() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  assert!(!sandbox.run(&["use", "work"])?.status.success());
  sandbox.ok(&["login", "work"])?;
  sandbox.ok(&["use", "work"])?;
  let list = sandbox.ok(&["list"])?;
  assert!(list.contains("* work"));
  assert!(list.contains("authenticated"));
  let output = sandbox
    .command()
    .args(["run", "--", "--config", "literal value", "--no-color"])
    .env("ANTHROPIC_API_KEY", "OTHER_ACCOUNT")
    .env("CLAUDE_CODE_OAUTH_TOKEN", "OTHER_ACCOUNT")
    .env("CLAUDE_SECURESTORAGE_CONFIG_DIR", "OTHER_ACCOUNT")
    .output()?;
  assert_eq!(output.status.code(), Some(23));
  let text = String::from_utf8(output.stdout)?;
  assert!(text.contains("data/ccusage/profiles/work"));
  assert!(text.contains("api=unset"));
  assert!(text.contains("oauth=unset"));
  assert!(text.contains("storage=unset"));
  assert!(text.contains("arg=--config\narg=literal value\narg=--no-color"));
  sandbox.ok(&["logout", "work"])?;
  assert!(!sandbox.run(&["run"])?.status.success());
  assert!(sandbox.root.join("state/ccusage/active.json").exists());
  Ok(())
}

#[test]
fn failed_refresh_retains_last_success_even_when_days_old() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  let fetched = ccusage::now() - 86400 * 3;
  sandbox.cached("work", fetched, 0)?;
  let output = sandbox.run(&["usage", "work", "--refresh", "--json"])?;
  assert!(!output.status.success());
  let reports: serde_json::Value = serde_json::from_slice(&output.stdout)?;
  assert_eq!(reports[0]["fetched_at"], fetched);
  assert_eq!(reports[0]["snapshot"]["five_hour"]["utilization"], 62);
  assert!(reports[0]["error"].is_string());
  let text = String::from_utf8(sandbox.run(&["usage", "work"])?.stdout)?;
  assert!(text.contains("as of"));
  assert!(text.contains("62%"));
  let cached: serde_json::Value =
    serde_json::from_slice(&fs::read(sandbox.root.join("cache/ccusage/work.json"))?)?;
  assert_eq!(cached["fetched_at"], fetched);
  Ok(())
}

#[test]
fn verified_cache_and_persistent_backoff_do_not_need_network() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.cached("work", ccusage::now(), 0)?;
  let report: serde_json::Value = serde_json::from_str(&sandbox.ok(&["usage", "work", "--json"])?)?;
  assert!(report[0]["error"].is_null());
  sandbox.cached("work", ccusage::now() - 900, ccusage::now() + 1800)?;
  let output = sandbox.run(&["usage", "work", "--refresh", "--json"])?;
  let report: serde_json::Value = serde_json::from_slice(&output.stdout)?;
  assert!(
    report[0]["error"]
      .as_str()
      .is_some_and(|error| error.contains("rate limited"))
  );
  assert_eq!(report[0]["snapshot"]["five_hour"]["utilization"], 62);
  Ok(())
}

// ST0002 AT-04.4: usage read from the cache, or held back by a cooldown, records nothing.
#[test]
fn a_cache_hit_or_a_cooldown_records_no_usage_history() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  let history = sandbox
    .root
    .join("state/ccusage")
    .join(ccusage::history::FILE);
  sandbox.cached("work", ccusage::now(), 0)?;
  sandbox.ok(&["usage", "work", "--json"])?;
  assert!(!history.exists());
  sandbox.cached("work", ccusage::now() - 900, ccusage::now() + 1800)?;
  sandbox.run(&["usage", "work", "--refresh", "--json"])?;
  assert!(!history.exists());
  Ok(())
}

#[test]
fn all_accounts_continue_after_one_fails_and_directory_change_invalidates_cache() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.ok(&["add", "personal"])?;
  sandbox.cached("work", ccusage::now(), 0)?;
  let output = sandbox.run(&["usage", "--all", "--json"])?;
  assert!(!output.status.success());
  let report: serde_json::Value = serde_json::from_slice(&output.stdout)?;
  assert_eq!(names(&report)?, ["personal", "work"]);
  assert!(report[0]["error"].is_string());
  assert!(report[1]["snapshot"].is_object());
  fs::write(
    sandbox.root.join("config/ccusage/config.toml"),
    "[profiles.work]\nconfig_dir='elsewhere'\n",
  )?;
  let changed = sandbox.run(&["usage", "work", "--json"])?;
  let report: serde_json::Value = serde_json::from_slice(&changed.stdout)?;
  assert!(report[0]["snapshot"].is_null());
  Ok(())
}

#[test]
fn removed_credentials_cannot_display_previously_verified_usage() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.cached("work", ccusage::now(), 0)?;
  fs::remove_file(
    sandbox
      .root
      .join("data/ccusage/profiles/work/.credentials.json"),
  )?;
  let output = sandbox.run(&["usage", "work", "--json"])?;
  let reports: serde_json::Value = serde_json::from_slice(&output.stdout)?;
  assert!(!output.status.success());
  assert!(reports[0]["snapshot"].is_null());
  Ok(())
}

#[test]
fn existing_directory_aliases_and_traversal_are_rejected() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  assert!(!sandbox.run(&["add", "../escape"])?.status.success());
  let dir = sandbox.root.join("shared");
  fs::create_dir(&dir)?;
  let alias = sandbox.root.join("alias");
  std::os::unix::fs::symlink(&dir, &alias)?;
  sandbox.ok(&[
    "add",
    "first",
    "--dir",
    dir.to_str().ok_or("path encoding")?,
  ])?;
  assert!(
    !sandbox
      .run(&[
        "add",
        "second",
        "--dir",
        alias.to_str().ok_or("path encoding")?
      ])?
      .status
      .success()
  );
  Ok(())
}

#[test]
fn data_directory_permissions_leave_existing_parents_unchanged() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let parent = sandbox.root.join("shared-config");
  fs::create_dir(&parent)?;
  fs::set_permissions(&parent, fs::Permissions::from_mode(0o755))?;
  sandbox
    .command()
    .args([
      "--config",
      parent
        .join("settings.toml")
        .to_str()
        .ok_or("path encoding")?,
      "init",
    ])
    .output()?;
  assert_eq!(fs::metadata(parent)?.permissions().mode() & 0o777, 0o755);
  Ok(())
}

#[test]
fn reauthentication_clears_old_usage_even_on_email_mismatch() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.cached("work", ccusage::now(), 0)?;
  fs::write(
    sandbox.root.join("config/ccusage/config.toml"),
    "[profiles.work]\nemail='other@example.com'\n",
  )?;
  let output = sandbox.run(&["login", "work"])?;
  assert!(!output.status.success());
  assert!(!sandbox.root.join("cache/ccusage/work.json").exists());
  Ok(())
}

#[test]
fn verbose_with_json_outputs_only_json() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.cached("work", ccusage::now(), 0)?;
  for args in [
    vec!["--verbose", "usage", "work", "--json"],
    vec!["usage", "work", "--json", "--verbose"],
  ] {
    let output = sandbox.run(&args)?;
    assert!(output.status.success());
    let reports: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(reports[0]["snapshot"]["five_hour"]["utilization"], 62);
    assert!(output.stderr.is_empty());
  }
  Ok(())
}

#[test]
fn verbose_shows_formatted_details_without_json_or_diagnostics() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.cached("work", ccusage::now(), 0)?;
  let cache_file = sandbox.root.join("cache/ccusage/work.json");
  let mut cache: serde_json::Value = serde_json::from_slice(&fs::read(&cache_file)?)?;
  cache["snapshot"] = serde_json::from_str(include_str!("fixtures/usage-with-breakdown.json"))?;
  fs::write(&cache_file, serde_json::to_vec(&cache)?)?;
  let default = sandbox.ok(&["usage", "work"])?;
  assert!(default.contains("Fable"));
  assert!(!default.contains("WEEKLY BREAKDOWN"));
  sandbox.ok(&["use", "work"])?;
  for args in [
    vec!["--verbose", "usage", "work"],
    vec!["usage", "work", "--verbose"],
    vec!["--verbose"],
  ] {
    let output = sandbox.run(&args)?;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout)?;
    assert!(text.contains("WEEKLY BREAKDOWN"));
    assert!(text.contains("EXTRA USAGE"));
    assert!(text.contains("QUOTA WINDOWS"));
    assert!(text.contains("Example model"));
    assert!(text.contains("Not provided"));
    assert!(!text.contains('{'));
    assert!(!text.contains("\"utilization\""));
  }
  Ok(())
}

#[test]
fn bare_ccusage_and_leading_usage_flags_report_as_usage_does() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.ok(&["add", "personal"])?;
  sandbox.cached("work", ccusage::now(), 0)?;
  sandbox.ok(&["use", "work"])?;
  for (bare, explicit) in [
    (vec!["--json"], vec!["usage", "--json"]),
    (vec!["--all", "--json"], vec!["usage", "--all", "--json"]),
  ] {
    let (bare, explicit) = (sandbox.run(&bare)?, sandbox.run(&explicit)?);
    assert_eq!(bare.status.code(), explicit.status.code());
    assert_eq!(bare.stdout, explicit.stdout);
    assert_eq!(bare.stderr, explicit.stderr);
  }
  let active: serde_json::Value = serde_json::from_str(&sandbox.ok(&["--json"])?)?;
  assert_eq!(names(&active)?, ["work"]);
  assert_eq!(active[0]["snapshot"]["five_hour"]["utilization"], 62);
  let all = sandbox.run(&["--all", "--json"])?;
  assert_eq!(
    names(&serde_json::from_slice(&all.stdout)?)?,
    ["personal", "work"]
  );
  Ok(())
}

#[test]
fn help_comes_only_from_help_and_dash_dash_help() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let help = sandbox.ok(&["help"])?;
  assert_eq!(sandbox.ok(&["--help"])?, help);
  let synopsis = help
    .lines()
    .find(|line| line.starts_with("Usage:"))
    .ok_or("help has no Usage: line")?;
  let first_run = sandbox.run(&[])?;
  assert!(!first_run.status.success());
  assert!(first_run.stdout.is_empty());
  sandbox.setup()?;
  sandbox.cached("work", ccusage::now(), 0)?;
  sandbox.ok(&["use", "work"])?;
  let bare = sandbox.ok(&[])?;
  assert!(!bare.contains(synopsis));
  assert!(bare.contains("62%"));
  Ok(())
}

/// The version line this test build of ccusage carries: the one bin/devbin build
/// stamped, or devbin's T4 line for a build it did not stamp, as from cargo test.
fn build_line() -> String {
  match option_env!("DEVBIN_VERSION_LINE") {
    Some(line) if !line.is_empty() => line.to_owned(),
    _ => format!("ccusage {} (unknown, unknown)", env!("CARGO_PKG_VERSION")),
  }
}

#[test]
fn version_prints_the_line_its_build_carries_outside_an_install_tree() -> Result<()> {
  let sandbox = Sandbox::new()?;
  assert_eq!(sandbox.ok(&["--version"])?, format!("{}\n", build_line()));
  assert_eq!(
    sandbox.ok(&["--no-color", "--version"])?,
    format!("{}\n", build_line())
  );
  Ok(())
}

#[test]
fn version_prints_its_trees_manifest_line_in_an_opt_or_brew_tree() -> Result<()> {
  const LINE: &str = "ccusage 0.0.1 (brew, 0123456789ab)";
  let manifest = |tool: &str| {
    format!(
      "# tool: {tool}\n# version: 0.0.1\n# source_commit: 0123456789abcdef0123456789abcdef01234567\n# managed_by: brew\n# line: {LINE}\n"
    )
  };
  let sandbox = Sandbox::new()?;
  // An install tree as bin/devbin install and the formula lay one out, and a link
  // into it from outside, as Homebrew's bin and ~/.local/bin hold
  let tree = sandbox.root.join("tree");
  fs::create_dir_all(tree.join("bin"))?;
  let binary = tree.join("bin/ccusage");
  fs::copy(env!("CARGO_BIN_EXE_ccusage"), &binary)?;
  fs::create_dir_all(sandbox.root.join("links"))?;
  let link = sandbox.root.join("links/ccusage");
  std::os::unix::fs::symlink(&binary, &link)?;
  fs::write(tree.join(".ccusage-install"), manifest("ccusage"))?;
  for program in [&binary, &link] {
    let output = sandbox.command_at(program).arg("--version").output()?;
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout)?, format!("{LINE}\n"));
  }
  // A manifest another tool wrote is no tree of ccusage's
  fs::write(tree.join(".ccusage-install"), manifest("other"))?;
  let output = sandbox.command_at(&binary).arg("--version").output()?;
  assert!(output.status.success());
  assert_eq!(
    String::from_utf8(output.stdout)?,
    format!("{}\n", build_line())
  );
  Ok(())
}

#[test]
fn bare_ccusage_without_config_gives_init_hint_and_help_pointer() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let config = sandbox.root.join("config/ccusage/config.toml");
  let expected = format!(
    "ccusage: no config at {}; run ccusage init, or see ccusage help\n",
    config.display()
  );
  for args in [vec![], vec!["--all", "--json"]] {
    let output = sandbox.run(&args)?;
    assert!(!output.status.success(), "{args:?}");
    assert!(output.stdout.is_empty(), "{args:?}");
    assert_eq!(String::from_utf8(output.stderr)?, expected, "{args:?}");
  }
  assert!(!sandbox.root.join("config").exists());
  Ok(())
}

#[test]
fn unknown_commands_and_options_fail_as_before() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let config = sandbox.root.join("config/ccusage/config.toml");
  let no_config = format!(
    "ccusage: no config at {}; run ccusage init\n",
    config.display()
  );
  let unknown_option = "ccusage: unknown option: --bogus\n";
  let failure = |args: &[&str]| -> Result<String> {
    let output = sandbox.run(args)?;
    assert!(!output.status.success(), "{args:?}");
    assert!(output.stdout.is_empty(), "{args:?}");
    Ok(String::from_utf8(output.stderr)?)
  };
  assert_eq!(failure(&["bogus"])?, no_config);
  assert_eq!(failure(&["--bogus"])?, unknown_option);
  assert_eq!(failure(&["--no-color", "--bogus"])?, unknown_option);
  sandbox.setup()?;
  assert_eq!(
    failure(&["bogus"])?,
    "ccusage: unknown command: bogus; see ccusage help\n"
  );
  assert_eq!(failure(&["--bogus"])?, unknown_option);
  Ok(())
}

#[test]
fn private_config_state_cache_and_credentials_are_mode_600() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.ok(&["login", "work"])?;
  sandbox.ok(&["use", "work"])?;
  sandbox.cached("work", ccusage::now(), 0)?;
  for relative in [
    "config/ccusage/config.toml",
    "state/ccusage/active.json",
    "cache/ccusage/work.json",
    "data/ccusage/profiles/work/.credentials.json",
  ] {
    let path = sandbox.root.join(relative);
    assert_eq!(
      fs::metadata(&path)?.permissions().mode() & 0o7777,
      0o600,
      "{relative}"
    );
    assert!(storage::read_optional(&path)?.is_some());
  }
  for relative in [
    "config/ccusage",
    "state/ccusage",
    "cache/ccusage",
    "data/ccusage/profiles/work",
  ] {
    assert_eq!(
      fs::metadata(sandbox.root.join(relative))?
        .permissions()
        .mode()
        & 0o7777,
      0o700,
      "{relative}"
    );
  }
  Ok(())
}

#[test]
fn insecure_existing_private_files_are_rejected_without_exposing_contents() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.cached("work", ccusage::now(), 0)?;
  let config = sandbox.root.join("config/ccusage/config.toml");
  fs::set_permissions(&config, fs::Permissions::from_mode(0o644))?;
  let output = sandbox.run(&["list"])?;
  assert!(!output.status.success());
  assert!(String::from_utf8(output.stderr)?.contains("mode 0644"));
  fs::set_permissions(&config, fs::Permissions::from_mode(0o600))?;
  let cache = sandbox.root.join("cache/ccusage/work.json");
  fs::set_permissions(cache, fs::Permissions::from_mode(0o644))?;
  assert!(!sandbox.run(&["usage", "work"])?.status.success());
  let credentials = sandbox.root.join("test-credentials.json");
  fs::write(&credentials, "PRIVATE_TEST_SECRET")?;
  for mode in [0o644, 0o660, 0o666, 0o700] {
    fs::set_permissions(&credentials, fs::Permissions::from_mode(mode))?;
    let error = storage::read_optional(&credentials)
      .err()
      .ok_or("expected permission rejection")?
      .to_string();
    assert!(error.contains("chmod 600"));
    assert!(!error.contains("PRIVATE_TEST_SECRET"));
  }
  fs::set_permissions(&credentials, fs::Permissions::from_mode(0o600))?;
  assert!(storage::read_optional(&credentials)?.is_some());
  Ok(())
}

#[test]
fn atomic_writes_replace_broad_permissions_with_mode_600() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let path = sandbox.root.join("private.json");
  fs::write(&path, "old")?;
  fs::set_permissions(&path, fs::Permissions::from_mode(0o644))?;
  storage::write_atomic(&path, b"new")?;
  assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o7777, 0o600);
  assert_eq!(storage::read_optional(&path)?, Some(b"new".to_vec()));
  Ok(())
}

#[test]
fn config_parser_errors_do_not_echo_misplaced_secrets() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  fs::write(
    sandbox.root.join("config/ccusage/config.toml"),
    "[profiles.work]\npassword='MISPLACED_PRIVATE_SECRET'\n",
  )?;
  let output = sandbox.run(&["list"])?;
  assert!(!output.status.success());
  let error = String::from_utf8(output.stderr)?;
  assert!(error.contains("contents redacted"));
  assert!(!error.contains("MISPLACED_PRIVATE_SECRET"));
  Ok(())
}

#[test]
fn doctor_checks_claude_and_explains_uninitialized_setup_without_writes() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let output = sandbox.ok(&["doctor"])?;
  assert!(output.contains("Claude on PATH"));
  assert!(output.contains("ccusage init"));
  assert!(!sandbox.root.join("config").exists());
  assert!(!sandbox.root.join("data").exists());
  Ok(())
}

#[test]
fn doctor_fails_when_claude_is_missing_or_private_files_are_insecure() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  let config = sandbox.root.join("config/ccusage/config.toml");
  fs::set_permissions(config, fs::Permissions::from_mode(0o644))?;
  let output = sandbox.run(&["doctor"])?;
  assert!(!output.status.success());
  assert!(String::from_utf8(output.stdout)?.contains("mode 0644"));
  fs::remove_file(sandbox.root.join("bin/claude"))?;
  let output = sandbox.run(&["doctor"])?;
  assert!(!output.status.success());
  assert!(String::from_utf8(output.stdout)?.contains("Cannot run claude from PATH"));
  Ok(())
}

#[test]
fn doctor_checks_the_usage_history_is_private() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  let history = sandbox
    .root
    .join("state/ccusage")
    .join(ccusage::history::FILE);
  storage::write_atomic(&history, b"")?;
  assert!(
    sandbox
      .ok(&["doctor"])?
      .contains(&format!("Private file: {}", history.display()))
  );
  fs::set_permissions(&history, fs::Permissions::from_mode(0o644))?;
  let output = sandbox.run(&["doctor"])?;
  assert!(!output.status.success());
  let text = String::from_utf8(output.stdout)?;
  assert!(text.contains(ccusage::history::FILE) && text.contains("mode 0644"));
  Ok(())
}

// ST0003 AT-03.5: in shared mode, dir prints no directory for the active account and
// fails saying why, and names a profile whose login is parked in its own directory.
#[test]
fn shared_mode_dir_names_only_a_parked_profile_directory() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.ok(&["login", "work"])?;
  sandbox.ok(&["use", "work"])?;
  let off = sandbox.run(&["dir"])?;
  assert!(!off.status.success() && off.stdout.is_empty());
  assert!(String::from_utf8(off.stderr)?.contains("shared mode is off"));
  let sandbox = Sandbox::new()?;
  sandbox.shared()?;
  sandbox.ok(&["add", "home"])?;
  sandbox.login_with_record("home", "u-home", "home@example.com")?;
  // No active profile: an empty value would mean ~/.claude to Claude Code.
  let none = sandbox.run(&["dir"])?;
  assert!(!none.status.success() && none.stdout.is_empty());
  sandbox.ok(&["use", "work"])?;
  for args in [&["dir"][..], &["dir", "work"]] {
    let active = sandbox.run(args)?;
    assert!(!active.status.success() && active.stdout.is_empty());
    assert!(String::from_utf8(active.stderr)?.contains("reads its login from the shared config"));
  }
  let home = format!("{}\n", sandbox.profile("home").display());
  assert_eq!(sandbox.ok(&["dir", "home"])?, home);
  assert!(!sandbox.run(&["dir", "home", "extra"])?.status.success());
  Ok(())
}

// ST0003 AT-03.2: in shared mode, use seats the profile's login in the shared store,
// copies only its account record into the shared .claude.json, and clears launchd's
// login directory.
#[test]
fn shared_mode_use_seats_the_login_copies_the_record_and_clears_launchd() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  let settings = shared.join(".claude.json");
  // Claude Code saves its .claude.json at 0600, and so does a switch, whatever it found.
  fs::set_permissions(&settings, fs::Permissions::from_mode(0o644))?;
  let said = sandbox.ok(&["use", "work"])?;
  assert!(
    said.contains("Every Claude session uses it from its next request"),
    "{said}"
  );
  assert_eq!(login_of(&shared)?.as_deref(), Some("TEST_TOKEN_work"));
  assert_eq!(login_of(&sandbox.profile("work"))?, None);
  assert_eq!(
    fs::read_to_string(&settings)?,
    "{\n  \"numStartups\": 3,\n  \"oauthAccount\": {\n    \"accountUuid\": \"u-work\",\n    \"emailAddress\": \"test@example.com\"\n  },\n  \"theme\": \"dark\"\n}\n"
  );
  assert_eq!(fs::metadata(&settings)?.permissions().mode() & 0o777, 0o600);
  assert!(!shared.join(".claude.json.lock").exists());
  assert_eq!(
    fs::read_to_string(sandbox.root.join("state/ccusage/active.json"))?,
    "\"work\""
  );
  let launchd = sandbox.launchctl()?;
  if cfg!(target_os = "macos") {
    assert_eq!(launchd, "unsetenv CLAUDE_SECURESTORAGE_CONFIG_DIR\n");
  } else {
    assert!(launchd.is_empty());
  }
  Ok(())
}

// ST0003 AT-03.3: a switch parks the previous account's login in its profile and seats
// the next one's, and list and usage read each login where it lives, before and after.
#[test]
fn shared_mode_switches_move_each_login_and_reads_follow_it() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  sandbox.ok(&["add", "home", "--email", "test@example.com"])?;
  sandbox.login_with_record("home", "u-home", "test@example.com")?;
  sandbox.ok(&["use", "work"])?;
  // A usage fetch leaves work's seated login verified as work's, so the switch can
  // take it out of the shared store; home's usage was fetched while it was parked.
  sandbox.verified("work", &shared, "TEST_TOKEN_work")?;
  sandbox.verified("home", &sandbox.profile("home"), "TEST_TOKEN_home")?;
  let usage = |name: &str| -> Result<serde_json::Value> {
    Ok(serde_json::from_str(
      &sandbox.ok(&["usage", name, "--json"])?,
    )?)
  };
  for name in ["work", "home"] {
    assert_eq!(usage(name)?[0]["cached"], true, "{name} before the switch");
  }
  sandbox.ok(&["use", "home"])?;
  assert_eq!(login_of(&shared)?.as_deref(), Some("TEST_TOKEN_home"));
  assert_eq!(
    login_of(&sandbox.profile("work"))?.as_deref(),
    Some("TEST_TOKEN_work")
  );
  assert_eq!(login_of(&sandbox.profile("home"))?, None);
  assert!(fs::read_to_string(shared.join(".claude.json"))?.contains("\"u-home\""));
  let list = sandbox.ok(&["list"])?;
  for (marker, name) in [('*', "home"), (' ', "work")] {
    let line = format!("{marker} {name:<20} authenticated\n");
    assert!(list.contains(&line), "{list}");
  }
  // Each login now lives elsewhere, so its cache is rebound there with one fetch each.
  sandbox.verified("work", &sandbox.profile("work"), "TEST_TOKEN_work")?;
  sandbox.verified("home", &shared, "TEST_TOKEN_home")?;
  for name in ["work", "home"] {
    assert_eq!(usage(name)?[0]["cached"], true, "{name} after the switch");
  }
  Ok(())
}

// ST0003 AT-03.4: use --none puts the active login back in its profile and leaves the
// shared store with none, the layout an older ccusage reads, and use seats it again;
// --none is refused beside a name and with shared mode off.
#[test]
fn use_none_returns_every_login_to_its_profile() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  sandbox.ok(&["use", "work"])?;
  sandbox.verified("work", &shared, "TEST_TOKEN_work")?;
  assert!(!sandbox.run(&["use", "work", "--none"])?.status.success());
  let said = sandbox.ok(&["use", "--none"])?;
  assert!(said.contains("No account is active"), "{said}");
  assert_eq!(login_of(&shared)?, None);
  assert_eq!(
    login_of(&sandbox.profile("work"))?.as_deref(),
    Some("TEST_TOKEN_work")
  );
  // As 0.1.8 left it: dir names the active profile's own directory again.
  assert_eq!(
    sandbox.ok(&["dir"])?,
    format!("{}\n", sandbox.profile("work").display())
  );
  sandbox.ok(&["use", "work"])?;
  assert_eq!(login_of(&shared)?.as_deref(), Some("TEST_TOKEN_work"));
  let off = Sandbox::new()?;
  off.setup()?;
  off.ok(&["login", "work"])?;
  let refused = off.run(&["use", "--none"])?;
  assert!(!refused.status.success());
  assert!(String::from_utf8(refused.stderr)?.contains("shared mode is off"));
  Ok(())
}

// ST0003 AT-03.6: login of the active profile seats its fresh login in place of the old
// one, and logout of the active profile is refused until another is active.
#[test]
fn shared_mode_login_reseats_and_logout_waits_for_a_switch() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  sandbox.ok(&["use", "work"])?;
  storage::write_atomic(
    &shared.join(".credentials.json"),
    br#"{"claudeAiOauth":{"accessToken":"OLD_work","expiresAt":9999999999999}}"#,
  )?;
  sandbox.ok(&["login", "work"])?;
  assert_eq!(login_of(&shared)?.as_deref(), Some("TEST_TOKEN_work"));
  assert_eq!(login_of(&sandbox.profile("work"))?, None);
  let refused = sandbox.run(&["logout", "work"])?;
  assert!(!refused.status.success());
  assert!(
    String::from_utf8(refused.stderr)?
      .contains("switch first with ccusage use NAME or ccusage use --none")
  );
  assert_eq!(login_of(&shared)?.as_deref(), Some("TEST_TOKEN_work"));
  Ok(())
}

#[test]
fn shared_mode_use_refuses_a_profile_without_an_account_record() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  fs::remove_file(sandbox.profile("work").join(".claude.json"))?;
  let output = sandbox.run(&["use", "work"])?;
  assert!(!output.status.success());
  assert!(String::from_utf8(output.stderr)?.contains("records no oauthAccount"));
  assert_eq!(
    fs::read_to_string(shared.join(".claude.json"))?,
    SHARED_SETTINGS
  );
  assert!(!sandbox.root.join("state/ccusage/active.json").exists());
  assert!(sandbox.launchctl()?.is_empty());
  Ok(())
}

// ST0003 AT-03.7: run gives Claude the shared config and no login directory, whatever
// the shell exported, and switches first to a profile that is not active.
#[test]
fn shared_mode_run_reads_the_shared_login_and_switches_first() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  sandbox.ok(&["add", "home"])?;
  sandbox.login_with_record("home", "u-home", "home@example.com")?;
  sandbox.ok(&["use", "work"])?;
  let output = sandbox
    .command()
    .args(["run"])
    .env("CLAUDE_SECURESTORAGE_CONFIG_DIR", "OTHER_ACCOUNT")
    .output()?;
  assert_eq!(output.status.code(), Some(23));
  let text = String::from_utf8(output.stdout)?;
  assert!(
    text.contains(&format!("dir={}\n", shared.display())),
    "{text}"
  );
  assert!(text.contains("storage=unset\n"), "{text}");
  // A profile that is not active is switched to first, so the record matches the login.
  sandbox.verified("work", &shared, "TEST_TOKEN_work")?;
  let output = sandbox.run(&["run", "home"])?;
  let text = String::from_utf8(output.stdout)?;
  assert!(text.contains("storage=unset\n"), "{text}");
  assert_eq!(login_of(&shared)?.as_deref(), Some("TEST_TOKEN_home"));
  assert_eq!(
    fs::read_to_string(sandbox.root.join("state/ccusage/active.json"))?,
    "\"home\""
  );
  assert!(fs::read_to_string(shared.join(".claude.json"))?.contains("\"u-home\""));
  Ok(())
}

#[test]
fn a_profile_in_the_shared_config_directory_is_refused() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  let config = sandbox.root.join("config/ccusage/config.toml");
  let text = fs::read_to_string(&config)?;
  storage::write_atomic(
    &config,
    format!(
      "{text}\n[profiles.main]\nconfig_dir = \"{}\"\n",
      shared.display()
    )
    .as_bytes(),
  )?;
  let output = sandbox.run(&["list"])?;
  assert!(!output.status.success());
  assert!(String::from_utf8(output.stderr)?.contains("uses the shared Claude config directory"));
  Ok(())
}

#[test]
fn shared_mode_doctor_warns_of_each_way_sessions_can_drift() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  sandbox.ok(&["use", "work"])?;
  let clean = sandbox.ok(&["doctor"])?;
  assert!(clean.contains("OK   Shared mode: sessions use"), "{clean}");
  assert!(
    clean.contains("holds work's login, which every session reads"),
    "{clean}"
  );
  assert!(clean.contains("records work's account"), "{clean}");
  assert!(!clean.contains("WARN"), "{clean}");
  if cfg!(target_os = "macos") {
    assert!(clean.contains("launchd gives GUI-started Claude Code the shared config's login"));
  }
  fs::write(shared.join(".claude.json"), SHARED_SETTINGS)?;
  // 0.1.8's layout: no profile is seated, and the shared config keeps a login.
  fs::remove_file(sandbox.root.join("state/ccusage/seat.json"))?;
  fs::create_dir_all(sandbox.root.join("data/ccusage/profiles/engineering03"))?;
  fs::write(
    sandbox.root.join("launchd-CLAUDE_SECURESTORAGE_CONFIG_DIR"),
    "/elsewhere",
  )?;
  let output = sandbox
    .command()
    .arg("doctor")
    .env("CLAUDE_SECURESTORAGE_CONFIG_DIR", "/nowhere")
    .output()?;
  // Warnings only: sessions still start while any of these stands.
  assert!(output.status.success());
  let drift = String::from_utf8(output.stdout)?;
  for warning in [
    "does not record work's account; run ccusage use work",
    "holds a login of its own",
    "engineering03 has no profile in the config",
    "CLAUDE_SECURESTORAGE_CONFIG_DIR=/nowhere is set in this shell",
  ] {
    assert!(drift.contains(warning), "{warning}: {drift}");
  }
  if cfg!(target_os = "macos") {
    assert!(
      drift.contains("launchd's CLAUDE_SECURESTORAGE_CONFIG_DIR=/elsewhere makes GUI-started Claude Code read that directory's login"),
      "{drift}"
    );
  }
  Ok(())
}

#[test]
fn doctor_is_unchanged_with_shared_mode_off() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  fs::create_dir_all(sandbox.root.join("data/ccusage/profiles/engineering03"))?;
  let output = sandbox
    .command()
    .arg("doctor")
    .env("CLAUDE_SECURESTORAGE_CONFIG_DIR", "/nowhere")
    .output()?;
  let text = String::from_utf8(output.stdout)?;
  assert!(
    !text.contains("Shared mode") && !text.contains("engineering03") && !text.contains("/nowhere"),
    "{text}"
  );
  assert!(sandbox.launchctl()?.is_empty());
  Ok(())
}

#[test]
fn shared_mode_use_waits_for_claude_codes_settings_lock() -> Result<()> {
  use std::{process::Stdio, time::Duration};
  let sandbox = Sandbox::new()?;
  let shared = sandbox.shared()?;
  // Held, as Claude Code's saveGlobalConfig holds it: a fresh lock is not stale for 10 s.
  let lock = shared.join(".claude.json.lock");
  fs::create_dir(&lock)?;
  let mut child = sandbox
    .command()
    .args(["use", "work"])
    .stdout(Stdio::null())
    .spawn()?;
  std::thread::sleep(Duration::from_millis(500));
  assert!(
    child.try_wait()?.is_none(),
    "use went ahead while the lock was held"
  );
  assert_eq!(
    fs::read_to_string(shared.join(".claude.json"))?,
    SHARED_SETTINGS
  );
  fs::remove_dir(&lock)?;
  assert!(child.wait()?.success());
  assert!(fs::read_to_string(shared.join(".claude.json"))?.contains("\"u-work\""));
  Ok(())
}

// ST0002 AT-03.1: the choice with its reason, then the rest in rank order; exit 1 with none.
#[test]
fn which_names_the_soonest_reset_with_headroom_then_ranks_the_rest() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.which_accounts()?;
  let cap = ccusage::which::DEFAULT_CAP;
  let output = sandbox.run(&["which"])?;
  assert!(output.status.success());
  let text = String::from_utf8(output.stdout)?;
  assert_eq!(line_names(&text), ["work", "personal", "spare", "broken"]);
  let lines: Vec<_> = text.lines().collect();
  // The headroom is what is below the cap, beside the percent used.
  let used = cap / 2.0;
  assert!(lines[0].starts_with(&format!(
    "work: {used:.0}% used, {:.0}% below the {cap}% cap, resets in ",
    cap - used
  )));
  assert!(lines[0].ends_with(", soonest reset with headroom"));
  assert!(lines[0].contains(&format!("Fable {FABLE}% used")));
  assert!(lines[1].ends_with(", eligible"));
  assert!(lines[2].ends_with(&format!(", at the {cap}% cap")));
  assert!(lines[3].starts_with("broken: unknown, "));
  assert!(sandbox.ok(&["help"])?.contains("which [--json]"));
  // With nothing eligible, stdout says so first and names the soonest reset, and the
  // reason goes to stderr with exit 1.
  let now = ccusage::now();
  sandbox.cached_usage("work", now, 0, weekly(cap, 2 * DAY, None)?)?;
  sandbox.cached_usage("personal", now, 0, weekly(100.0, 5 * DAY, None)?)?;
  let output = sandbox.run(&["which"])?;
  assert!(!output.status.success());
  let text = String::from_utf8(output.stdout)?;
  let first = text.lines().next().ok_or("the report's first line")?;
  assert!(first.starts_with("Nothing eligible: ") && first.contains("spare's"));
  assert_eq!(line_names(&text), ["spare", "work", "personal", "broken"]);
  assert!(String::from_utf8(output.stderr)?.contains("nothing eligible: "));
  Ok(())
}

// ST0002 AT-03.2: version 1 of the agreed shape, in the text's order.
#[test]
fn which_json_is_version_1_of_the_agreed_shape_in_the_texts_order() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.which_accounts()?;
  let cap = ccusage::which::DEFAULT_CAP;
  let text = sandbox.ok(&["which"])?;
  let json: serde_json::Value = serde_json::from_str(&sandbox.ok(&["which", "--json"])?)?;
  let keys = |value: &serde_json::Value| -> Vec<String> {
    value
      .as_object()
      .map(|object| object.keys().cloned().collect())
      .unwrap_or_default()
  };
  assert_eq!(
    keys(&json),
    [
      "version",
      "now",
      "cap",
      "model",
      "active",
      "choice",
      "activated",
      "soonest",
      "accounts"
    ]
  );
  assert_eq!(json["version"], ccusage::which::view::VERSION);
  assert_eq!(json["cap"], cap);
  assert_eq!(json["model"], serde_json::Value::Null);
  assert_eq!(json["choice"], "work");
  assert_eq!(json["activated"], false);
  assert_eq!(json["soonest"]["name"], "spare");
  let accounts = json["accounts"].as_array().ok_or("an accounts array")?;
  let field = |name: &str| -> Vec<serde_json::Value> {
    accounts
      .iter()
      .map(|account| account[name].clone())
      .collect()
  };
  assert_eq!(field("name"), line_names(&text));
  assert_eq!(field("state"), ["chosen", "eligible", "at_cap", "unknown"]);
  assert_eq!(
    field("rank"),
    [
      serde_json::json!(1),
      serde_json::json!(2),
      serde_json::json!(3),
      serde_json::Value::Null
    ]
  );
  assert_eq!(
    field("cause"),
    [
      serde_json::Value::Null,
      serde_json::Value::Null,
      serde_json::Value::Null,
      serde_json::json!("error")
    ]
  );
  assert_eq!(accounts[0]["headroom"], cap / 2.0);
  assert_eq!(accounts[2]["headroom"], 0.0);
  for account in accounts {
    assert_eq!(
      keys(account),
      [
        "name",
        "state",
        "rank",
        "percent",
        "headroom",
        "resets_at",
        "reason",
        "cause",
        "scoped",
        "session",
        "fetched_at",
        "error"
      ]
    );
  }
  let scoped = &accounts[0]["scoped"][0];
  assert_eq!(keys(scoped), ["model", "percent", "share", "resets_at"]);
  assert_eq!(
    (&scoped["model"], &scoped["percent"]),
    (&serde_json::json!("Fable"), &serde_json::json!(FABLE))
  );
  // Every time is RFC 3339 in UTC, to the second, with a Z.
  for stamp in [
    &json["now"],
    &json["soonest"]["resets_at"],
    &accounts[0]["resets_at"],
    &accounts[0]["fetched_at"],
    &scoped["resets_at"],
  ] {
    let stamp = stamp.as_str().ok_or("a time is a string")?;
    assert!(stamp.ends_with('Z') && !stamp.contains('.'));
    time::OffsetDateTime::parse(stamp, &time::format_description::well_known::Rfc3339)?;
  }
  Ok(())
}

// ST0002 AT-03.3: which --use makes the choice active through use's one function.
#[test]
fn which_use_makes_the_choice_active_and_changes_nothing_when_nothing_is_eligible() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.which_accounts()?;
  let active = sandbox.root.join("state/ccusage/active.json");
  let said = sandbox.ok(&["which", "--use"])?;
  assert!(said.ends_with("Active: work. Future ccusage run launches use this account.\n"));
  assert_eq!(fs::read_to_string(&active)?, "\"work\"");
  assert!(
    sandbox
      .ok(&["which", "--use"])?
      .ends_with("work is already the active account.\n")
  );
  let json: serde_json::Value = serde_json::from_str(&sandbox.ok(&["which", "--use", "--json"])?)?;
  assert_eq!(
    (&json["active"], &json["activated"]),
    (&serde_json::json!("work"), &serde_json::json!(false))
  );
  sandbox.ok(&["use", "personal"])?;
  let json: serde_json::Value = serde_json::from_str(&sandbox.ok(&["which", "--use", "--json"])?)?;
  assert_eq!(
    (&json["active"], &json["activated"]),
    (&serde_json::json!("work"), &serde_json::json!(true))
  );
  // With nothing eligible, --use leaves the active account as it was.
  sandbox.ok(&["use", "personal"])?;
  let (now, cap) = (ccusage::now(), ccusage::which::DEFAULT_CAP);
  sandbox.cached_usage("work", now, 0, weekly(cap, 2 * DAY, None)?)?;
  sandbox.cached_usage("personal", now, 0, weekly(cap, 5 * DAY, None)?)?;
  assert!(!sandbox.run(&["which", "--use"])?.status.success());
  assert_eq!(fs::read_to_string(&active)?, "\"personal\"");
  Ok(())
}

// ST0002 AT-03.4: a model no scoped window carries is refused, naming the models seen.
#[test]
fn which_refuses_a_model_no_scoped_window_carries_and_names_the_models_seen() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.which_accounts()?;
  let output = sandbox.run(&["which", "--model", "Fabel"])?;
  assert!(!output.status.success());
  assert!(output.stdout.is_empty());
  let refusal = String::from_utf8(output.stderr)?;
  assert!(refusal.contains("Fabel") && refusal.contains("the models seen: Fable"));
  // Any case of a model seen is that model, spelled as the usage spells it.
  let json: serde_json::Value =
    serde_json::from_str(&sandbox.ok(&["which", "--model", "fable", "--json"])?)?;
  assert_eq!(json["model"], "Fable");
  assert_eq!(json["choice"], "work");
  // Under a model, the headroom is what the model may still use before the cap.
  let text = sandbox.ok(&["which", "--model", "fable"])?;
  assert!(
    text
      .lines()
      .next()
      .is_some_and(|line| line.contains("% more on Fable before the cap, "))
  );
  Ok(())
}

// hv's ruling 3: any Claude account works, and shows whatever its usage reports.
#[test]
fn which_lists_an_account_without_a_weekly_window_with_what_it_reports_and_never_chooses_it()
-> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.which_accounts()?;
  sandbox.ok(&["add", "free", "--email", "test@example.com"])?;
  let session = 40.0;
  sandbox.cached_usage(
    "free",
    ccusage::now(),
    0,
    serde_json::json!({"limits": [{"kind": "session", "percent": session, "resets_at": reset_in(DAY)?}]}),
  )?;
  let text = sandbox.ok(&["which"])?;
  let free = text
    .lines()
    .find(|line| line.starts_with("free: "))
    .ok_or("free's line")?;
  assert_eq!(
    free,
    format!("free: unknown, session {session}% used, no weekly window in the usage")
  );
  let json: serde_json::Value = serde_json::from_str(&sandbox.ok(&["which", "--json"])?)?;
  let entry = json["accounts"]
    .as_array()
    .ok_or("an accounts array")?
    .iter()
    .find(|account| account["name"] == "free")
    .ok_or("free's entry")?;
  assert_eq!(
    (&entry["state"], &entry["cause"], &entry["rank"]),
    (
      &serde_json::json!("unknown"),
      &serde_json::json!("no_weekly_window"),
      &serde_json::Value::Null
    )
  );
  assert_eq!(entry["session"]["percent"], session);
  assert_eq!(json["choice"], "work");
  Ok(())
}

// ST0002 AT-05.3: predict from a history, what it rests on, and too little said plainly.
#[test]
fn predict_projects_each_accounts_unused_share_from_the_history_and_says_when_it_has_too_little()
-> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.which_accounts()?;
  // With no history yet there is too little, which is said plainly and is no failure.
  let text = sandbox.ok(&["predict"])?;
  assert!(text.starts_with("Too little history to project: "));
  assert!(text.contains("\nbroken: left out, "));
  // A day of history with one rise in it, on work's window.
  let (now, reset) = (ccusage::now(), reset_in(3 * DAY)?);
  let (from, risen) = (10.0, 12.0);
  let history = [(from, now - DAY), (from + risen, now)];
  for (percent, sampled_at) in history {
    let snapshot: ccusage::usage::Snapshot =
      serde_json::from_value(resetting(percent, &reset, None))?;
    ccusage::history::record(
      &sandbox.root.join("state/ccusage"),
      "work",
      &snapshot,
      sampled_at,
    )?;
  }
  let per_hour = risen / ccusage::predict::hours(DAY);
  let text = sandbox.ok(&["predict"])?;
  assert!(text.starts_with(&format!(
    "Burn rate: {per_hour:.1}% of one weekly allowance an hour, from {} samples over {} of history.",
    history.len(),
    ccusage::display::duration(DAY)
  )));
  assert!(text.contains("assuming every account is on one plan"));
  assert!(text.contains("\nbroken: left out, "));
  let json: serde_json::Value = serde_json::from_str(&sandbox.ok(&["predict", "--json"])?)?;
  assert_eq!(json["version"], ccusage::predict::view::VERSION);
  assert_eq!(json["rate"]["samples"], history.len());
  assert_eq!(json["rate"]["hours"], ccusage::predict::hours(DAY));
  assert_eq!(
    json["rate"]["percent_per_hour"],
    ccusage::display::tenth(per_hour)
  );
  let accounts = json["accounts"].as_array().ok_or("an accounts array")?;
  let names: Vec<_> = accounts
    .iter()
    .map(|account| account["name"].clone())
    .collect();
  assert_eq!(names, ["work", "personal", "spare"]);
  let mut unused = 0.0;
  for account in accounts {
    let (projected, left) = (
      account["projected"].as_f64().ok_or("a projected percent")?,
      account["unused"].as_f64().ok_or("an unused percent")?,
    );
    // Each is rounded to 0.1 on its own, so the two meet 100 within that rounding, and
    // within a float's error of it.
    assert!((projected + left - 100.0).abs() <= 0.1 + 1e-9);
    let name = account["name"].as_str().ok_or("a name")?;
    assert!(
      text
        .lines()
        .any(|line| line.starts_with(&format!("{name}: ")) && line.ends_with("% unused"))
    );
    unused += left;
  }
  let total = json["unused"].as_f64().ok_or("a total")?;
  assert!((total - unused).abs() <= 0.1 * accounts.len() as f64);
  assert_eq!(json["left_out"][0]["name"], "broken");
  Ok(())
}
