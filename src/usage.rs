use crate::{Result, auth, config::Store, storage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, time::Duration};

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
/// How long a fetched snapshot stays fresh, in seconds: the usage cache's lifetime. The
/// which rule takes it as its lifetime, so data it may use is never older than this.
pub const TTL: i64 = 300;
/// One weekly window's length, in seconds: how long the usage history keeps a sample.
pub const WEEK: i64 = 7 * 24 * 60 * 60;
const BACKOFF: i64 = 1800;

/// Whether data fetched at `fetched_at` is still fresh at `now`, under `lifetime`. A
/// fetch time ahead of `now` is not fresh, so a clock that moved back refetches.
pub fn fresh(fetched_at: i64, now: i64, lifetime: i64) -> bool {
  (0..lifetime).contains(&(now - fetched_at))
}

/// What a usage window limits. The response's `limits` array names the kind, and the
/// legacy top-level fields map onto the same kinds, so a window is selected by its kind
/// whichever form a response takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
  /// The five-hour session window: `session`, or the legacy `five_hour`.
  Session,
  /// The weekly window across every model: `weekly_all`, or the legacy `seven_day`.
  WeeklyAll,
  /// A weekly window for one model: `weekly_scoped`, or a legacy `seven_day_<model>`,
  /// with the model's display name when the response gives one.
  WeeklyScoped(Option<String>),
}

impl Kind {
  /// The label a window of this kind is shown under, derived here alone.
  pub fn label(&self) -> String {
    match self {
      Self::Session => "5 hour".into(),
      Self::WeeklyAll => "7 day".into(),
      Self::WeeklyScoped(Some(model)) => model.clone(),
      Self::WeeklyScoped(None) => "Weekly scoped".into(),
    }
  }

  /// The name `limits` gives this kind, which the usage history records too.
  pub fn name(&self) -> &'static str {
    match self {
      Self::Session => "session",
      Self::WeeklyAll => "weekly_all",
      Self::WeeklyScoped(_) => "weekly_scoped",
    }
  }

  /// The kind `name` names, for the model `display_name` names when it is weekly_scoped;
  /// none for a name this build does not know.
  pub fn from_name(name: &str, display_name: Option<&str>) -> Option<Self> {
    match name {
      "session" => Some(Self::Session),
      "weekly_all" => Some(Self::WeeklyAll),
      "weekly_scoped" => Some(Self::WeeklyScoped(display_name.and_then(model))),
      _ => None,
    }
  }

  /// The model a weekly_scoped window is for, when the usage names one.
  pub fn model(&self) -> Option<&str> {
    match self {
      Self::WeeklyScoped(model) => model.as_deref(),
      Self::Session | Self::WeeklyAll => None,
    }
  }
}

#[derive(Clone, Debug)]
pub struct Window {
  pub kind: Kind,
  pub utilization: f64,
  /// When the window resets, parsed once from the response's RFC 3339 text; none when
  /// the response gives no reset.
  pub resets_at: Option<time::OffsetDateTime>,
  /// A weekly_scoped window's allowance as a percent of the weekly one, from the
  /// response's `weekly_scoped_shares`; none for the other kinds, for the legacy
  /// fields, and when the response gives the window no share it can read.
  pub share: Option<f64>,
}

impl Window {
  /// A window from a response row's parts, or a history line's, refused at `context`
  /// when its percent is outside 0..100 or its reset is not RFC 3339.
  pub(crate) fn checked(
    kind: Kind,
    utilization: f64,
    resets_at: Option<&str>,
    share: Option<f64>,
    context: &str,
  ) -> Result<Self> {
    let context = crate::clean(context);
    if !is_percent(utilization) {
      return Err(format!("usage percentage at {context} is outside 0..100").into());
    }
    let resets_at = resets_at
      .map(|reset| {
        time::OffsetDateTime::parse(reset, &time::format_description::well_known::Rfc3339)
          .map_err(|_| format!("invalid usage reset timestamp at {context}"))
      })
      .transpose()?;
    Ok(Self {
      kind,
      utilization,
      resets_at,
      share,
    })
  }

