//! Claude Code's macOS Keychain items, reached only through `/usr/bin/security`, the
//! way Claude Code 2.1.317 reaches them.
//!
//! Every in-place write resets an item's partition list to the writer's own partition.
//! Claude Code and Claude Usage read through `security`, whose partition is
//! `apple-tool:`, so only a write made by `security` leaves the items readable without
//! a password prompt. ccusage therefore never opens these items itself.

use crate::Result;
use sha2::{Digest, Sha256};
use std::{
  io::Write,
  path::{Path, PathBuf},
  process::{Command, Stdio},
};
use unicode_normalization::UnicodeNormalization;

const SECURITY: &str = "/usr/bin/security";
/// `security` exits with the low byte of errSecItemNotFound (-25300).
const ITEM_NOT_FOUND: i32 = 44;
/// The longest command line, newline included, that Claude Code 2.1.317 sends to
/// `security -i`; `security` splits longer lines.
const STDIN_LIMIT: usize = 4032;
/// The account Claude Code files an item under when the login name will not do.
const FALLBACK_ACCOUNT: &str = "claude-code-user";

/// Claude Code's item name for a config directory: the first four bytes of the SHA-256
/// of the path's NFC form, as Claude Code hashes `CLAUDE_CONFIG_DIR`.
pub fn service(dir: &Path) -> String {
  let path: String = dir.to_string_lossy().nfc().collect();
  let digest = Sha256::digest(path.as_bytes());
  format!(
    "Claude Code-credentials-{:02x}{:02x}{:02x}{:02x}",
    digest[0], digest[1], digest[2], digest[3]
  )
}

/// The account Claude Code files its items under: `$USER`, else the login name.
pub fn account() -> String {
  match std::env::var_os("USER").filter(|user| !user.is_empty()) {
    Some(user) => account_for(user.to_str()),
    None => account_for(login_name().as_deref()),
  }
}

/// Claude Code replaces a name it does not accept with its fallback account, and so
/// does this, so that the name matches the item Claude Code wrote.
fn account_for(user: Option<&str>) -> String {
  match user {
    Some(user)
      if !user.is_empty()
        && user
          .bytes()
          .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')) =>
    {
      user.to_owned()
    }
    _ => FALLBACK_ACCOUNT.to_owned(),
  }
}

/// The effective user's name, as Node's `os.userInfo()` gives it to Claude Code. Claude
/// Code uses its fallback account when that lookup fails, so a failure here does too.
fn login_name() -> Option<String> {
  let output = Command::new("/usr/bin/id")
    .arg("-un")
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .output()
    .ok()
    .filter(|output| output.status.success())?;
  String::from_utf8(output.stdout)
    .ok()
    .map(|name| name.trim_end().to_owned())
}

/// One of Claude Code's credential items, named by its service and account. `keychain`
/// confines every lookup to one keychain file instead of the user's search list.
pub struct Item<'a> {
  pub service: &'a str,
  pub account: &'a str,
  pub keychain: Option<&'a Path>,
}

