//! Shared mode: Claude sessions keep one config directory, `claude_config_dir`, and
//! borrow the active profile's login through `CLAUDE_SECURESTORAGE_CONFIG_DIR`. Claude
//! Code shows the account its config's `.claude.json` records as `oauthAccount`, not
//! the login's own, so a switch copies that record in from the profile's `.claude.json`,
//! where Claude Code wrote it at `ccusage login`.

use crate::{Result, auth_lock::Lock, config::Store, storage};
use serde_json::Value;
use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
  time::Duration,
};

const RECORD: &str = "oauthAccount";
const SETTINGS: &str = ".claude.json";
/// The variable Claude Code names a login's Keychain item and lock directory after.
pub const LOGIN_DIR: &str = "CLAUDE_SECURESTORAGE_CONFIG_DIR";

/// The oauthAccount record in a `.claude.json`'s text, if it holds one. Pure.
pub fn account(text: &str) -> Result<Option<Value>> {
  let document: Value = serde_json::from_str(text)
    .map_err(|_| "a .claude.json is not valid JSON (contents redacted)")?;
  Ok(
    document
      .get(RECORD)
      .filter(|record| !record.is_null())
      .cloned(),
  )
}

/// A `.claude.json`'s text with `record` as its oauthAccount, and every other key, the
/// keys' order and the final newline as they were. Pure.
pub fn with_account(text: &str, record: &Value) -> Result<String> {
  let mut document: serde_json::Map<String, Value> = serde_json::from_str(text)
    .map_err(|_| "the shared .claude.json is not a JSON object (contents redacted)")?;
  document.insert(RECORD.to_owned(), record.clone());
  let mut updated = serde_json::to_string_pretty(&document)?;
  if text.ends_with('\n') {
    updated.push('\n');
  }
  Ok(updated)
}

/// Whether two oauthAccount records name the same account: by account UUID when both
/// carry one, else by email address. Pure.
pub fn same_account(a: &Value, b: &Value) -> bool {
  let field = |record: &Value, key: &str| {
    record
      .get(key)
      .and_then(Value::as_str)
      .map(str::to_ascii_lowercase)
  };
  match (field(a, "accountUuid"), field(b, "accountUuid")) {
    (Some(a), Some(b)) => a == b,
    _ => field(a, "emailAddress").is_some_and(|email| Some(email) == field(b, "emailAddress")),
  }
}

