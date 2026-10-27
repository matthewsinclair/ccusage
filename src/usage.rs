use crate::{Result, auth, config::Store, storage};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
const TTL: i64 = 300;
const BACKOFF: i64 = 1800;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Window {
  pub utilization: f64,
  pub resets_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot {
  #[serde(flatten)]
  pub fields: BTreeMap<String, serde_json::Value>,
}

impl Snapshot {
  pub fn windows(&self) -> Result<Vec<(String, Window)>> {
    self.parse_windows()
  }

  fn parse_windows(&self) -> Result<Vec<(String, Window)>> {
    // Newer responses expose canonical windows, including model display names,
    // in `limits`. These supersede their duplicated legacy top-level fields.
    if let Some(value) = self.fields.get("limits").filter(|value| !value.is_null()) {
      let entries = value
        .as_array()
        .ok_or("usage field limits must be an array")?;
      let mut result = Vec::new();
      for (index, entry) in entries.iter().enumerate() {
        let kind = entry.get("kind").and_then(serde_json::Value::as_str);
        let name = match kind {
          Some("session") => "5 hour".to_owned(),
          Some("weekly_all") => "7 day".to_owned(),
          Some("weekly_scoped") => entry
            .pointer("/scope/model/display_name")
            .and_then(serde_json::Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .map(crate::clean)
            .unwrap_or_else(|| "Weekly scoped".into()),
          // Unknown future categories and unrelated metadata aren't quota rows.
          _ => continue,
        };
        let context = format!("limits[{index}]");
        let row: Limit = serde_json::from_value(entry.clone())
          .map_err(|_| format!("invalid usage window at {context}; expected numeric percent and optional string resets_at"))?;
        let window = Window {
          utilization: row.percent,
          resets_at: row.resets_at,
        };
        validate_window(&window, &context)?;
        result.push((name, window));
      }
      if !result.is_empty() {
        return Ok(result);
      }
    }

    let mut result = Vec::new();
    for key in ["five_hour", "seven_day"] {
      if let Some(value) = self.fields.get(key).filter(|value| !value.is_null()) {
        result.push((label(key), parse_legacy(value, key)?));
      }
    }
    for (key, value) in &self.fields {
      // The prefix alone is insufficient: e.g. seven_day_breakdown contains
      // attribution rows and timestamps, not a utilization window.
      if key.starts_with("seven_day_") && value.get("utilization").is_some() {
        result.push((label(key), parse_legacy(value, key)?));
      }
    }
    Ok(result)
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

fn parse_legacy(value: &serde_json::Value, key: &str) -> Result<Window> {
  let window: Window = serde_json::from_value(value.clone()).map_err(|_| {
    format!(
      "invalid usage window at {}; expected numeric utilization and optional string resets_at",
      crate::clean(key)
    )
  })?;
  validate_window(&window, key)?;
  Ok(window)
}

fn validate_window(window: &Window, context: &str) -> Result<()> {
  let context = crate::clean(context);
  if !window.utilization.is_finite() || !(0.0..=100.0).contains(&window.utilization) {
    return Err(format!("usage percentage at {context} is outside 0..100").into());
  }
  if let Some(reset) = &window.resets_at {
    time::OffsetDateTime::parse(reset, &time::format_description::well_known::Rfc3339)
      .map_err(|_| format!("invalid usage reset timestamp at {context}"))?;
  }
  Ok(())
}

fn label(key: &str) -> String {
  match key {
    "five_hour" => "5 hour".into(),
    "seven_day" => "7 day".into(),
    other => crate::clean(other.trim_start_matches("seven_day_"))
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
  }
}

#[derive(Default, Deserialize, Serialize)]
struct Cache {
  fetched_at: i64,
  retry_at: i64,
  // Bind cache to its source directory, not merely its editable profile name.
  source: String,
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
      let bytes = response.body_mut().with_config().limit(128 * 1024).read_to_vec()
        .map_err(|_| "could not read usage response")?;
      let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| "invalid usage response JSON")?;
      // Redact before persistence or either display mode, including echoed tokens.
      let snapshot: Snapshot = serde_json::from_value(redact(&value, Some(token)))
        .map_err(|_| "invalid usage response JSON")?;
      snapshot.validate()?;
      Ok(Fetch::Success(snapshot))
    },
    401 | 403 => Err("Claude login expired or lacks access; run ccusage login NAME (or launch this profile in Claude to refresh it)".into()),
    429 => {
      let retry = response.headers().get("retry-after").and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok()).unwrap_or(BACKOFF).max(BACKOFF);
      Ok(Fetch::Limited(now.saturating_add(retry)))
    },
    status => Err(format!("usage endpoint returned HTTP {status}").into()),
  }
}