  /// The reset rounded to the nearest minute, counted in minutes from the epoch; none
  /// when the window gives no reset. The which rule orders resets by it and history
  /// tells one window from the next by it, so the endpoint's jitter of under a second
  /// either side of the hour moves neither, and no reset falls in two minutes.
  pub fn reset_minute(&self) -> Option<i128> {
    const NANOS_PER_MINUTE: i128 = 60_000_000_000;
    self.resets_at.map(|reset| {
      (reset.unix_timestamp_nanos() + NANOS_PER_MINUTE / 2).div_euclid(NANOS_PER_MINUTE)
    })
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot {
  #[serde(flatten)]
  pub fields: BTreeMap<String, serde_json::Value>,
}

impl Snapshot {
  pub fn windows(&self) -> Result<Vec<Window>> {
    self.parse_windows()
  }

  fn parse_windows(&self) -> Result<Vec<Window>> {
    // Newer responses expose canonical windows, including model display names,
    // in `limits`. These supersede their duplicated legacy top-level fields.
    if let Some(value) = self.fields.get("limits").filter(|value| !value.is_null()) {
      let entries = value
        .as_array()
        .ok_or("usage field limits must be an array")?;
      let shares = self.shares();
      let mut result = Vec::new();
      for (index, entry) in entries.iter().enumerate() {
        let display_name = entry
          .pointer("/scope/model/display_name")
          .and_then(serde_json::Value::as_str);
        let Some(kind) = entry
          .get("kind")
          .and_then(serde_json::Value::as_str)
          .and_then(|name| Kind::from_name(name, display_name))
        else {
          // Unknown future categories and unrelated metadata aren't quota rows.
          continue;
        };
        let context = format!("limits[{index}]");
        let row: Limit = serde_json::from_value(entry.clone())
          .map_err(|_| format!("invalid usage window at {context}; expected numeric percent and optional string resets_at"))?;
        let share = match kind {
          Kind::WeeklyScoped(_) => shares.get(&index).copied(),
          Kind::Session | Kind::WeeklyAll => None,
        };
        result.push(Window::checked(
          kind,
          row.percent,
          row.resets_at.as_deref(),
          share,
          &context,
        )?);
      }
      if !result.is_empty() {
        return Ok(result);
      }
    }

    let mut result = Vec::new();
    for key in ["five_hour", "seven_day"] {
      if let Some(value) = self.fields.get(key).filter(|value| !value.is_null()) {
        result.push(parse_legacy(value, key)?);
      }
    }
    for (key, value) in &self.fields {
      // The prefix alone is insufficient: eg seven_day_breakdown contains
      // attribution rows and timestamps, not a utilization window.
      if key.starts_with("seven_day_") && value.get("utilization").is_some() {
        result.push(parse_legacy(value, key)?);
      }
    }
    Ok(result)
  }

  /// Each weekly_scoped window's allowance as a percent of the weekly one, by the index
  /// of its row in `limits`. Only the which rule reads a share, and only with a model
  /// named, so a share it cannot read refuses nothing: a field that is not a list gives
  /// no window a share, and an entry that does not parse, or whose share is not a
  /// percent, is passed over. Its window then has no share, and the rule says so.
  fn shares(&self) -> BTreeMap<usize, f64> {
    self
      .fields
      .get("weekly_scoped_shares")
      .and_then(serde_json::Value::as_array)
      .into_iter()
      .flatten()
      .filter_map(|entry| serde_json::from_value::<Share>(entry.clone()).ok())
      .filter(|row| is_percent(row.allowance_percent_of_weekly))
      .map(|row| (row.limit_index, row.allowance_percent_of_weekly))
      .collect()
  }

  fn validate(&self) -> Result<()> {
    if self.parse_windows()?.is_empty() {
      return Err("no subscription usage windows returned for this account".into());
    }
    Ok(())
  }
}

#[derive(Deserialize)]
struct Limit {
  percent: f64,
  resets_at: Option<String>,
}

#[derive(Deserialize)]
struct Share {
  limit_index: usize,
  allowance_percent_of_weekly: f64,
}

#[derive(Deserialize)]
struct Legacy {
  utilization: f64,
  resets_at: Option<String>,
}

fn parse_legacy(value: &serde_json::Value, key: &str) -> Result<Window> {
  let row: Legacy = serde_json::from_value(value.clone()).map_err(|_| {
    format!(
      "invalid usage window at {}; expected numeric utilization and optional string resets_at",
      crate::clean(key)
    )
  })?;
  Window::checked(
    legacy_kind(key),
    row.utilization,
    row.resets_at.as_deref(),
    None,
    key,
  )
}

/// The kind a legacy top-level key names: `five_hour` is the session, `seven_day` the
/// week across every model, and `seven_day_<model>` the week on that model, whose name
/// is the rest of the key in title case.
fn legacy_kind(key: &str) -> Kind {
  match key {
    "five_hour" => Kind::Session,
    "seven_day" => Kind::WeeklyAll,
    other => Kind::WeeklyScoped(model(
      &crate::clean(other.trim_start_matches("seven_day_"))
        .split('_')
        .map(|word| {
          let mut chars = word.chars();
          match chars.next() {
            Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
            None => String::new(),
          }
        })
        .collect::<Vec<_>>()
        .join(" "),
    )),
  }
}

/// Whether `value` is a percent: a number from 0 to 100, which no NaN or infinity is.
fn is_percent(value: f64) -> bool {
  (0.0..=100.0).contains(&value)
}

/// A model's display name with any control characters taken out, or none when nothing
/// but blanks is left.
fn model(name: &str) -> Option<String> {
  let name = crate::clean(name);
  (!name.trim().is_empty()).then_some(name)
}

#[derive(Default, Deserialize, Serialize)]
struct Cache {
  fetched_at: i64,
  retry_at: i64,
  // Bind cache to its source directory, not merely its editable profile name.
  source: String,
  // Identity verification is valid only for this exact token and expected email.
  #[serde(default)]
  credential_id: String,
  #[serde(default)]
  verified_email: Option<String>,
  snapshot: Option<Snapshot>,
}

#[derive(Clone, Serialize)]
pub struct Report {
  pub name: String,
  pub fetched_at: Option<i64>,
  pub cached: bool,
  pub snapshot: Option<Snapshot>,
  pub error: Option<String>,
}

impl Report {
  pub fn failed(name: &str, error: impl ToString) -> Self {
    Self {
      name: name.into(),
      fetched_at: None,
      cached: false,
      snapshot: None,
      error: Some(crate::clean(&error.to_string())),
    }
  }
}

pub struct Client {
  agent: ureq::Agent,
}

impl Default for Client {
  fn default() -> Self {
    Self {
      agent: ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .http_status_as_error(false)
        .max_redirects(0)
        .build()
        .into(),
    }
  }
}

enum Fetch {
  Success(Snapshot),
  Limited(i64),
  Unauthorized,
}

pub(crate) fn redact(value: &serde_json::Value, token: Option<&str>) -> serde_json::Value {
  match value {
    serde_json::Value::Object(fields) => fields
      .iter()
      .map(|(key, value)| {
        let normalized = key.to_ascii_lowercase().replace(['_', '-'], "");
        let sensitive = [
          "token",
          "authorization",
          "password",
          "secret",
          "cookie",
          "apikey",
        ]
        .iter()
        .any(|part| normalized.contains(part));
        let value = if sensitive {
          serde_json::Value::String("[REDACTED]".into())
        } else {
          redact(value, token)
        };
        let key = token
          .filter(|token| !token.is_empty())
          .map_or_else(|| key.clone(), |token| key.replace(token, "[REDACTED]"));
        (key, value)
      })
      .collect(),
    serde_json::Value::Array(values) => values.iter().map(|value| redact(value, token)).collect(),
    serde_json::Value::String(text) => {
      serde_json::Value::String(token.filter(|token| !token.is_empty()).map_or_else(
        || crate::clean(text),
        |token| crate::clean(&text.replace(token, "[REDACTED]")),
      ))
    }
    other => other.clone(),
  }
}

fn fetch(agent: &ureq::Agent, endpoint: &str, token: &str, now: i64) -> Result<Fetch> {
  let mut response = agent
    .get(endpoint)
    .header("Authorization", &format!("Bearer {token}"))
    .header("Accept", "application/json")
    .header("anthropic-beta", "oauth-2025-04-20")
    .call()
    .map_err(|_| "usage request failed (network or TLS); credentials redacted")?;
  match response.status().as_u16() {
    200 => {
      let bytes = response
        .body_mut()
        .with_config()
        .limit(128 * 1024)
        .read_to_vec()
        .map_err(|_| "could not read usage response")?;
      let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "invalid usage response JSON")?;
      // Redact before persistence or either display mode, including echoed tokens.
      let snapshot: Snapshot = serde_json::from_value(redact(&value, Some(token)))
        .map_err(|_| "invalid usage response JSON")?;
      snapshot.validate()?;
      Ok(Fetch::Success(snapshot))
    }
    401 => Ok(Fetch::Unauthorized),
    403 => Err("Claude login lacks usage access (HTTP 403); check account permissions".into()),
    429 => {
      let retry = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(BACKOFF)
        .max(BACKOFF);
      Ok(Fetch::Limited(now.saturating_add(retry)))
    }
    status => Err(format!("usage endpoint returned HTTP {status}").into()),
  }
}

fn fetch_authenticated(
  agent: &ureq::Agent,
  endpoint: &str,
  now: i64,
  mut token: impl FnMut(Option<&str>) -> Result<auth::Token>,
) -> Result<Fetch> {
  let first = token(None)?;
  let result = fetch(agent, endpoint, &first.access_token, now)?;
  if matches!(result, Fetch::Unauthorized) {
    let renewed = token(Some(&first.access_token))?;
    fetch(agent, endpoint, &renewed.access_token, now)
  } else {
    Ok(result)
  }
}

fn credential_id(token: &auth::Token, expected: Option<&str>) -> String {
  let mut digest = Sha256::new();
  digest.update(token.access_token.as_bytes());
  digest.update([0]);
  digest.update(expected.unwrap_or("").to_ascii_lowercase().as_bytes());
  format!("{:x}", digest.finalize())
}

fn check_identity(value: &serde_json::Value, expected: &str) -> Result<()> {
  let actual = value
    .pointer("/account/email")
    .and_then(serde_json::Value::as_str);
  if !actual.is_some_and(|email| email.eq_ignore_ascii_case(expected)) {
    // Do not quote a response body or account fields in diagnostics.
    return Err("stored Claude login does not match this profile's email; authenticate the intended account in its dedicated directory".into());
  }
  Ok(())
}

fn verify_identity(agent: &ureq::Agent, token: &auth::Token, expected: Option<&str>) -> Result<()> {
  verify_identity_at(
    agent,
    "https://api.anthropic.com/api/oauth/profile",
    token,
    expected,
  )
}

fn verify_identity_at(
  agent: &ureq::Agent,
  endpoint: &str,
  token: &auth::Token,
  expected: Option<&str>,
) -> Result<()> {
  let Some(expected) = expected else {
    return Ok(());
  };
  let mut response = agent
    .get(endpoint)
    .header("Authorization", &format!("Bearer {}", token.access_token))
    .header("anthropic-beta", "oauth-2025-04-20")
    .header("Accept", "application/json")
    .call()
    .map_err(|_| "could not verify Claude account identity (network or TLS)")?;
  if response.status().as_u16() != 200 {
    return Err(
      format!(
        "could not verify Claude account identity (HTTP {})",
        response.status().as_u16()
      )
      .into(),
    );
  }
  let bytes = response
    .body_mut()
    .with_config()
    .limit(128 * 1024)
    .read_to_vec()
    .map_err(|_| "could not read Claude account identity")?;
  let value = serde_json::from_slice(&bytes)
    .map_err(|_| "invalid Claude account identity response (contents redacted)")?;
  check_identity(&value, expected)
}

impl Client {
  #[cfg(test)]
  pub(crate) fn agent_for_test(&self) -> ureq::Agent {
    self.agent.clone()
  }
  pub fn report(&self, store: &Store, name: &str, refresh: bool) -> Report {
    match self.load(store, name, refresh) {
      Ok(report) => report,
      Err(error) => Report::failed(name, error),
    }
  }