/// A `.claude.json` Claude Code owns, read whatever its mode: it is not ccusage's
/// private file.
pub fn settings(dir: &Path) -> Result<Option<String>> {
  let path = dir.join(SETTINGS);
  match fs::metadata(&path) {
    Ok(metadata) if metadata.is_file() => Ok(Some(fs::read_to_string(&path)?)),
    Ok(_) => Err(format!("{} is not a regular file", path.display()).into()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
    Err(error) => Err(error.into()),
  }
}

/// The oauthAccount record a profile's own `.claude.json` holds, if any.
pub fn profile_account(store: &Store, name: &str) -> Result<Option<Value>> {
  settings(&store.profile_dir(name)?)?
    .as_deref()
    .map(account)
    .transpose()
    .map(Option::flatten)
}

/// Make `name` the active profile in shared mode: copy its oauthAccount record into the
/// shared `.claude.json` under Claude Code's lock, changing no other key, then select
/// it, and on macOS point launchd at its login too. Running sessions keep their login
/// until they are restarted.
pub fn switch(store: &Store, name: &str) -> Result<()> {
  let shared = store
    .shared_dir()?
    .ok_or("shared mode is off; set claude_config_dir")?;
  let record = profile_account(store, name)?
    .ok_or_else(|| format!("profile {name} records no oauthAccount; run ccusage login {name}"))?;
  let file = shared.join(SETTINGS);
  {
    // Held for a read, a merge and a rename: every session shares this file, and
    // Claude Code writes over a lock held past its 10 s stale time.
    let _lock = lock(&file)?;
    let text = settings(&shared)?.ok_or_else(|| {
      format!(
        "{} does not exist; start Claude once with CLAUDE_CONFIG_DIR={}",
        file.display(),
        shared.display()
      )
    })?;
    // A 0600 file staged beside it and renamed over it, as Claude Code saves its own.
    storage::write_atomic(&file, with_account(&text, &record)?.as_bytes())?;
  }
  store.activate(name)?;
  #[cfg(target_os = "macos")]
  set_launchd(&store.profile_dir(name)?)?;
  Ok(())
}

/// Claude Code's lock on a `.claude.json`, which its saveGlobalConfig takes:
/// proper-lockfile's `<file>.lock` directory, with an mtime heartbeat, stale after
/// 10 s (read from Claude Code 2.1.317's source).
fn lock(file: &Path) -> Result<Lock> {
  let mut path = file.as_os_str().to_owned();
  path.push(".lock");
  Lock::acquire(PathBuf::from(path), Duration::from_secs(10))
}

/// launchctl, for launchd's environment. CCUSAGE_LAUNCHCTL names a stand-in, which the
/// CLI tests always set, so no test reaches launchd.
pub fn launchctl() -> Command {
  Command::new(std::env::var_os("CCUSAGE_LAUNCHCTL").unwrap_or_else(|| "/bin/launchctl".into()))
}

/// Point launchd's CLAUDE_SECURESTORAGE_CONFIG_DIR at `dir`, so Claude Code started
/// from a GUI app follows the active profile as a terminal session does.
#[cfg(target_os = "macos")]
fn set_launchd(dir: &Path) -> Result<()> {
  let status = launchctl()
    .args(["setenv", LOGIN_DIR])
    .arg(dir)
    .status()
    .map_err(|_| "could not run launchctl to point GUI sessions at the active profile")?;
  if !status.success() {
    return Err(
      "launchctl could not set CLAUDE_SECURESTORAGE_CONFIG_DIR; the switch holds for terminal sessions, and GUI ones keep the previous account".into(),
    );
  }
  Ok(())
}

/// launchd's CLAUDE_SECURESTORAGE_CONFIG_DIR, empty when it has none.
#[cfg(target_os = "macos")]
pub fn launchd_dir() -> Result<String> {
  let output = launchctl()
    .args(["getenv", LOGIN_DIR])
    .output()
    .map_err(|_| "could not run launchctl to read launchd's environment")?;
  if !output.status.success() {
    return Err("launchctl could not read CLAUDE_SECURESTORAGE_CONFIG_DIR".into());
  }
  Ok(
    String::from_utf8(output.stdout)?
      .trim_end_matches('\n')
      .to_owned(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn a_switch_changes_only_the_record_keeping_order_and_newline() -> Result<()> {
    let text = "{\n  \"numStartups\": 42,\n  \"oauthAccount\": {\n    \"emailAddress\": \"old@example.com\"\n  },\n  \"theme\": \"dark\"\n}\n";
    let record = json!({"accountUuid": "u-2", "emailAddress": "new@example.com"});
    let updated = with_account(text, &record)?;
    assert_eq!(
      updated,
      "{\n  \"numStartups\": 42,\n  \"oauthAccount\": {\n    \"accountUuid\": \"u-2\",\n    \"emailAddress\": \"new@example.com\"\n  },\n  \"theme\": \"dark\"\n}\n"
    );
    assert!(!with_account(text.trim_end(), &record)?.ends_with('\n'));
    Ok(())
  }

  #[test]
  fn a_settings_file_without_a_record_gains_one_at_its_end() -> Result<()> {
    let updated = with_account(
      "{\"theme\": \"dark\"}",
      &json!({"emailAddress": "a@example.com"}),
    )?;
    let keys: Vec<String> = serde_json::from_str::<serde_json::Map<String, Value>>(&updated)?
      .keys()
      .cloned()
      .collect();
    assert_eq!(keys, ["theme", RECORD]);
    Ok(())
  }

  #[test]
  fn records_are_read_only_when_present_and_not_null() -> Result<()> {
    assert_eq!(
      account("{\"oauthAccount\": {\"emailAddress\": \"a@example.com\"}}")?,
      Some(json!({"emailAddress": "a@example.com"}))
    );
    assert_eq!(account("{\"oauthAccount\": null}")?, None);
    assert_eq!(account("{}")?, None);
    assert!(account("not json").is_err());
    assert!(with_account("[]", &json!({})).is_err());
    Ok(())
  }

  #[test]
  fn accounts_match_by_uuid_then_email() {
    let a = json!({"accountUuid": "U-1", "emailAddress": "a@example.com"});
    assert!(same_account(
      &a,
      &json!({"accountUuid": "u-1", "emailAddress": "other@example.com"})
    ));
    assert!(!same_account(
      &a,
      &json!({"accountUuid": "u-2", "emailAddress": "a@example.com"})
    ));
    assert!(same_account(
      &json!({"emailAddress": "A@example.com"}),
      &json!({"emailAddress": "a@example.com"})
    ));
    assert!(!same_account(&json!({}), &json!({})));
  }

  #[test]
  fn a_switch_keeps_every_float_bit_for_bit() -> Result<()> {
    // Claude Code's .claude.json holds costs and frame timings, floats with every digit
    // significant. A switch rewrites the whole file, so none may move by even an ulp.
    const SAMPLES: usize = 2000;
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut document = serde_json::Map::new();
    for index in 0..SAMPLES {
      // xorshift64, scaled to the costs and timings Claude Code records
      state ^= state << 13;
      state ^= state >> 7;
      state ^= state << 17;
      let value = (state >> 11) as f64 / (1_u64 << 53) as f64 * 1000.0;
      document.insert(format!("value{index}"), json!(value));
    }
    let record = json!({"emailAddress": "a@example.com"});
    document.insert(RECORD.to_owned(), record.clone());
    // serde_json writes each float as its shortest exact text
    let text = serde_json::to_string_pretty(&document)?;
    assert_eq!(with_account(&text, &record)?, text);
    Ok(())
  }
}
