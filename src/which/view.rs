//! The which verb's two faces of one `Choice`, in one order: the text hv reads, and the
//! JSON other programs read, version 1 of the contract geodica-estate and the Herdr
//! panel depend on. Both are pure.

use super::{Account, Cause, Choice, State};
use crate::{Result, display, usage::Window};
use serde::Serialize;
use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

/// The contract's version. A field added keeps it; a field removed, renamed or given a
/// new meaning raises it.
pub const VERSION: u32 = 1;

/// The text: with a choice, its line comes first with its reason on it; with none, a
/// line saying why nothing is eligible, naming the soonest reset. Then every other
/// account one line each, in the JSON's order.
pub fn text(choice: &Choice, now: i64, cap: f64) -> String {
  let mut lines = Vec::new();
  if choice.chosen().is_none() {
    lines.push(format!(
      "Nothing eligible: {}.",
      nothing_eligible(choice, now, cap)
    ));
  }
  lines.extend(choice.accounts.iter().map(|account| line(account, now)));
  lines.join("\n")
}

/// Why nothing is eligible, with the soonest weekly reset when an account gives one: the
/// text's first line, and the error the verb exits with.
pub fn nothing_eligible(choice: &Choice, now: i64, cap: f64) -> String {
  if choice.accounts.iter().all(|account| account.rank.is_none()) {
    return "no account's usage could be used".into();
  }
  match soonest_reset(choice) {
    Some((account, reset)) => format!(
      "every account whose usage could be used is at the {cap}% cap; the soonest weekly reset is {}'s, in {}",
      crate::clean(&account.name),
      display::duration(reset.unix_timestamp() - now)
    ),
    None => format!(
      "every account whose usage could be used is at the {cap}% cap, and none gives a weekly reset"
    ),
  }
}

/// One account's line: its name, then its weekly window's room or use and reset, its
/// scoped and session windows, and the rule's reason. An unknown account gives its
/// reason alone.
fn line(account: &Account, now: i64) -> String {
  let name = crate::clean(&account.name);
  if matches!(account.state, State::Unknown(_)) {
    return format!("{name}: unknown, {}", account.reason);
  }
  let mut parts = Vec::new();
  if let Some(weekly) = &account.weekly {
    parts.push(match (account.state, account.headroom) {
      (State::Chosen | State::Eligible, Some(headroom)) => format!("{}% left", whole(headroom)),
      _ => format!("{}% used", whole(weekly.utilization)),
    });
    if let Some(reset) = weekly.resets_at {
      parts.push(format!(
        "resets in {}",
        display::duration(reset.unix_timestamp() - now)
      ));
    }
  }
  parts.extend(account.scoped.iter().map(|window| {
    format!(
      "{} {}% used",
      window.kind.label(),
      whole(window.utilization)
    )
  }));
  if let Some(session) = &account.session {
    parts.push(format!("session {}% used", whole(session.utilization)));
  }
  parts.push(account.reason.clone());
  format!("{name}: {}", parts.join(", "))
}

/// A percent to the whole number, as the overview shows one.
fn whole(percent: f64) -> String {
  format!("{percent:.0}")
}

/// The JSON, version 1: every field the contract names, in its order. `active` is the
/// active account after the call, and `activated` whether this call made the choice
/// active.
pub fn json(
  choice: &Choice,
  now: i64,
  cap: f64,
  model: Option<&str>,
  active: Option<&str>,
  activated: bool,
) -> Result<String> {
  let soonest = match soonest_reset(choice) {
    Some((account, reset)) => Some(Soonest {
      name: &account.name,
      resets_at: stamp(reset)?,
    }),
    None => None,
  };
  let document = Document {
    version: VERSION,
    now: stamp_unix(now)?,
    cap,
    model,
    active,
    choice: choice.chosen().map(|account| account.name.as_str()),
    activated,
    soonest,
    accounts: choice.accounts.iter().map(entry).collect::<Result<_>>()?,
  };
  Ok(serde_json::to_string_pretty(&document)?)
}

