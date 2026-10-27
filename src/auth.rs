use crate::{Result, config::Store, storage};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};

// Deliberately no Debug/Serialize: tokens must never enter diagnostics or caches.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
  pub access_token: String,
  pub expires_at: Option<i64>,
}
#[derive(Deserialize)]
struct Credentials {
  #[serde(rename = "claudeAiOauth")]
  oauth: Option<Token>,
}

pub fn keychain_service(dir: &Path) -> String {
  let digest = Sha256::digest(dir.as_os_str().as_encoded_bytes());
  format!(
    "Claude Code-credentials-{:02x}{:02x}{:02x}{:02x}",
    digest[0], digest[1], digest[2], digest[3]
  )
}

fn parse(bytes: &[u8]) -> Result<Token> {
  // Do not include a decoder error that might quote credential contents.
  let credentials: Credentials = serde_json::from_slice(bytes)
    .map_err(|_| "could not decode Claude credentials (contents redacted)")?;
  let token = credentials
    .oauth
    .ok_or("no Claude subscription login; run ccusage login NAME")?;
  if token.access_token.is_empty() {
    return Err("empty Claude access token; log in again".into());
  }
  Ok(token)
}

pub fn read(dir: &Path) -> Result<Token> {
  #[cfg(target_os = "macos")]
  {
    let mut services = vec![keychain_service(dir)];
    if dir == crate::config::home()?.join(".claude") {
      services.push("Claude Code-credentials".to_owned());
    }
    for service in services {
      let output = Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", &service, "-w"])
        .output()?;
      if output.status.success() {
        return parse(&output.stdout);
      }
      // errSecItemNotFound. A locked/denied Keychain must not silently use an old file.
      if output.status.code() != Some(44) {
        return Err(
          "Keychain access failed; unlock it and allow access to the profile's Claude credential"
            .into(),
        );
      }
    }
  }
  let bytes = storage::read_optional(&dir.join(".credentials.json"))?
    .ok_or("not authenticated; run ccusage login NAME")?;
  parse(&bytes)
}

pub fn command(dir: &Path) -> Command {
  let mut command = Command::new("claude");
  command.env("CLAUDE_CONFIG_DIR", dir);
  // An inherited auth override must never make a named profile use another account.
  for key in [
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CODE_USE_FOUNDRY",
  ] {
    command.env_remove(key);
  }
  command
}

pub fn login(store: &Store, name: &str) -> Result<()> {
  let dir = store.profile_dir(name)?;
  storage::private_dir(&dir)?;
  let mut child = command(&dir);
  child.args(["auth", "login", "--claudeai"]);
  if let Some(email) = &store.profile(name)?.email {
    child.args(["--email", email]);
  }
  let status = child.status()?;
  if !status.success() {
    return Err(format!("Claude login exited with {status}").into());
  }
  // Login may have changed the account even if subsequent verification fails.
  let cache = store.cache.join(format!("{name}.json"));
  match std::fs::remove_file(cache) {
    Ok(()) => {}
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
    Err(error) => return Err(error.into()),
  }
  // Validate credential availability without printing any part of it.
  read(&dir)?;
  if let Some(expected) = &store.profile(name)?.email {
    let output = command(&dir).args(["auth", "status"]).output()?;
    if !output.status.success() {
      return Err("login completed but Claude could not verify authentication status".into());
    }
    let status: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let actual = status.get("email").and_then(serde_json::Value::as_str);
    if !actual.is_some_and(|email| email.eq_ignore_ascii_case(expected)) {
      return Err(format!("login saved, but its email does not match profile {name}; check claude auth status using this profile").into());
    }
  }
  println!("Authenticated {name}. Credentials remain in Claude's native store.");
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn credential_errors_do_not_echo_secrets() {
    let error =
      parse(br#"{"claudeAiOauth":{"accessToken":"TOP_SECRET","expiresAt":"TOP_SECRET"}}"#).err();
    assert!(error.is_some_and(|error| !error.to_string().contains("TOP_SECRET")));
  }
  #[test]
  fn directory_keys_are_stable_and_separate() {
    assert_eq!(
      keychain_service(Path::new("/tmp/a")),
      "Claude Code-credentials-d9b741ea"
    );
    assert_ne!(
      keychain_service(Path::new("/tmp/a")),
      keychain_service(Path::new("/tmp/b"))
    );
  }
}