impl Item<'_> {
  /// The credential document, or None when no such item exists. Any other failure,
  /// such as a locked keychain or refused access, is an error and never a missing item.
  pub fn read(&self) -> Result<Option<Vec<u8>>> {
    let output = self
      .find(true)
      .output()
      .map_err(|_| "could not run /usr/bin/security to read Claude's Keychain item")?;
    match output.status.code() {
      Some(0) => Ok(Some(password(&output.stdout))),
      Some(ITEM_NOT_FOUND) => Ok(None),
      _ => Err("Keychain access failed; unlock it and allow access to Claude's item".into()),
    }
  }

  /// Check, before a renewal consumes its refresh token, that the item exists and that
  /// a document this size fits on `security -i`'s line.
  pub fn check(&self, password: &[u8]) -> Result<()> {
    update_line(self.service, self.account, &self.locate()?, password).map(|_| ())
  }

  /// Replace the item's password in place. `add-generic-password -U` adds a missing
  /// item, so this finds the item by its attributes first and refuses when it is
  /// absent; the password then travels on `security -i`'s stdin, never in argv.
  pub fn update(&self, password: &[u8]) -> Result<()> {
    let line = update_line(self.service, self.account, &self.locate()?, password)?;
    let mut child = writer()
      .spawn()
      .map_err(|_| "could not run /usr/bin/security to save renewed Claude credentials")?;
    let sent = child
      .stdin
      .take()
      .ok_or("could not reach /usr/bin/security's input")
      .and_then(|mut stdin| {
        stdin
          .write_all(line.as_bytes())
          .map_err(|_| "could not send renewed credentials to /usr/bin/security")
      });
    let status = child
      .wait()
      .map_err(|_| "could not wait for /usr/bin/security")?;
    sent?;
    if !status.success() {
      return Err(
        "could not save renewed Claude credentials in Keychain; unlock it and retry".into(),
      );
    }
    // security -i can report success for a write it made somewhere else.
    if self.read()?.as_deref() != Some(password) {
      return Err("Claude's Keychain item did not take the renewed credentials".into());
    }
    Ok(())
  }

  /// The keychain file holding the item, found from its attributes alone.
  fn locate(&self) -> Result<PathBuf> {
    let output = self
      .find(false)
      .output()
      .map_err(|_| "could not run /usr/bin/security to find Claude's Keychain item")?;
    match output.status.code() {
      Some(0) => keychain_of(&output.stdout)
        .ok_or_else(|| "could not tell which keychain holds Claude's item".into()),
      Some(ITEM_NOT_FOUND) => {
        Err("Claude Keychain item is missing; credentials were not overwritten".into())
      }
      _ => {
        Err("could not open Claude's Keychain item for renewal; unlock Keychain and retry".into())
      }
    }
  }

  /// `find-generic-password` for this item: its password with `-w`, else its
  /// attributes alone, which decrypts nothing.
  fn find(&self, password: bool) -> Command {
    let mut command = Command::new(SECURITY);
    command.args(["find-generic-password", "-a", self.account]);
    if password {
      command.arg("-w");
    }
    command
      .args(["-s", self.service])
      .stdin(Stdio::null())
      .stderr(Stdio::null());
    if let Some(keychain) = self.keychain {
      command.arg(keychain);
    }
    command
  }
}

/// `security -i`, which reads its commands, password included, from stdin. Its output
/// can echo a line it could not parse, so it is discarded.
fn writer() -> Command {
  let mut command = Command::new(SECURITY);
  command
    .arg("-i")
    .stdin(Stdio::piped())
    .stdout(Stdio::null())
    .stderr(Stdio::null());
  command
}

/// The `security -i` line that replaces the item's password in `keychain`, refused
/// when it would not fit on one line or a field would break security's quoting.
fn update_line(service: &str, account: &str, keychain: &Path, password: &[u8]) -> Result<String> {
  let keychain = keychain
    .to_str()
    .ok_or("the keychain holding Claude's item has a path that is not UTF-8")?;
  if [service, account, keychain]
    .iter()
    .any(|field| field.contains(['"', '\\']) || field.chars().any(char::is_control))
  {
    return Err("Claude's Keychain item cannot be named safely to /usr/bin/security".into());
  }
  let hex: String = password.iter().map(|byte| format!("{byte:02x}")).collect();
  let line = format!(
    "add-generic-password -U -a \"{account}\" -s \"{service}\" -X \"{hex}\" \"{keychain}\"\n"
  );
  if line.len() > STDIN_LIMIT {
    return Err("Claude's credentials are too large to update through security -i, and ccusage never passes them as a process argument".into());
  }
  Ok(line)
}

/// `find-generic-password -w` prints the password and a newline, as hex when any byte
/// would not print. A credential document is JSON, so it is never all hex digits.
fn password(stdout: &[u8]) -> Vec<u8> {
  let text = stdout.strip_suffix(b"\n").unwrap_or(stdout);
  let hex =
    !text.is_empty() && text.len().is_multiple_of(2) && text.iter().all(u8::is_ascii_hexdigit);
  hex
    .then(|| unhex(text))
    .flatten()
    .unwrap_or_else(|| text.to_vec())
}

fn unhex(text: &[u8]) -> Option<Vec<u8>> {
  text
    .chunks(2)
    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
    .collect()
}

