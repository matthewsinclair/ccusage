#[cfg(target_os = "macos")]
use crate::keychain::Item;
use crate::{Result, config::Store, storage};
use serde::Deserialize;
use std::{
  path::{Path, PathBuf},
  process::Command,
};

// Deliberately no Debug/Serialize: tokens must never enter diagnostics or caches.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
  pub access_token: String,
  pub expires_at: Option<i64>,
  refresh_token: Option<String>,
  client_id: Option<String>,
}
#[derive(Deserialize)]
struct Credentials {
  #[serde(rename = "claudeAiOauth")]
  oauth: Option<Token>,
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

// Retain the whole document and the exact source so refresh preserves unrelated
// metadata and never copies a Keychain credential into a plaintext fallback.
struct Stored {
  document: serde_json::Value,
  token: Token,
  location: Location,
}

/// Where Claude Code keeps a config directory's credential document: a Keychain item
/// on macOS, else the directory's `.credentials.json`. Every read and write of a
/// login, a renewal's and a move's alike, goes through it.
pub(crate) enum Location {
  File(PathBuf),
  #[cfg(target_os = "macos")]
  Keychain {
    service: String,
    account: String,
    /// One keychain file to confine every lookup to, as a test's disposable one;
    /// None searches the user's keychains, as Claude Code does.
    keychain: Option<PathBuf>,
  },
}

impl Location {
  #[cfg(target_os = "macos")]
  fn item<'a>(service: &'a str, account: &'a str, keychain: &'a Option<PathBuf>) -> Item<'a> {
    Item {
      service,
      account,
      keychain: keychain.as_deref(),
    }
  }

  /// The document stored here, or None when the item or the file does not exist.
  pub(crate) fn read(&self) -> Result<Option<Vec<u8>>> {
    match self {
      Self::File(path) => storage::read_optional(path),
      #[cfg(target_os = "macos")]
      Self::Keychain {
        service,
        account,
        keychain,
      } => Self::item(service, account, keychain).read(),
    }
  }

  /// Check, before anything is written, that `bytes` can be saved here: for a Keychain
  /// item, that it exists and that a document this size fits on `security -i`'s line.
  pub(crate) fn check(&self, bytes: &[u8]) -> Result<()> {
    match self {
      Self::File(_) => Ok(()),
      #[cfg(target_os = "macos")]
      Self::Keychain {
        service,
        account,
        keychain,
      } => Self::item(service, account, keychain).check(bytes),
    }
  }

  /// Save `bytes` here and read them back, so a write that did not land is an error.
  pub(crate) fn write(&self, bytes: &[u8]) -> Result<()> {
    match self {
      Self::File(path) => {
        storage::write_atomic(path, bytes)?;
        if storage::read_optional(path)?.as_deref() != Some(bytes) {
          return Err("Claude's credentials file did not take the new document".into());
        }
        Ok(())
      }
      // The update reads the item back from the keychain it found it in.
      #[cfg(target_os = "macos")]
      Self::Keychain {
        service,
        account,
        keychain,
      } => Self::item(service, account, keychain).update(bytes),
    }
  }
}

/// The store Claude Code reads `dir`'s login from, with its document: on macOS the
/// first of the directory's Keychain items that exists, else its credentials file.
/// None when there is neither. The document may hold no login.
pub(crate) fn load(dir: &Path) -> Result<Option<(Location, Vec<u8>)>> {
  #[cfg(target_os = "macos")]
  {
    let account = crate::keychain::account();
    let mut services = vec![crate::keychain::service(dir)];
    if dir == crate::config::home()?.join(".claude") {
      services.push("Claude Code-credentials".to_owned());
    }
    for service in services {
      let location = Location::Keychain {
        service,
        account: account.clone(),
        keychain: None,
      };
      if let Some(bytes) = location.read()? {
        return Ok(Some((location, bytes)));
      }
    }
  }
  let location = Location::File(dir.join(".credentials.json"));
  Ok(location.read()?.map(|bytes| (location, bytes)))
}