impl Client {
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
    let now = crate::now();
    let fresh = cache.snapshot.is_some() && (0..TTL).contains(&(now - cache.fetched_at));
    let mut error = None;
    let mut cached = true;
    if cache.retry_at > now {
      error = Some(format!(
        "usage rate limited; retry in {}",
        crate::display::duration(cache.retry_at - now)
      ));
    } else if refresh || !fresh {
      let result = auth::read(&dir).and_then(|token| {
        if token.expires_at.is_some_and(|expiry| expiry / 1000 <= now) {
          return Err("login token expired; launch this profile in Claude to refresh it, or run ccusage login NAME".into());
        }
        fetch(&self.agent, ENDPOINT, &token.access_token, now)
      });
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
  fn validates_percentages_and_optional_windows() -> Result<()> {
    let good: Snapshot = serde_json::from_str(
      r#"{"five_hour":{"utilization":62.4,"resets_at":null},"seven_day_opus":null}"#,
    )?;
    good.validate()?;
    assert_eq!(good.windows()?.len(), 1);
    for json in [
      r#"{"five_hour":{"utilization":101}}"#,
      r#"{"five_hour":{"utilization":12,"resets_at":"bad"}}"#,
      r#"{"five_hour":null}"#,
    ] {
      assert!(serde_json::from_str::<Snapshot>(json)?.validate().is_err());
    }
    Ok(())
  }

  #[test]
  fn current_response_supports_model_limits_and_ignores_attribution() -> Result<()> {
    let snapshot: Snapshot =
      serde_json::from_str(include_str!("../tests/fixtures/usage-with-breakdown.json"))?;
    snapshot.validate()?;
    let windows = snapshot.windows()?;
    assert_eq!(
      windows
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>(),
      ["5 hour", "7 day", "Fable"]
    );
    assert_eq!(windows[0].1.utilization, 23.5);
    assert_eq!(windows[2].1.utilization, 0.0);
    assert!(windows[2].1.resets_at.is_none());
    Ok(())
  }

  #[test]
  fn legacy_breakdown_metadata_is_not_a_quota_window() -> Result<()> {
    let mut snapshot: Snapshot =
      serde_json::from_str(include_str!("../tests/fixtures/usage-with-breakdown.json"))?;
    snapshot.fields.remove("limits");
    snapshot.fields.insert(
      "seven_day_fable".into(),
      serde_json::json!({"utilization": 0.5, "resets_at": null}),
    );
    snapshot.validate()?;
    let windows = snapshot.windows()?;
    assert_eq!(windows.len(), 3);
    assert_eq!(windows[2].0, "Fable");
    assert_eq!(windows[2].1.utilization, 0.5);
    Ok(())
  }

  #[test]
  fn canonical_limits_supersede_legacy_duplicates_and_unknown_kinds_are_ignored() -> Result<()> {
    let snapshot: Snapshot = serde_json::from_str(
      r#"{
      "five_hour":{"utilization":99},
      "limits":[{"kind":"session","percent":1},{"kind":"future_metadata","data":"unused"}]
    }"#,
    )?;
    snapshot.validate()?;
    let windows = snapshot.windows()?;
    assert_eq!(windows.len(), 1);
    assert_eq!(windows[0].1.utilization, 1.0);
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
    for status in [
      "401 Unauthorized",
      "403 Forbidden",
      "302 Found",
      "500 Server Error",
    ] {
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
}