/// The keychain file `find-generic-password` names first, for the item it found.
fn keychain_of(stdout: &[u8]) -> Option<PathBuf> {
  let path = std::str::from_utf8(stdout)
    .ok()?
    .lines()
    .next()?
    .strip_prefix("keychain: \"")?
    .strip_suffix('"')?;
  Some(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn directory_keys_are_stable_separate_and_nfc() {
    assert_eq!(
      service(Path::new("/tmp/a")),
      "Claude Code-credentials-d9b741ea"
    );
    assert_ne!(service(Path::new("/tmp/a")), service(Path::new("/tmp/b")));
    // Claude Code hashes the NFC form, so both spellings of é name the same item.
    let composed = "/tmp/caf\u{e9}";
    let decomposed = "/tmp/cafe\u{301}";
    assert_ne!(composed, decomposed);
    assert_eq!(service(Path::new(decomposed)), service(Path::new(composed)));
    let digest = Sha256::digest(composed.as_bytes());
    assert!(service(Path::new(decomposed)).ends_with(&format!(
      "{:02x}{:02x}{:02x}{:02x}",
      digest[0], digest[1], digest[2], digest[3]
    )));
  }

  #[test]
  fn accounts_follow_claude_code_naming() {
    for name in ["matts", "first.last", "a_b-c9"] {
      assert_eq!(account_for(Some(name)), name);
    }
    for name in [
      None,
      Some(""),
      Some("two words"),
      Some("me@host"),
      Some("jos\u{e9}"),
    ] {
      assert_eq!(account_for(name), FALLBACK_ACCOUNT, "{name:?}");
    }
  }

  #[test]
  fn passwords_print_as_text_or_hex() {
    let document = br#"{"claudeAiOauth":{"accessToken":"T"}}"#;
    let mut text = document.to_vec();
    text.push(b'\n');
    assert_eq!(password(&text), document);
    let hex: String = document
      .iter()
      .map(|byte| format!("{byte:02x}\n"))
      .collect();
    let mut printed = hex.replace('\n', "").into_bytes();
    printed.push(b'\n');
    assert_eq!(password(&printed), document);
  }

  #[test]
  fn update_lines_carry_the_password_only_on_stdin() -> Result<()> {
    let keychain = Path::new("/Users/fixture/Library/Keychains/login.keychain-db");
    let line = update_line("service", "account", keychain, b"{}")?;
    assert_eq!(
      line,
      "add-generic-password -U -a \"account\" -s \"service\" -X \"7b7d\" \"/Users/fixture/Library/Keychains/login.keychain-db\"\n"
    );
    let args: Vec<_> = writer().get_args().map(|arg| arg.to_owned()).collect();
    assert_eq!(args, ["-i"]);
    for field in ["quote\"d", "back\\slash", "new\nline"] {
      assert!(update_line(field, "account", keychain, b"{}").is_err());
      assert!(update_line("service", field, keychain, b"{}").is_err());
    }
    Ok(())
  }

  #[test]
  fn update_lines_over_the_stdin_limit_are_refused_without_the_secret() -> Result<()> {
    let keychain = Path::new("/Users/fixture/Library/Keychains/login.keychain-db");
    let line = |password: &[u8]| update_line("service", "account", keychain, password);
    let overhead = line(b"")?.len();
    let per_byte = line(b"x")?.len() - overhead;
    let fits = vec![b'x'; (STDIN_LIMIT - overhead) / per_byte];
    assert!(line(&fits)?.len() <= STDIN_LIMIT);
    let mut over = fits.clone();
    over.push(b'x');
    let error = line(&over)
      .err()
      .ok_or("expected the oversized line to be refused")?;
    assert!(!error.to_string().contains("7878"));
    Ok(())
  }

  #[test]
  fn found_items_name_their_keychain() {
    let printed = b"keychain: \"/Users/fixture/Library/Keychains/login.keychain-db\"\nversion: 512\nclass: \"genp\"\n";
    assert_eq!(
      keychain_of(printed),
      Some(PathBuf::from(
        "/Users/fixture/Library/Keychains/login.keychain-db"
      ))
    );
    assert_eq!(keychain_of(b"version: 512\n"), None);
  }
}