impl Stored {
  fn decode(bytes: &[u8], location: Location) -> Result<Self> {
    let token = parse(bytes)?;
    let document = serde_json::from_slice(bytes)
      .map_err(|_| "could not decode Claude credentials (contents redacted)")?;
    Ok(Self {
      document,
      token,
      location,
    })
  }

  fn bytes(&self) -> Result<Vec<u8>> {
    serde_json::to_vec(&self.document).map_err(|_| "could not encode Claude credentials".into())
  }

  // A missing item, refused access or an oversized document must surface before the
  // renewal consumes a rotating refresh token.
  fn prepare_write(&self) -> Result<()> {
    self.location.check(&self.bytes()?)
  }

  fn save(&self) -> Result<()> {
    self.location.write(&self.bytes()?)
  }
}

const NOT_AUTHENTICATED: &str = "not authenticated; run ccusage login NAME";

fn read_stored(dir: &Path) -> Result<Stored> {
  let (location, bytes) = load(dir)?.ok_or(NOT_AUTHENTICATED)?;
  Stored::decode(&bytes, location)
}

#[cfg(test)]
fn read_file(dir: &Path) -> Result<Stored> {
  let location = Location::File(dir.join(".credentials.json"));
  let bytes = location.read()?.ok_or(NOT_AUTHENTICATED)?;
  Stored::decode(&bytes, location)
}

pub fn read(dir: &Path) -> Result<Token> {
  Ok(read_stored(dir)?.token)
}

pub(crate) const TOKEN_ENDPOINT: &str = "https://platform.claude.com/v1/oauth/token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const RENEW_BEFORE: i64 = 60;

impl Token {
  fn expiring(&self) -> bool {
    self
      .expires_at
      .is_some_and(|expiry| expiry / 1000 <= crate::now() + RENEW_BEFORE)
  }

  fn changed_from(&self, other: &Self) -> bool {
    self.access_token != other.access_token || self.refresh_token != other.refresh_token
  }
}

pub(crate) fn fresh(agent: &ureq::Agent, dir: &Path, rejected: Option<&str>) -> Result<Token> {
  fresh_at(agent, dir, rejected, TOKEN_ENDPOINT)
}

/// `fresh`, renewing at `endpoint`.
pub(crate) fn fresh_at(
  agent: &ureq::Agent,
  dir: &Path,
  rejected: Option<&str>,
  endpoint: &str,
) -> Result<Token> {
  fresh_with(agent, dir, rejected, endpoint, read_stored)
}

/// The token endpoint refused a login's refresh token outright (HTTP 400 or 401), so
/// Claude can no longer renew that login. A network error, a timeout, a 429 or a 5xx is
/// never this.
#[derive(Debug)]
pub(crate) struct Refused;

impl std::fmt::Display for Refused {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(
      "Claude could not renew this login; run ccusage login NAME if retrying does not help",
    )
  }
}

impl std::error::Error for Refused {}

fn fresh_with(
  agent: &ureq::Agent,
  dir: &Path,
  rejected: Option<&str>,
  endpoint: &str,
  read: fn(&Path) -> Result<Stored>,
) -> Result<Token> {
  let needs_refresh =
    |token: &Token| token.expiring() || rejected.is_some_and(|old| token.access_token == old);
  let stored = read(dir)?;
  if !needs_refresh(&stored.token) {
    return Ok(stored.token);
  }
  // Same lock order and heartbeat protocol as Claude Code, including its legacy
  // config-directory lock. Re-read only after acquiring both locks.
  let (current_lock, legacy_lock) = crate::auth_lock::refresh(dir)?;
  let stored = read(dir)?;
  if !needs_refresh(&stored.token) {
    return Ok(stored.token);
  }
  stored.prepare_write()?;
  current_lock.verify()?;
  legacy_lock.verify()?;
  let renewal = match renew(agent, endpoint, &stored) {
    Ok(renewal) => renewal,
    Err(error) => {
      // An older Claude may not honor our locks. Never replay a rejected refresh
      // token; reuse a replacement already saved by another process instead.
      let latest = read(dir)?;
      if latest.token.changed_from(&stored.token) && !needs_refresh(&latest.token) {
        return Ok(latest.token);
      }
      return Err(error);
    }
  };
  let write_lock = crate::auth_lock::writer(dir)?;
  current_lock.verify()?;
  legacy_lock.verify()?;
  write_lock.verify()?;
  let mut latest = read(dir)?;
  if latest.token.changed_from(&stored.token) {
    if !needs_refresh(&latest.token) {
      return Ok(latest.token);
    }
    return Err("Claude credentials changed during renewal; retry usage".into());
  }
  renewal.apply(&mut latest.document)?;
  latest.save()?;
  // Decode from memory, avoiding any decoder diagnostics containing secrets.
  parse(&serde_json::to_vec(&latest.document)?)
}