#[derive(Serialize)]
struct Document<'a> {
  version: u32,
  now: String,
  cap: f64,
  model: Option<&'a str>,
  active: Option<&'a str>,
  choice: Option<&'a str>,
  activated: bool,
  soonest: Option<Soonest<'a>>,
  accounts: Vec<Entry<'a>>,
}

#[derive(Serialize)]
struct Soonest<'a> {
  name: &'a str,
  resets_at: String,
}

#[derive(Serialize)]
struct Entry<'a> {
  name: &'a str,
  state: &'static str,
  rank: Option<usize>,
  percent: Option<f64>,
  headroom: Option<f64>,
  resets_at: Option<String>,
  reason: &'a str,
  cause: Option<&'static str>,
  scoped: Vec<Scoped<'a>>,
  session: Option<Session>,
  fetched_at: Option<String>,
  error: Option<&'a str>,
}

#[derive(Serialize)]
struct Scoped<'a> {
  model: Option<&'a str>,
  percent: f64,
  share: Option<f64>,
  resets_at: Option<String>,
}

#[derive(Serialize)]
struct Session {
  percent: f64,
  resets_at: Option<String>,
}

/// One account as the contract gives it. Its percent and reset are what its data says,
/// even when the rule could not use them; its state and cause say whether it could.
fn entry(account: &Account) -> Result<Entry<'_>> {
  Ok(Entry {
    name: &account.name,
    state: state_name(account.state),
    rank: account.rank,
    percent: account.weekly.as_ref().map(|weekly| weekly.utilization),
    headroom: account.headroom.map(|room| (room * 10.0).round() / 10.0),
    resets_at: account.weekly.as_ref().map(reset).transpose()?.flatten(),
    reason: &account.reason,
    cause: match account.state {
      State::Unknown(cause) => Some(cause_name(cause)),
      State::Chosen | State::Eligible | State::AtCap => None,
    },
    scoped: account.scoped.iter().map(scoped).collect::<Result<_>>()?,
    session: account.session.as_ref().map(session).transpose()?,
    fetched_at: account.fetched_at.map(stamp_unix).transpose()?,
    error: account.error.as_deref(),
  })
}

fn scoped(window: &Window) -> Result<Scoped<'_>> {
  Ok(Scoped {
    model: window.kind.model(),
    percent: window.utilization,
    share: window.share,
    resets_at: reset(window)?,
  })
}

fn session(window: &Window) -> Result<Session> {
  Ok(Session {
    percent: window.utilization,
    resets_at: reset(window)?,
  })
}

/// The soonest account's name and weekly reset, as `Choice::soonest` names them.
fn soonest_reset(choice: &Choice) -> Option<(&Account, OffsetDateTime)> {
  let account = choice.soonest()?;
  Some((account, account.weekly.as_ref()?.resets_at?))
}

/// A window's reset as the contract writes it; none when it gives none.
fn reset(window: &Window) -> Result<Option<String>> {
  window.resets_at.map(stamp).transpose()
}

/// A time as the contract writes every time: RFC 3339 in UTC, cut to the second, with
/// a Z. The rule's rounding of resets to the minute stays inside the rule.
fn stamp(at: OffsetDateTime) -> Result<String> {
  Ok(
    at.to_offset(UtcOffset::UTC)
      .replace_nanosecond(0)?
      .format(&Rfc3339)?,
  )
}

/// A time given in seconds from the epoch, written as `stamp` writes every time.
fn stamp_unix(seconds: i64) -> Result<String> {
  stamp(OffsetDateTime::from_unix_timestamp(seconds)?)
}

/// A state as the contract spells it.
fn state_name(state: State) -> &'static str {
  match state {
    State::Chosen => "chosen",
    State::Eligible => "eligible",
    State::AtCap => "at_cap",
    State::Unknown(_) => "unknown",
  }
}

/// A cause as the contract spells it.
fn cause_name(cause: Cause) -> &'static str {
  match cause {
    Cause::Error => "error",
    Cause::Stale => "stale",
    Cause::ResetPassed => "reset_passed",
    Cause::NoWeeklyWindow => "no_weekly_window",
  }
}
