#![cfg(unix)]

use ccusage::{Result, storage};
use std::{
  fs,
  os::unix::fs::PermissionsExt,
  path::PathBuf,
  process::{Command, Output},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
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
    /bin/cat > "$CLAUDE_CONFIG_DIR/.credentials.json" <<'JSON'
{"claudeAiOauth":{"accessToken":"TEST_TOKEN","expiresAt":9999999999999}}
JSON
    ;;
  'auth status') /bin/echo '{"email":"test@example.com"}' ;;
  'auth logout') /bin/rm -f "$CLAUDE_CONFIG_DIR/.credentials.json" ;;
  *)
    /bin/echo "dir=$CLAUDE_CONFIG_DIR"
    /bin/echo "api=${ANTHROPIC_API_KEY-unset}"
    /bin/echo "oauth=${CLAUDE_CODE_OAUTH_TOKEN-unset}"
    for arg in "$@"; do /bin/echo "arg=$arg"; done
    exit 23 ;;
esac
"##;
    fs::write(root.join("bin/claude"), script)?;
    fs::set_permissions(root.join("bin/claude"), fs::Permissions::from_mode(0o700))?;
    Ok(Self { root })
  }
  fn command(&self) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ccusage"));
    command
      .env("HOME", self.root.join("home"))
      .env("XDG_CONFIG_HOME", self.root.join("config"))
      .env("XDG_DATA_HOME", self.root.join("data"))
      .env("XDG_STATE_HOME", self.root.join("state"))
      .env("XDG_CACHE_HOME", self.root.join("cache"))
      .env("PATH", self.root.join("bin"))
      .env_remove("CCUSAGE_CONFIG")
      .env("NO_COLOR", "1");
    command
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
    let source = self.root.join("data/ccusage/profiles").join(name);
    let cache = serde_json::json!({"fetched_at":timestamp,"retry_at":retry_at,"source":source,
      "snapshot":{"five_hour":{"utilization":62,"resets_at":null}}});
    storage::write_atomic(
      &self.root.join(format!("cache/ccusage/{name}.json")),
      &serde_json::to_vec(&cache)?,
    )
  }
}
impl Drop for Sandbox {
  fn drop(&mut self) {
    if let Err(error) = fs::remove_dir_all(&self.root) {
      eprintln!("could not clean test sandbox: {error}");
    }
  }
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
    .output()?;
  assert_eq!(output.status.code(), Some(23));
  let text = String::from_utf8(output.stdout)?;
  assert!(text.contains("data/ccusage/profiles/work"));
  assert!(text.contains("api=unset"));
  assert!(text.contains("oauth=unset"));
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
fn fresh_cache_and_persistent_backoff_do_not_need_credentials() -> Result<()> {
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

#[test]
fn all_accounts_continue_after_one_fails_and_directory_change_invalidates_cache() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.setup()?;
  sandbox.ok(&["add", "personal"])?;
  sandbox.cached("work", ccusage::now(), 0)?;
  let output = sandbox.run(&["usage", "--all", "--json"])?;
  assert!(!output.status.success());
  let report: serde_json::Value = serde_json::from_slice(&output.stdout)?;
  assert_eq!(report.as_array().map(Vec::len), Some(2));
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
  for args in [
    vec!["--verbose", "usage", "work"],
    vec!["usage", "work", "--verbose"],
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