#[derive(Deserialize)]
struct Renewal {
  access_token: String,
  refresh_token: Option<String>,
  expires_in: i64,
  refresh_token_expires_in: Option<i64>,
  scope: Option<String>,
  token_type: Option<String>,
}

impl Renewal {
  fn apply(self, document: &mut serde_json::Value) -> Result<()> {
    let now = crate::now().saturating_mul(1000);
    let expires_at = self
      .expires_in
      .checked_mul(1000)
      .and_then(|n| now.checked_add(n))
      .filter(|_| self.expires_in > 0)
      .ok_or("invalid token renewal lifetime")?;
    if self.access_token.is_empty()
      || self.refresh_token.as_ref().is_some_and(String::is_empty)
      || self
        .token_type
        .as_ref()
        .is_some_and(|kind| !kind.eq_ignore_ascii_case("bearer"))
    {
      return Err("invalid token renewal response (contents redacted)".into());
    }
    let oauth = document
      .get_mut("claudeAiOauth")
      .and_then(serde_json::Value::as_object_mut)
      .ok_or("invalid Claude credential document")?;
    oauth.insert("accessToken".into(), self.access_token.into());
    oauth.insert("expiresAt".into(), expires_at.into());
    if let Some(token) = self.refresh_token {
      oauth.insert("refreshToken".into(), token.into());
      // A rotated token without a supplied lifetime must not keep the old
      // token's expiry metadata.
      oauth.remove("refreshTokenExpiresAt");
    }
    if let Some(lifetime) = self.refresh_token_expires_in {
      let expiry = lifetime
        .checked_mul(1000)
        .and_then(|n| now.checked_add(n))
        .filter(|_| lifetime > 0)
        .ok_or("invalid refresh token renewal lifetime")?;
      oauth.insert("refreshTokenExpiresAt".into(), expiry.into());
    }
    if let Some(scope) = self.scope {
      oauth.insert(
        "scopes".into(),
        scope.split_whitespace().collect::<Vec<_>>().into(),
      );
    }
    Ok(())
  }
}

fn renew(agent: &ureq::Agent, endpoint: &str, stored: &Stored) -> Result<Renewal> {
  let token = stored
    .token
    .refresh_token
    .as_deref()
    .filter(|token| !token.is_empty())
    .ok_or("Claude login cannot be renewed (no refresh token); run ccusage login NAME")?;
  let mut request = serde_json::json!({
    "grant_type": "refresh_token",
    "refresh_token": token,
    "client_id": stored.token.client_id.as_deref().unwrap_or(CLIENT_ID),
  });
  // Preserve the scopes granted at login rather than requesting extra access.
  if let Some(scopes) = stored
    .document
    .pointer("/claudeAiOauth/scopes")
    .and_then(serde_json::Value::as_array)
  {
    let scope = scopes
      .iter()
      .filter_map(serde_json::Value::as_str)
      .collect::<Vec<_>>()
      .join(" ");
    if !scope.is_empty() {
      request["scope"] = scope.into();
    }
  }
  // Never retry token exchange automatically: rotation may already have happened
  // even if the connection fails before the response arrives.
  let bytes = serde_json::to_vec(&request).map_err(|_| "could not encode token renewal request")?;
  let mut response = agent
    .post(endpoint)
    .header("Content-Type", "application/json")
    .send(&bytes)
    .map_err(|_| "token renewal failed (network or TLS); try again later; credentials redacted")?;
  match response.status().as_u16() {
    200 => {
      let bytes = response
        .body_mut()
        .with_config()
        .limit(128 * 1024)
        .read_to_vec()
        .map_err(|_| "could not read token renewal response; credentials redacted")?;
      serde_json::from_slice(&bytes)
        .map_err(|_| "invalid token renewal response (contents redacted)".into())
    }
    400 | 401 => Err(Refused.into()),
    403 => Err("Claude denied token renewal; check account access before logging in again".into()),
    429 => Err("token renewal rate limited; try again later".into()),
    status => Err(format!("token renewal returned HTTP {status}; try again later").into()),
  }
}