  fn load(&self, store: &Store, name: &str, refresh: bool) -> Result<Report> {
    let dir = store.profile_dir(name)?;
    let source = dir.to_string_lossy().into_owned();
    let path = store.cache.join(format!("{name}.json"));

    let mut cache: Cache = match storage::read_optional(&path)? {
      Some(bytes) => serde_json::from_slice(&bytes)
        .map_err(|_| "invalid usage cache; remove it to fetch again")?,
      None => Cache::default(),
    };
    if cache.source != source {
      cache = Cache {
        source,
        ..Cache::default()
      };
    }
    let expected = store.profile(name)?.email.as_deref();
    let token = auth::read(&dir)?;
    let mut verified_id = credential_id(&token, expected);
    if cache.credential_id != verified_id {
      let token = auth::fresh(&self.agent, &dir, None)?;
      verify_identity(&self.agent, &token, expected)?;
      verified_id = credential_id(&token, expected);
      // Token rotation for the same verified account can retain its last usage.
      // Legacy caches and anonymous profiles cannot establish that continuity.
      let email = expected.map(str::to_ascii_lowercase);
      if email.is_none() || cache.verified_email != email {
        cache.snapshot = None;
        cache.fetched_at = 0;
      }
      cache.verified_email = email;
      cache.credential_id = verified_id.clone();
    }
    let now = crate::now();
    let fresh = cache.snapshot.is_some() && fresh(cache.fetched_at, now, TTL);
    let mut error = None;
    let mut cached = true;
    if cache.retry_at > now {
      error = Some(format!(
        "usage rate limited; retry in {}",
        crate::display::duration(cache.retry_at - now)
      ));
    } else if refresh || !fresh {
      let mut identity_failed = false;
      let result = fetch_authenticated(&self.agent, ENDPOINT, now, |rejected| {
        let current = auth::fresh(&self.agent, &dir, rejected)?;
        let id = credential_id(&current, expected);
        if id != verified_id {
          if let Err(error) = verify_identity(&self.agent, &current, expected) {
            identity_failed = true;
            return Err(error);
          }
          verified_id = id;
        }
        Ok(current)
      });
      if identity_failed {
        return Err(
          "could not verify the renewed login matches this profile; usage withheld".into(),
        );
      }
      cache.credential_id = verified_id;
      match result {
        Ok(Fetch::Success(snapshot)) => {
          cache.snapshot = Some(snapshot);
          cache.fetched_at = now;
          cache.retry_at = 0;
          cached = false;
          if let Err(failure) = storage::write_atomic(&path, &serde_json::to_vec(&cache)?) {
            error = Some(format!(
              "usage fetched, but could not save cache: {failure}"
            ));
          }
        }
        Ok(Fetch::Limited(retry_at)) => {
          cache.retry_at = retry_at;
          error = Some(format!(
            "usage rate limited; retry in {}",
            crate::display::duration(retry_at - now)
          ));
          if let Err(failure) = storage::write_atomic(&path, &serde_json::to_vec(&cache)?) {
            error = Some(format!(
              "usage rate limited; could not persist cooldown: {failure}"
            ));
          }
        }
        Ok(Fetch::Unauthorized) => {
          error = Some("Claude rejected the renewed login; run ccusage login NAME".into());
        }
        Err(failure) => {
          error = Some(failure.to_string());
        }
      }
    }
    Ok(Report {
      name: name.into(),
      fetched_at: cache.snapshot.as_ref().map(|_| cache.fetched_at),
      cached,
      snapshot: cache.snapshot,
      error,
    })
  }
}

pub fn demo() -> Vec<Report> {
  let now = time::OffsetDateTime::now_utc();
  let reset = |seconds| {
    (now + time::Duration::seconds(seconds))
      .format(&time::format_description::well_known::Rfc3339)
      .ok()
  };
  let session_reset = reset(10800);
  let weekly_reset = reset(489600);
  let fields = BTreeMap::from([
    (
      "five_hour".into(),
      serde_json::json!({"utilization": 100, "resets_at": session_reset}),
    ),
    (
      "seven_day".into(),
      serde_json::json!({"utilization": 62, "resets_at": weekly_reset}),
    ),
    (
      "seven_day_fable".into(),
      serde_json::json!({"utilization": 0, "resets_at": weekly_reset}),
    ),
  ]);
  vec![Report {
    name: "work (demo)".into(),
    fetched_at: Some(crate::now()),
    cached: false,
    snapshot: Some(Snapshot { fields }),
    error: None,
  }]
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::{
    io::{Read, Write},
    net::TcpListener,
  };

  #[test]
  fn account_identity_must_match_and_errors_never_echo_response_fields() -> Result<()> {
    check_identity(
      &serde_json::json!({"account":{"email":"WORK@EXAMPLE.COM"}}),
      "work@example.com",
    )?;
    for value in [
      serde_json::json!({"account":{"email":"PRIVATE_OTHER_ACCOUNT"}}),
      serde_json::json!({}),
      serde_json::json!({"account":{"email":42}}),
    ] {
      let error = check_identity(&value, "work@example.com")
        .err()
        .ok_or("must reject mismatch")?;
      assert!(!error.to_string().contains("PRIVATE_OTHER_ACCOUNT"));
    }
    let token: auth::Token = serde_json::from_value(serde_json::json!({"accessToken":"ONE"}))?;
    let other: auth::Token = serde_json::from_value(serde_json::json!({"accessToken":"TWO"}))?;
    assert_ne!(
      credential_id(&token, Some("work@example.com")),
      credential_id(&other, Some("work@example.com"))
    );
    assert_ne!(
      credential_id(&token, Some("work@example.com")),
      credential_id(&token, Some("other@example.com"))
    );
    assert_eq!(
      credential_id(&token, Some("WORK@EXAMPLE.COM")),
      credential_id(&token, Some("work@example.com"))
    );
    Ok(())
  }

  #[test]
  fn identity_http_requires_success_matching_email_and_redacts_failures() -> Result<()> {
    for (status, body, succeeds) in [
      (
        "200 OK",
        r#"{"account":{"email":"work@example.com"}}"#,
        true,
      ),
      ("200 OK", r#"{"account":{"email":"PRIVATE_OTHER"}}"#, false),
      ("200 OK", "PRIVATE_INVALID_JSON", false),
      ("401 Unauthorized", "PRIVATE_SECRET", false),
      ("302 Found", "PRIVATE_SECRET", false),
    ] {
      let server = TcpListener::bind("127.0.0.1:0")?;
      let endpoint = format!("http://{}/profile", server.local_addr()?);
      let thread = std::thread::spawn(move || -> std::io::Result<()> {
        let (mut stream, _) = server.accept()?;
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut request = [0; 4096];
        let received = stream.read(&mut request)?;
        let request = String::from_utf8_lossy(&request[..received]);
        assert!(request.starts_with("GET /profile "));
        assert!(request.contains("Bearer TEST_SECRET"));
        write!(
          stream,
          "HTTP/1.1 {status}\r\nConnection: close\r\nLocation: https://example.invalid/\r\nContent-Length: {}\r\n\r\n{body}",
          body.len()
        )
      });
      let token: auth::Token =
        serde_json::from_value(serde_json::json!({"accessToken":"TEST_SECRET"}))?;
      let result = verify_identity_at(
        &Client::default().agent,
        &endpoint,
        &token,
        Some("work@example.com"),
      );
      thread.join().map_err(|_| "identity fixture panicked")??;
      assert_eq!(result.is_ok(), succeeds);
      if let Err(error) = result {
        assert!(!error.to_string().contains("PRIVATE_"));
        assert!(!error.to_string().contains("TEST_SECRET"));
      }
    }
    Ok(())
  }

  /// Each window's kind, in the order the parser gives them.
  fn kinds(snapshot: &Snapshot) -> Result<Vec<Kind>> {
    Ok(
      snapshot
        .windows()?
        .into_iter()
        .map(|window| window.kind)
        .collect(),
    )
  }

  #[test]
  fn validates_percentages_and_optional_windows() -> Result<()> {
    let good: Snapshot = serde_json::from_str(
      r#"{"five_hour":{"utilization":62.4,"resets_at":null},"seven_day_opus":null}"#,
    )?;
    good.validate()?;
    assert_eq!(kinds(&good)?, [Kind::Session]);
    for json in [
      r#"{"five_hour":{"utilization":101}}"#,
      r#"{"five_hour":{"utilization":12,"resets_at":"bad"}}"#,
      r#"{"five_hour":null}"#,
    ] {
      assert!(serde_json::from_str::<Snapshot>(json)?.validate().is_err());
    }
    Ok(())
  }

  // ST0002 AT-01.1: the kinds and model read from limits, with the scoped window's share.
  #[test]
  fn current_response_supports_model_limits_and_ignores_attribution() -> Result<()> {
    let snapshot: Snapshot =
      serde_json::from_str(include_str!("../tests/fixtures/usage-with-breakdown.json"))?;
    snapshot.validate()?;
    assert_eq!(
      kinds(&snapshot)?,
      [
        Kind::Session,
        Kind::WeeklyAll,
        Kind::WeeklyScoped(Some("Fable".into()))
      ]
    );
    let windows = snapshot.windows()?;
    assert_eq!(windows[0].utilization, 23.5);
    assert_eq!(windows[2].utilization, 0.0);
    assert!(windows[2].resets_at.is_none());
    // The reset is parsed once, keeping the fixture's 20:00:00.123456 to the nanosecond.
    let reset = windows[0]
      .resets_at
      .ok_or("the session window has a reset")?;
    assert_eq!(
      (
        reset.hour(),
        reset.minute(),
        reset.second(),
        reset.nanosecond()
      ),
      (20, 0, 0, 123_456_000)
    );
    assert_eq!(
      windows
        .iter()
        .map(|window| window.share)
        .collect::<Vec<_>>(),
      [None, None, Some(50.0)]
    );
    Ok(())
  }

  /// Each window's share, in the order the parser gives them, from a snapshot with a
  /// weekly window and Fable's and Sonnet's scoped ones, and `shares` as its
  /// weekly_scoped_shares. The usage stands whatever `shares` holds.
  fn window_shares(shares: serde_json::Value) -> Result<Vec<Option<f64>>> {
    let snapshot: Snapshot = serde_json::from_value(serde_json::json!({
      "limits": [
        {"kind": "weekly_all", "percent": 1},
        {"kind": "weekly_scoped", "percent": 2, "scope": {"model": {"display_name": "Fable"}}},
        {"kind": "weekly_scoped", "percent": 3, "scope": {"model": {"display_name": "Sonnet"}}}
      ],
      "weekly_scoped_shares": shares
    }))?;
    snapshot.validate()?;
    Ok(
      snapshot
        .windows()?
        .into_iter()
        .map(|window| window.share)
        .collect(),
    )
  }

  #[test]
  fn a_share_attaches_to_its_scoped_row_alone() -> Result<()> {
    let (fable, sonnet) = (50.0, 30.0);
    let shares = window_shares(serde_json::json!([
      {"limit_index": 0, "allowance_percent_of_weekly": 40},
      {"limit_index": 1, "allowance_percent_of_weekly": fable},
      {"limit_index": 2, "allowance_percent_of_weekly": sonnet}
    ]))?;
    assert_eq!(shares, [None, Some(fable), Some(sonnet)]);
    Ok(())
  }

  #[test]
  fn a_shares_field_that_is_not_a_list_gives_no_window_a_share() -> Result<()> {
    let shares =
      window_shares(serde_json::json!({"limit_index": 1, "allowance_percent_of_weekly": 50}))?;
    assert!(shares.iter().all(Option::is_none));
    Ok(())
  }

  #[test]
  fn a_share_entry_that_does_not_parse_is_passed_over_and_the_rest_stand() -> Result<()> {
    let sonnet = 30.0;
    let shares = window_shares(serde_json::json!([
      {"limit_index": 1, "allowance_percent_of_weekly": "50"},
      {"allowance_percent_of_weekly": 50},
      {"limit_index": 2, "allowance_percent_of_weekly": sonnet}
    ]))?;
    assert_eq!(shares, [None, None, Some(sonnet)]);
    Ok(())
  }

  #[test]
  fn a_share_that_is_not_a_percent_is_passed_over_and_the_rest_stand() -> Result<()> {
    let sonnet = 30.0;
    for outside in [-1.0, 101.0] {
      let shares = window_shares(serde_json::json!([
        {"limit_index": 1, "allowance_percent_of_weekly": outside},
        {"limit_index": 2, "allowance_percent_of_weekly": sonnet}
      ]))?;
      assert_eq!(shares, [None, None, Some(sonnet)]);
    }
    Ok(())
  }

  // ST0002 AT-01.2: a legacy-only snapshot maps onto the same kinds.
  #[test]
  fn legacy_fields_map_onto_the_same_kinds_and_breakdown_metadata_is_not_a_window() -> Result<()> {
    let mut snapshot: Snapshot =
      serde_json::from_str(include_str!("../tests/fixtures/usage-with-breakdown.json"))?;
    snapshot.fields.remove("limits");
    snapshot.fields.insert(
      "seven_day_fable".into(),
      serde_json::json!({"utilization": 0.5, "resets_at": null}),
    );
    snapshot.fields.insert(
      "seven_day_oauth_apps".into(),
      serde_json::json!({"utilization": 1, "resets_at": null}),
    );
    snapshot.validate()?;
    assert_eq!(
      kinds(&snapshot)?,
      [
        Kind::Session,
        Kind::WeeklyAll,
        Kind::WeeklyScoped(Some("Fable".into())),
        Kind::WeeklyScoped(Some("Oauth Apps".into()))
      ]
    );
    assert!(
      snapshot
        .windows()?
        .iter()
        .all(|window| window.share.is_none())
    );
    assert_eq!(snapshot.windows()?[2].utilization, 0.5);
    Ok(())
  }

  #[test]
  fn a_scoped_window_with_no_printable_model_name_has_none() -> Result<()> {
    for scope in [
      serde_json::Value::Null,
      serde_json::json!({"model": {"display_name": " "}}),
      serde_json::json!({"model": {"display_name": "\u{1b}"}}),
    ] {
      let snapshot: Snapshot = serde_json::from_value(serde_json::json!({
        "limits": [{"kind": "weekly_scoped", "percent": 1, "scope": scope}]
      }))?;
      assert_eq!(kinds(&snapshot)?, [Kind::WeeklyScoped(None)]);
    }
    let legacy: Snapshot =
      serde_json::from_value(serde_json::json!({"seven_day_": {"utilization": 1}}))?;
    assert_eq!(kinds(&legacy)?, [Kind::WeeklyScoped(None)]);
    assert_eq!(Kind::WeeklyScoped(None).label(), "Weekly scoped");
    Ok(())
  }

  // ST0002 AT-01.3: an unknown limits kind stays ignored.
  #[test]
  fn canonical_limits_supersede_legacy_duplicates_and_unknown_kinds_are_ignored() -> Result<()> {
    let snapshot: Snapshot = serde_json::from_str(
      r#"{
      "five_hour":{"utilization":99},
      "limits":[{"kind":"session","percent":1},{"kind":"future_metadata","data":"unused"}]
    }"#,
    )?;
    snapshot.validate()?;
    assert_eq!(kinds(&snapshot)?, [Kind::Session]);
    assert_eq!(snapshot.windows()?[0].utilization, 1.0);
    Ok(())
  }

  #[test]
  fn malformed_known_rows_report_location_without_echoing_values() -> Result<()> {
    for (json, location) in [
      (
        r#"{"limits":[{"kind":"session","percent":"PRIVATE_VALUE"}]}"#,
        "limits[0]",
      ),
      (
        r#"{"five_hour":{"utilization":"PRIVATE_VALUE"}}"#,
        "five_hour",
      ),
    ] {
      let snapshot: Snapshot = serde_json::from_str(json)?;
      let error = snapshot
        .validate()
        .err()
        .ok_or("expected invalid window")?
        .to_string();
      assert!(error.contains(location));
      assert!(!error.contains("PRIVATE_VALUE"));
    }
    Ok(())
  }

  #[test]
  fn usage_response_redacts_nested_secrets_and_echoed_access_token() -> Result<()> {
    let value = serde_json::json!({
      "accessToken": "PRIVATE_ACCESS", "refresh_token": "PRIVATE_REFRESH",
      "headers": {"Authorization": "Bearer PRIVATE_ACCESS", "cookie": "PRIVATE_COOKIE"},
      "rows": [{"api-key": "PRIVATE_KEY", "message": "bad PRIVATE_ACCESS\u{1b}[31m"}],
      "five_hour": {"utilization": 62},
      "seven_day_breakdown": {"rows": []}
    });
    let output = serde_json::to_string(&redact(&value, Some("PRIVATE_ACCESS")))?;
    for secret in [
      "PRIVATE_ACCESS",
      "PRIVATE_REFRESH",
      "PRIVATE_COOKIE",
      "PRIVATE_KEY",
    ] {
      assert!(!output.contains(secret));
    }
    assert!(!output.contains(r"\u001b"));
    assert!(output.contains("seven_day_breakdown"));
    assert!(output.contains("62"));
    Ok(())
  }

  fn response(status: &str, headers: &str, body: &str) -> Result<Fetch> {
    let server = TcpListener::bind("127.0.0.1:0")?;
    let endpoint = format!("http://{}/usage", server.local_addr()?);
    let reply = format!(
      "HTTP/1.1 {status}\r\nConnection: close\r\n{headers}Content-Length: {}\r\n\r\n{body}",
      body.len()
    );
    let thread = std::thread::spawn(move || -> std::io::Result<()> {
      let (mut stream, _) = server.accept()?;
      stream.set_read_timeout(Some(Duration::from_secs(5)))?;
      let mut request = [0; 4096];
      let received = stream.read(&mut request)?;
      if received == 0 {
        return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
      }
      stream.write_all(reply.as_bytes())
    });
    let result = fetch(&Client::default().agent, &endpoint, "TEST_SECRET", 100);
    thread.join().map_err(|_| "HTTP fixture panicked")??;
    result
  }

  #[test]
  fn http_success_and_backoff() -> Result<()> {
    assert!(matches!(
      response("200 OK", "", r#"{"five_hour":{"utilization":62}}"#)?,
      Fetch::Success(_)
    ));
    assert!(matches!(
      response(
        "429 Too Many Requests",
        "Retry-After: 3600\r\n",
        "TEST_SECRET"
      )?,
      Fetch::Limited(3700)
    ));
    assert!(matches!(
      response("429 Too Many Requests", "Retry-After: 5\r\n", "")?,
      Fetch::Limited(1900)
    ));
    Ok(())
  }

  #[test]
  fn auth_errors_and_redirects_do_not_disclose_response_body() -> Result<()> {
    for status in ["403 Forbidden", "302 Found", "500 Server Error"] {
      let result = response(
        status,
        "Location: https://example.invalid/\r\n",
        "TEST_SECRET",
      );
      assert!(result.is_err());
      if let Err(error) = result {
        assert!(!error.to_string().contains("TEST_SECRET"));
      }
    }
    Ok(())
  }

  #[test]
  fn usage_retries_401_once_but_does_not_refresh_for_403_or_429() -> Result<()> {
    for (statuses, succeeds, renewals) in [
      (vec!["401 Unauthorized", "200 OK"], true, 1),
      (vec!["401 Unauthorized", "401 Unauthorized"], false, 1),
      (vec!["403 Forbidden"], false, 0),
      (vec!["429 Too Many Requests"], false, 0),
    ] {
      let server = TcpListener::bind("127.0.0.1:0")?;
      let endpoint = format!("http://{}/usage", server.local_addr()?);
      let thread = std::thread::spawn(move || -> Result<Vec<String>> {
        let mut requests = Vec::new();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        for status in statuses {
          let mut stream = crate::loopback::accept_by(&server, deadline)?;
          stream.set_read_timeout(Some(Duration::from_secs(5)))?;
          let mut buffer = [0; 8192];
          let received = stream.read(&mut buffer)?;
          requests.push(String::from_utf8(buffer[..received].to_vec())?);
          let body = r#"{"five_hour":{"utilization":42}}"#;
          write!(
            stream,
            "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
          )?;
        }
        Ok(requests)
      });
      let mut renew_count = 0;
      let outcome = fetch_authenticated(&Client::default().agent, &endpoint, 100, |rejected| {
        let access = if let Some(rejected) = rejected {
          assert_eq!(rejected, "OLD_ACCESS");
          renew_count += 1;
          "NEW_ACCESS"
        } else {
          "OLD_ACCESS"
        };
        Ok(serde_json::from_value(
          serde_json::json!({"accessToken":access}),
        )?)
      });
      let requests = thread.join().map_err(|_| "HTTP fixture panicked")??;
      assert_eq!(renew_count, renewals);
      assert!(requests[0].contains("Bearer OLD_ACCESS"));
      if renewals != 0 {
        assert!(requests[1].contains("Bearer NEW_ACCESS"));
      }
      assert_eq!(matches!(outcome, Ok(Fetch::Success(_))), succeeds);
    }
    Ok(())
  }
}