/// Claude with `dir` as its config directory, where its login lives too: a profile's
/// own, or in shared mode the shared config, whose store holds the seated login.
pub fn command(dir: &Path) -> Command {
  let mut command = claude();
  command.env("CLAUDE_CONFIG_DIR", dir);
  command
}

fn claude() -> Command {
  let mut command = Command::new("claude");
  // An inherited auth override must never make a named profile use another account.
  for key in [
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "CLAUDE_SECURESTORAGE_CONFIG_DIR",
    "CLAUDE_CODE_CUSTOM_OAUTH_URL",
    "CLAUDE_CODE_OAUTH_CLIENT_ID",
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
  struct Fixture {
    dir: std::path::PathBuf,
  }
  impl Fixture {
    fn new() -> Result<Self> {
      static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
      let dir = std::env::temp_dir().join(format!(
        "ccusage-auth-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
      ));
      storage::private_dir(&dir)?;
      let fixture = Self { dir };
      fixture.write(serde_json::json!({
        "unrelated": {"keep": true},
        "claudeAiOauth": {"accessToken": "OLD_ACCESS", "refreshToken": "OLD_REFRESH", "expiresAt": 0,
          "scopes": ["user:profile", "user:inference"], "clientId": "original-client",
          "subscriptionType": "max", "rateLimitTier": "tier", "futureField": {"keep": 42}}
      }))?;
      Ok(fixture)
    }
    fn write(&self, document: serde_json::Value) -> Result<()> {
      storage::write_atomic(
        &self.dir.join(".credentials.json"),
        &serde_json::to_vec(&document)?,
      )
    }
    fn fresh(&self, endpoint: &str, rejected: Option<&str>) -> Result<Token> {
      fresh_with(
        &crate::usage::Client::default().agent_for_test(),
        &self.dir,
        rejected,
        endpoint,
        read_file,
      )
    }
    fn document(&self) -> Result<serde_json::Value> {
      Ok(read_file(&self.dir)?.document)
    }
  }
  impl Drop for Fixture {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.dir);
    }
  }

  fn server(
    status: &'static str,
    body: &'static str,
    before_reply: impl FnOnce() -> Result<()> + Send + 'static,
  ) -> Result<(String, std::thread::JoinHandle<Result<String>>)> {
    use std::io::{Read, Write};
    use std::time::{Duration, Instant};
    let server = std::net::TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("http://{}/token", server.local_addr()?);
    let thread = std::thread::spawn(move || -> Result<String> {
      let deadline = Instant::now() + Duration::from_secs(5);
      let mut stream = crate::loopback::accept_by(&server, deadline)?;
      stream.set_read_timeout(Some(Duration::from_secs(5)))?;
      let mut request = Vec::new();
      let mut chunk = [0; 4096];
      loop {
        let size = stream.read(&mut chunk)?;
        if size == 0 {
          return Err("incomplete HTTP fixture request".into());
        }
        request.extend_from_slice(&chunk[..size]);
        let text = String::from_utf8_lossy(&request);
        if let Some((header, body)) = text.split_once("\r\n\r\n") {
          let length = header
            .lines()
            .find_map(|line| {
              line
                .to_ascii_lowercase()
                .strip_prefix("content-length: ")
                .and_then(|value| value.parse::<usize>().ok())
            })
            .ok_or("missing request length")?;
          if body.len() >= length {
            break;
          }
        }
      }
      before_reply()?;
      write!(
        stream,
        "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
      )?;
      Ok(String::from_utf8(request)?)
    });
    Ok((endpoint, thread))
  }

  const RENEWED: &str = r#"{"access_token":"NEW_ACCESS","refresh_token":"NEW_REFRESH","expires_in":3600,"scope":"user:profile user:inference","token_type":"Bearer","refresh_token_expires_in":86400}"#;

  #[test]
  fn expired_credentials_rotate_and_preserve_native_metadata() -> Result<()> {
    let fixture = Fixture::new()?;
    let (endpoint, server) = server("200 OK", RENEWED, || Ok(()))?;
    let token = fixture.fresh(&endpoint, None)?;
    assert_eq!(token.access_token, "NEW_ACCESS");
    assert!(
      token
        .expires_at
        .is_some_and(|expiry| expiry > crate::now() * 1000)
    );
    let request = server.join().map_err(|_| "HTTP fixture panicked")??;
    let (_, body) = request.split_once("\r\n\r\n").ok_or("no HTTP body")?;
    let body: serde_json::Value = serde_json::from_str(body)?;
    assert_eq!(body["grant_type"], "refresh_token");
    assert_eq!(body["refresh_token"], "OLD_REFRESH");
    assert_eq!(body["client_id"], "original-client");
    assert_eq!(body["scope"], "user:profile user:inference");
    let document = fixture.document()?;
    assert_eq!(document["claudeAiOauth"]["refreshToken"], "NEW_REFRESH");
    assert_eq!(document["claudeAiOauth"]["subscriptionType"], "max");
    assert_eq!(document["claudeAiOauth"]["futureField"]["keep"], 42);
    assert_eq!(document["unrelated"]["keep"], true);
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      assert_eq!(
        std::fs::metadata(fixture.dir.join(".credentials.json"))?
          .permissions()
          .mode()
          & 0o7777,
        0o600
      );
    }
    assert!(!fixture.dir.join(".oauth_refresh.lock").exists());
    assert!(!fixture.dir.join(".storage-write.lock").exists());
    // Subsequent access, including an old rejected token, reuses stored renewal.
    assert_eq!(
      fixture
        .fresh("http://127.0.0.1:1", Some("OLD_ACCESS"))?
        .access_token,
      "NEW_ACCESS"
    );
    Ok(())
  }

  #[test]
  fn concurrent_refreshes_use_one_exchange() -> Result<()> {
    let fixture = Fixture::new()?;
    let (endpoint, server) = server("200 OK", RENEWED, || {
      std::thread::sleep(std::time::Duration::from_millis(150));
      Ok(())
    })?;
    let results = std::thread::scope(|scope| {
      let mut workers = Vec::new();
      for _ in 0..6 {
        workers.push(scope.spawn(|| fixture.fresh(&endpoint, None)));
      }
      workers
        .into_iter()
        .map(|worker| worker.join().map_err(|_| "refresh fixture panicked")?)
        .collect::<Result<Vec<_>>>()
    })?;
    assert!(
      results
        .iter()
        .all(|token| token.access_token == "NEW_ACCESS")
    );
    server.join().map_err(|_| "HTTP fixture panicked")??;
    Ok(())
  }

  #[test]
  fn failed_renewal_preserves_original_credentials_and_redacts_errors() -> Result<()> {
    for (status, body) in [
      ("400 Bad Request", "OLD_REFRESH PRIVATE_BODY"),
      ("401 Unauthorized", "OLD_ACCESS PRIVATE_BODY"),
      ("429 Too Many Requests", "PRIVATE_BODY"),
      ("500 Server Error", "PRIVATE_BODY"),
      ("302 Found", "PRIVATE_BODY"),
      (
        "200 OK",
        r#"{"access_token":"PRIVATE_BODY","expires_in":"PRIVATE_BODY"}"#,
      ),
      (
        "200 OK",
        r#"{"access_token":"PRIVATE_BODY","expires_in":-1}"#,
      ),
      ("200 OK", r#"{"access_token":"","expires_in":3600}"#),
    ] {
      let fixture = Fixture::new()?;
      let original = std::fs::read(fixture.dir.join(".credentials.json"))?;
      let (endpoint, server) = server(status, body, || Ok(()))?;
      let error = fixture
        .fresh(&endpoint, None)
        .err()
        .ok_or("expected renewal failure")?
        .to_string();
      assert!(!error.contains("OLD_REFRESH"));
      assert!(!error.contains("OLD_ACCESS"));
      assert!(!error.contains("PRIVATE_BODY"));
      assert_eq!(
        std::fs::read(fixture.dir.join(".credentials.json"))?,
        original
      );
      server.join().map_err(|_| "HTTP fixture panicked")??;
    }
    Ok(())
  }

  #[test]
  fn refresh_failure_reuses_a_concurrent_replacement() -> Result<()> {
    let fixture = Fixture::new()?;
    let path = fixture.dir.join(".credentials.json");
    let (endpoint, server) = server("400 Bad Request", "invalid_grant", move || {
      storage::write_atomic(&path, br#"{"claudeAiOauth":{"accessToken":"CLAUDE_RENEWED","refreshToken":"CLAUDE_REFRESH","expiresAt":9999999999999}}"#)
    })?;
    assert_eq!(
      fixture.fresh(&endpoint, None)?.access_token,
      "CLAUDE_RENEWED"
    );
    server.join().map_err(|_| "HTTP fixture panicked")??;
    Ok(())
  }

  /// In shared mode Claude Code locks a login in CLAUDE_SECURESTORAGE_CONFIG_DIR, the
  /// profile's directory, which is where a renewal takes its refresh lock. While Claude
  /// Code holds it, the renewal waits, then uses the login Claude Code wrote.
  #[test]
  fn a_renewal_waits_for_a_refresh_lock_claude_code_holds() -> Result<()> {
    use std::time::Duration;
    let fixture = Fixture::new()?;
    let held = crate::auth_lock::Lock::acquire(
      fixture.dir.join(".oauth_refresh.lock"),
      Duration::from_secs(60),
    )?;
    let (finished, done) = std::sync::mpsc::channel();
    std::thread::scope(|scope| -> Result<()> {
      scope.spawn(|| {
        // Nothing listens there, so a renewal that skipped the lock fails at once.
        let renewed = fixture
          .fresh("http://127.0.0.1:1", None)
          .map(|token| token.access_token);
        let _ = finished.send(renewed);
      });
      assert!(
        done.recv_timeout(Duration::from_millis(500)).is_err(),
        "the renewal went ahead while Claude Code held the lock"
      );
      fixture.write(
        serde_json::json!({"claudeAiOauth": {"accessToken": "CLAUDE_RENEWED",
        "refreshToken": "CLAUDE_REFRESH", "expiresAt": 9_999_999_999_999_i64}}),
      )?;
      drop(held);
      let renewed = done
        .recv_timeout(Duration::from_secs(10))
        .map_err(|_| "the renewal did not finish once the lock was free")??;
      assert_eq!(renewed, "CLAUDE_RENEWED");
      Ok(())
    })
  }

  #[test]
  fn successful_refresh_does_not_overwrite_a_concurrent_login() -> Result<()> {
    let fixture = Fixture::new()?;
    let path = fixture.dir.join(".credentials.json");
    let (endpoint, server) = server("200 OK", RENEWED, move || {
      storage::write_atomic(&path, br#"{"claudeAiOauth":{"accessToken":"NEW_LOGIN","refreshToken":"LOGIN_REFRESH","expiresAt":9999999999999}}"#)
    })?;
    assert_eq!(fixture.fresh(&endpoint, None)?.access_token, "NEW_LOGIN");
    assert_eq!(
      fixture.document()?["claudeAiOauth"]["refreshToken"],
      "LOGIN_REFRESH"
    );
    server.join().map_err(|_| "HTTP fixture panicked")??;
    Ok(())
  }

  #[test]
  fn fresh_and_nonrefreshable_credentials_do_not_contact_the_endpoint() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.write(
      serde_json::json!({"claudeAiOauth":{"accessToken":"FRESH", "expiresAt":9999999999999_i64}}),
    )?;
    assert_eq!(
      fixture.fresh("http://127.0.0.1:1", None)?.access_token,
      "FRESH"
    );
    let error = fixture
      .fresh("http://127.0.0.1:1", Some("FRESH"))
      .err()
      .ok_or("expected no refresh token")?
      .to_string();
    assert!(error.contains("no refresh token"));
    assert!(!error.contains("FRESH"));
    Ok(())
  }

  #[test]
  fn renewal_without_rotation_retains_refresh_token_and_missing_metadata() -> Result<()> {
    let fixture = Fixture::new()?;
    let (endpoint, server) = server(
      "200 OK",
      r#"{"access_token":"NEW_ACCESS","expires_in":3600}"#,
      || Ok(()),
    )?;
    fixture.fresh(&endpoint, None)?;
    let document = fixture.document()?;
    assert_eq!(document["claudeAiOauth"]["refreshToken"], "OLD_REFRESH");
    assert_eq!(
      document["claudeAiOauth"]["scopes"],
      serde_json::json!(["user:profile", "user:inference"])
    );
    server.join().map_err(|_| "HTTP fixture panicked")??;
    Ok(())
  }

  #[cfg(target_os = "macos")]
  #[test]
  #[ignore = "requires the macOS Security service; uses only a disposable keychain"]
  fn renewal_write_keeps_claude_items_readable_by_security() -> Result<()> {
    use security_framework::os::macos::keychain::SecKeychain;
    // Interaction off: an in-process Keychain call fails rather than prompts.
    let _no_prompts = SecKeychain::disable_user_interaction()?;
    let keychain = crate::keychain::native::Keychain::new()?;
    let (service, account) = (keychain.name("claude"), "ccusage-fixture");
    keychain.add_as_claude_code(
      &service,
      account,
      br#"{"claudeAiOauth":{"accessToken":"OLD"}}"#,
    )?;
    keychain.allow_this_binary(&service, account)?;
    let renewed = br#"{"claudeAiOauth":{"accessToken":"RENEWED"}}"#;
    // ccusage's renewal write, confined to the fixture keychain.
    crate::keychain::Item {
      service: &service,
      account,
      keychain: Some(&keychain.path),
    }
    .update(renewed)?;
    // update() has already read the item back through security -w, as Claude Code does,
    // so a write that dropped apple-tool: would have prompted there: turning interaction
    // off here cannot stop a prompt in another process. Deny such a prompt and the test
    // fails; Always Allow would change the item's access and could hide the fault.
    let access = keychain.access(&service)?;
    assert!(
      access.partitions.iter().any(|id| id == "apple-tool:"),
      "partitions after the renewal write: {:?}",
      access.partitions
    );
    assert!(
      access.decrypt.iter().any(|app| app == "/usr/bin/security"),
      "decrypt access after the renewal write: {:?}",
      access.decrypt
    );
    assert_eq!(keychain.read_as_claude_code(&service, account)?, renewed);
    // Renewal only updates: a missing item stays missing, here and in the default keychain.
    let missing = keychain.name("missing");
    let refused = crate::keychain::Item {
      service: &missing,
      account,
      keychain: Some(&keychain.path),
    }
    .update(renewed);
    assert!(refused.is_err());
    assert!(!keychain.holds(&missing, account)?);
    assert!(!crate::keychain::native::anywhere(&missing, account)?);
    Ok(())
  }
}
