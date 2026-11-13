//! The which rule, "use it or lose it": work on the account with headroom whose weekly
//! reset is soonest, so that no weekly allowance goes unused at its reset, and keep the
//! accounts whose resets are further off as the reserve. `choose` is the rule's one
//! implementation: the which verb, the TUI's next marker and predict's simulation all
//! call it. It is pure: the time now, the cap and the usage cache's lifetime are its
//! arguments, and it reads no clock, file or network.

use crate::usage::{self, Kind, Report, Window};
use std::cmp::Ordering;

/// The cap a config that sets none gets, in percent of the weekly allowance.
pub const DEFAULT_CAP: f64 = 95.0;

const NANOS_PER_SECOND: i128 = 1_000_000_000;

/// What the rule makes of an account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
  /// The account to work on now.
  Chosen,
  /// Usable and below the cap, and ranked after the choice.
  Eligible,
  /// Usable, and at or above the cap.
  AtCap,
  /// Not usable, so never chosen.
  Unknown(Cause),
}

/// Why an account is unknown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cause {
  /// The report holds no usage the rule can read, and its error says why.
  Error,
  /// The usage is older than the cache's lifetime.
  Stale,
  /// The weekly reset has passed, so the percent belongs to the window before it.
  ResetPassed,
  /// The usage has no weekly window.
  NoWeeklyWindow,
}

/// An account as the rule sees it.
#[derive(Clone, Debug)]
pub struct Account {
  pub name: String,
  pub state: State,
  /// Its 1-based place in the ranking of the usable accounts; none when unknown.
  pub rank: Option<usize>,
  /// The weekly window across every model, when the usage has one.
  pub weekly: Option<Window>,
  /// Each weekly_scoped window.
  pub scoped: Vec<Window>,
  /// The session window, which the rule notes and never ranks on.
  pub session: Option<Window>,
  /// The room left before the rule moves on, in percent of the weekly allowance: the cap
  /// less the weekly percent, and with a model named, no more than the cap less that
  /// model's scoped percent, times the scoped window's share of the weekly allowance.
  /// Floored at 0, and 0 at the cap; none when unknown, and with a model named when its
  /// scoped window has no share.
  pub headroom: Option<f64>,
  /// One line on why the account stands where it does.
  pub reason: String,
  pub fetched_at: Option<i64>,
  pub error: Option<String>,
}

/// The rule's verdict on every account.
#[derive(Clone, Debug)]
pub struct Choice {
  /// The usable accounts in rank order, the choice first when there is one, then the
  /// unknown ones by name.
  pub accounts: Vec<Account>,
}

impl Choice {
  /// The account to work on now; none when nothing is eligible.
  pub fn chosen(&self) -> Option<&Account> {
    self
      .accounts
      .iter()
      .find(|account| account.state == State::Chosen)
  }

  /// The usable account whose weekly reset is soonest, as the rule orders resets, the
  /// higher ranked on a tie; none when no usable account's weekly window gives a reset.
  /// What the text names when nothing is eligible.
  pub fn soonest(&self) -> Option<&Account> {
    self
      .accounts
      .iter()
      .filter(|account| account.rank.is_some() && dated(account))
      .min_by_key(|account| reset_order(account.weekly.as_ref()))
  }
}

/// Applies the rule at `now` to every account's report. An account is usable when its
/// report holds usage fetched within `lifetime` with a weekly window whose reset is
/// still ahead, and eligible when it is usable and below `cap`, and with `model` named,
/// below the cap on that model's scoped window too. The eligible accounts rank by
/// weekly reset, soonest first, then by most headroom, then by name, and the first is
/// the choice. The accounts at the cap follow, by reset and then name, and the unknown
/// ones come last, by name.
pub fn choose(
  reports: &[Report],
  now: i64,
  cap: f64,
  model: Option<&str>,
  lifetime: i64,
) -> Choice {
  let (mut accounts, mut unknown): (Vec<_>, Vec<_>) = reports
    .iter()
    .map(|report| assess(report, now, cap, model, lifetime))
    .partition(|account| !matches!(account.state, State::Unknown(_)));
  accounts.sort_by(|a, b| {
    (a.state != State::Eligible)
      .cmp(&(b.state != State::Eligible))
      .then_with(|| reset_order(a.weekly.as_ref()).cmp(&reset_order(b.weekly.as_ref())))
      .then_with(|| headroom_order(a.headroom, b.headroom))
      .then_with(|| a.name.cmp(&b.name))
  });
  for (index, account) in accounts.iter_mut().enumerate() {
    account.rank = Some(index + 1);
  }
  if let Some(first) = accounts
    .first_mut()
    .filter(|account| account.state == State::Eligible)
  {
    first.state = State::Chosen;
    first.reason = below_cap(first, model);
  }
  unknown.sort_by(|a, b| a.name.cmp(&b.name));
  accounts.extend(unknown);
  Choice { accounts }
}

/// One account's standing before the ranking: unknown with its cause, eligible, or at
/// the cap.
fn assess(report: &Report, now: i64, cap: f64, model: Option<&str>, lifetime: i64) -> Account {
  let mut account = Account {
    name: report.name.clone(),
    state: State::Unknown(Cause::Error),
    rank: None,
    weekly: None,
    scoped: Vec::new(),
    session: None,
    headroom: None,
    reason: String::new(),
    fetched_at: report.fetched_at,
    error: report.error.clone(),
  };
  let Some(snapshot) = &report.snapshot else {
    account.reason = report
      .error
      .as_deref()
      .map_or_else(|| "no usage fetched yet".into(), crate::clean);
    return account;
  };
  let windows = match snapshot.windows() {
    Ok(windows) => windows,
    Err(error) => {
      account.reason = crate::clean(&error.to_string());
      return account;
    }
  };
  for window in windows {
    match window.kind {
      Kind::WeeklyScoped(_) => account.scoped.push(window),
      // The endpoint gives one of each; were it to give two, the first stands.
      Kind::WeeklyAll if account.weekly.is_none() => account.weekly = Some(window),
      Kind::Session if account.session.is_none() => account.session = Some(window),
      Kind::WeeklyAll | Kind::Session => {}
    }
  }
  if !report
    .fetched_at
    .is_some_and(|fetched_at| usage::fresh(fetched_at, now, lifetime))
  {
    let age = report.fetched_at.map_or_else(
      || "at an unknown time".into(),
      |fetched_at| format!("{} ago", crate::display::duration(now - fetched_at)),
    );
    account.state = State::Unknown(Cause::Stale);
    account.reason = with_error(format!("usage last fetched {age}"), report);
    return account;
  }
  let Some(weekly) = &account.weekly else {
    account.state = State::Unknown(Cause::NoWeeklyWindow);
    account.reason = with_error("no weekly window in the usage".into(), report);
    return account;
  };
  if weekly
    .resets_at
    .is_some_and(|reset| reset.unix_timestamp_nanos() <= i128::from(now) * NANOS_PER_SECOND)
  {
    account.state = State::Unknown(Cause::ResetPassed);
    account.reason = "the weekly reset has passed, so the percent is the last window's".into();
    return account;
  }
  let weekly_room = cap - weekly.utilization;
  let scoped = model.and_then(|model| scoped_for(&account.scoped, model));
  let over = scoped.filter(|window| window.utilization >= cap);
  if weekly.utilization >= cap || over.is_some() {
    account.state = State::AtCap;
    account.headroom = Some(0.0);
    account.reason = match over {
      Some(window) => format!("{} at the {cap}% cap", window.kind.label()),
      None => format!("at the {cap}% cap"),
    };
    return account;
  }
  let headroom = match scoped {
    None => Some(weekly_room),
    Some(window) => window
      .share
      .map(|share| weekly_room.min((cap - window.utilization) * share / 100.0)),
  };
  account.state = State::Eligible;
  account.headroom = headroom.map(|room| room.max(0.0));
  account.reason = below_cap(&account, model);
  account
}

/// The account's scoped window for `model`, matched by name regardless of case.
fn scoped_for<'a>(scoped: &'a [Window], model: &str) -> Option<&'a Window> {
  scoped.iter().find(|window| match &window.kind {
    Kind::WeeklyScoped(Some(name)) => name.to_lowercase() == model.to_lowercase(),
    _ => false,
  })
}

/// Why a usable account below the cap stands where it does: chosen or eligible, after
/// every dated reset when its weekly window gives none, and with `model` named and its
/// scoped window giving no share, that the share is unknown, as its headroom then is.
fn below_cap(account: &Account, model: Option<&str>) -> String {
  let standing = match (account.state == State::Chosen, dated(account)) {
    (true, true) => "soonest reset with headroom",
    // An account at the cap may give a reset, so the choice speaks of eligible ones.
    (true, false) => "headroom, though no eligible account gives a weekly reset",
    (false, true) => "eligible",
    (false, false) => "eligible, but with no weekly reset it ranks after every dated one",
  };
  match model
    .and_then(|model| scoped_for(&account.scoped, model))
    .filter(|window| window.share.is_none())
  {
    Some(window) => format!(
      "{standing}; {}'s share of the weekly allowance is unknown",
      window.kind.label()
    ),
    None => standing.into(),
  }
}

/// Whether the account's weekly window gives a reset.
fn dated(account: &Account) -> bool {
  account
    .weekly
    .as_ref()
    .is_some_and(|window| window.resets_at.is_some())
}

/// `reason`, with the report's error after it when it has one.
fn with_error(reason: String, report: &Report) -> String {
  match &report.error {
    Some(error) => format!("{reason}; {}", crate::clean(error)),
    None => reason,
  }
}

/// The order the rule gives resets: each rounded to the nearest minute, so the
/// endpoint's jitter of under a second either side of the hour can neither decide a
/// rank nor put one reset in two minutes, and no reset after every dated one.
fn reset_order(weekly: Option<&Window>) -> (bool, i128) {
  match weekly.and_then(Window::reset_minute) {
    Some(minute) => (false, minute),
    None => (true, 0),
  }
}

/// Most headroom first, and a headroom the rule cannot work out after a known one.
fn headroom_order(a: Option<f64>, b: Option<f64>) -> Ordering {
  match (a, b) {
    (Some(a), Some(b)) => b.total_cmp(&a),
    (Some(_), None) => Ordering::Less,
    (None, Some(_)) => Ordering::Greater,
    (None, None) => Ordering::Equal,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{Result, usage::Snapshot};

  fn at(text: &str) -> Result<i64> {
    Ok(
      time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339)?
        .unix_timestamp(),
    )
  }

  /// A report fetched at `fetched_at`, whose usage has a weekly window at `percent`
  /// resetting at `reset`, and a scoped window for each (model, percent, share).
  fn report(
    name: &str,
    fetched_at: i64,
    percent: f64,
    reset: Option<&str>,
    scoped: &[(&str, f64, Option<f64>)],
  ) -> Result<Report> {
    let mut limits =
      vec![serde_json::json!({"kind": "weekly_all", "percent": percent, "resets_at": reset})];
    let mut shares = Vec::new();
    for (model, percent, share) in scoped {
      if let Some(share) = share {
        shares.push(
          serde_json::json!({"limit_index": limits.len(), "allowance_percent_of_weekly": share}),
        );
      }
      limits.push(serde_json::json!({
        "kind": "weekly_scoped", "percent": percent, "resets_at": reset,
        "scope": {"model": {"display_name": model}}
      }));
    }
    let snapshot: Snapshot = serde_json::from_value(
      serde_json::json!({"limits": limits, "weekly_scoped_shares": shares}),
    )?;
    Ok(Report {
      name: name.into(),
      fetched_at: Some(fetched_at),
      cached: true,
      snapshot: Some(snapshot),
      error: None,
    })
  }

  fn names(choice: &Choice) -> Vec<&str> {
    choice
      .accounts
      .iter()
      .map(|account| account.name.as_str())
      .collect()
  }

  fn states(choice: &Choice) -> Vec<State> {
    choice
      .accounts
      .iter()
      .map(|account| account.state)
      .collect()
  }

  fn chosen(choice: &Choice) -> Option<&str> {
    choice.chosen().map(|account| account.name.as_str())
  }

  /// ST0002's worked example: the percents of the caches fetched between 05:23Z and
  /// 05:27Z on 2026-11-13, and the resets of the same weekly windows as the 07:15Z fetch
  /// gave them, jitter included: hello and engineering04 a fraction of a second before
  /// the hour, engineering01 a fraction after, engineering02 and engineering00 on it.
  /// Ranked on the raw instants, engineering02 would come before engineering01.
  // ST0002 AT-02.1: the worked example's choice and ranking.
  #[test]
  fn the_worked_example_chooses_hello_and_keeps_the_later_resets_in_reserve() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let reports = [
      report(
        "engineering00",
        now,
        0.0,
        Some("2026-11-20T04:00:00+00:00"),
        &[],
      )?,
      report(
        "engineering01",
        now,
        0.0,
        Some("2026-11-19T21:00:00.480719+00:00"),
        &[],
      )?,
      report(
        "engineering02",
        now,
        0.0,
        Some("2026-11-19T21:00:00+00:00"),
        &[],
      )?,
      report(
        "engineering04",
        now,
        22.0,
        Some("2026-11-16T11:59:59.510416+00:00"),
        &[],
      )?,
      report(
        "hello",
        now,
        73.0,
        Some("2026-11-16T09:59:59.910994+00:00"),
        &[],
      )?,
    ];
    let choice = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert_eq!(chosen(&choice), Some("hello"));
    assert_eq!(
      names(&choice),
      [
        "hello",
        "engineering04",
        "engineering01",
        "engineering02",
        "engineering00"
      ]
    );
    assert_eq!(
      states(&choice),
      [
        State::Chosen,
        State::Eligible,
        State::Eligible,
        State::Eligible,
        State::Eligible
      ]
    );
    let ranks: Vec<_> = choice.accounts.iter().map(|account| account.rank).collect();
    let places: Vec<_> = (1..=reports.len()).map(Some).collect();
    assert_eq!(ranks, places);
    assert_eq!(choice.accounts[0].headroom, Some(DEFAULT_CAP - 73.0));
    assert_eq!(choice.accounts[0].reason, "soonest reset with headroom");
    Ok(())
  }

  #[test]
  fn resets_either_side_of_the_hour_tie_and_then_more_headroom_ranks_first() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let reports = [
      report("after", now, 10.0, Some("2026-11-16T10:00:00.4+00:00"), &[])?,
      report(
        "before",
        now,
        20.0,
        Some("2026-11-16T09:59:59.6+00:00"),
        &[],
      )?,
    ];
    // By the raw instant "before" resets first; both round to 10:00, so the headroom
    // decides.
    let choice = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert_eq!(names(&choice), ["after", "before"]);
    assert_eq!(
      choice
        .accounts
        .iter()
        .map(|account| account.headroom)
        .collect::<Vec<_>>(),
      [Some(DEFAULT_CAP - 10.0), Some(DEFAULT_CAP - 20.0)]
    );
    Ok(())
  }

  // ST0002 AT-02.2: stale, reset-passed and no-weekly-window accounts are unknown.
  #[test]
  fn an_account_the_rule_cannot_read_is_unknown_with_its_reason_and_never_chosen() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let soon = Some("2026-11-14T00:00:00Z");
    let later = Some("2026-11-18T00:00:00Z");
    let failed = Report::failed("a-failed", "usage endpoint returned HTTP 500");
    // Fetched one whole lifetime ago: no longer fresh.
    let stale = report("b-stale", now - usage::TTL, 0.0, soon, &[])?;
    let passed = report("c-passed", now, 0.0, Some("2026-11-13T05:26:59Z"), &[])?;
    let mut no_weekly = report("d-no-weekly", now, 0.0, soon, &[])?;
    no_weekly.snapshot = Some(serde_json::from_value(
      serde_json::json!({"limits": [{"kind": "session", "percent": 1, "resets_at": soon}]}),
    )?);
    // A fresh fetch whose cache save failed is usable all the same.
    let mut saved = report("e-unsaved", now, 50.0, later, &[])?;
    saved.error = Some("usage fetched, but could not save cache: disk full".into());
    let choice = choose(
      &[failed, stale, passed, no_weekly, saved],
      now,
      DEFAULT_CAP,
      None,
      usage::TTL,
    );
    assert_eq!(chosen(&choice), Some("e-unsaved"));
    assert_eq!(
      states(&choice),
      [
        State::Chosen,
        State::Unknown(Cause::Error),
        State::Unknown(Cause::Stale),
        State::Unknown(Cause::ResetPassed),
        State::Unknown(Cause::NoWeeklyWindow)
      ]
    );
    let reasons: Vec<_> = choice
      .accounts
      .iter()
      .map(|account| account.reason.as_str())
      .collect();
    assert!(reasons[1].contains("HTTP 500"));
    assert!(reasons[2].contains("ago"));
    assert!(reasons[3].contains("passed"));
    assert!(reasons[4].contains("no weekly window"));
    assert!(
      choice.accounts[1..]
        .iter()
        .all(|account| account.rank.is_none() && account.headroom.is_none())
    );
    Ok(())
  }

  #[test]
  fn a_weekly_window_with_no_reset_ranks_after_every_dated_one_and_says_so() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let reports = [
      report("a-undated", now, 0.0, None, &[])?,
      report("b-dated", now, 50.0, Some("2026-11-18T00:00:00Z"), &[])?,
    ];
    let choice = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert_eq!(names(&choice), ["b-dated", "a-undated"]);
    assert!(choice.accounts[1].reason.contains("no weekly reset"));
    // Chosen with no reset of its own, it says no eligible account gives one, though an
    // account at the cap does.
    let reports = [
      report("a-undated", now, 0.0, None, &[])?,
      report(
        "b-capped",
        now,
        DEFAULT_CAP,
        Some("2026-11-18T00:00:00Z"),
        &[],
      )?,
    ];
    let choice = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert_eq!(chosen(&choice), Some("a-undated"));
    assert_eq!(
      choice.accounts[0].reason,
      "headroom, though no eligible account gives a weekly reset"
    );
    Ok(())
  }

  #[test]
  fn with_nothing_eligible_there_is_no_choice_and_the_soonest_reset_is_named() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let reports = [
      report(
        "a-late",
        now,
        DEFAULT_CAP,
        Some("2026-11-19T00:00:00Z"),
        &[],
      )?,
      report("b-soon", now, 100.0, Some("2026-11-16T00:00:00Z"), &[])?,
      Report::failed("c-broken", "usage endpoint returned HTTP 500"),
    ];
    let choice = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert_eq!(chosen(&choice), None);
    assert_eq!(names(&choice), ["b-soon", "a-late", "c-broken"]);
    assert_eq!(
      states(&choice),
      [State::AtCap, State::AtCap, State::Unknown(Cause::Error)]
    );
    assert_eq!(
      choice.soonest().map(|account| account.name.as_str()),
      Some("b-soon")
    );
    assert!(
      choice.accounts[0]
        .reason
        .contains(&format!("{DEFAULT_CAP}%"))
    );
    assert_eq!(choice.accounts[0].headroom, Some(0.0));
    let unknown = choose(&reports[2..], now, DEFAULT_CAP, None, usage::TTL);
    assert!(unknown.soonest().is_none());
    Ok(())
  }

  #[test]
  fn the_soonest_reset_is_named_only_among_accounts_that_give_one() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let capped = |name, reset| report(name, now, DEFAULT_CAP, reset, &[]);
    let mut reports = vec![capped("a-undated", None)?, capped("b-undated", None)?];
    let choice = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert!(
      choice
        .accounts
        .iter()
        .all(|account| account.state == State::AtCap)
    );
    assert!(choice.soonest().is_none());
    reports.push(capped("c-dated", Some("2026-11-19T00:00:00Z"))?);
    let choice = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert_eq!(
      choice.soonest().map(|account| account.name.as_str()),
      Some("c-dated")
    );
    Ok(())
  }

  // ST0002 AT-02.5: with a model named, its scoped window must be below the cap too.
  #[test]
  fn with_a_model_named_its_scoped_window_must_be_below_the_cap_too() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let share = Some(50.0);
    let reports = [
      report(
        "capped",
        now,
        10.0,
        Some("2026-11-14T00:00:00Z"),
        &[("Fable", DEFAULT_CAP, share)],
      )?,
      report(
        "open",
        now,
        20.0,
        Some("2026-11-15T00:00:00Z"),
        &[("Fable", 10.0, share)],
      )?,
      report("unscoped", now, 30.0, Some("2026-11-16T00:00:00Z"), &[])?,
    ];
    // With no model named, a scoped window bounds nothing.
    let any = choose(&reports, now, DEFAULT_CAP, None, usage::TTL);
    assert_eq!(chosen(&any), Some("capped"));
    let fable = choose(&reports, now, DEFAULT_CAP, Some("fable"), usage::TTL);
    assert_eq!(names(&fable), ["open", "unscoped", "capped"]);
    assert_eq!(
      states(&fable),
      [State::Chosen, State::Eligible, State::AtCap]
    );
    assert!(fable.accounts[2].reason.contains("Fable"));
    // The headroom is the lesser of the weekly room and the scoped room in weekly terms;
    // an account with no scoped window for the model is bounded by the weekly one alone.
    let expected = (DEFAULT_CAP - 20.0).min((DEFAULT_CAP - 10.0) * 50.0 / 100.0);
    assert_eq!(fable.accounts[0].headroom, Some(expected));
    assert_eq!(fable.accounts[1].headroom, Some(DEFAULT_CAP - 30.0));
    Ok(())
  }

  #[test]
  fn under_a_model_a_scoped_window_with_no_share_has_no_headroom_and_says_so() -> Result<()> {
    let now = at("2026-11-13T05:27:00Z")?;
    let (sooner, later) = (Some("2026-11-15T00:00:00Z"), Some("2026-11-16T00:00:00Z"));
    let unshared = |reset| report("a-unshared", now, 10.0, reset, &[("Fable", 10.0, None)]);
    let shared = report("b-shared", now, 10.0, later, &[("Fable", 10.0, Some(50.0))])?;
    let unknown = "Fable's share of the weekly allowance is unknown";
    // On the same reset, the known headroom ranks first. The reason names the model as
    // the usage spells it, whatever the case it was asked for in.
    let choice = choose(
      &[unshared(later)?, shared.clone()],
      now,
      DEFAULT_CAP,
      Some("fable"),
      usage::TTL,
    );
    assert_eq!(names(&choice), ["b-shared", "a-unshared"]);
    let account = &choice.accounts[1];
    assert_eq!((account.state, account.headroom), (State::Eligible, None));
    assert_eq!(account.reason, format!("eligible; {unknown}"));
    // With the sooner reset it is the choice all the same, and says so too.
    let choice = choose(
      &[unshared(sooner)?, shared.clone()],
      now,
      DEFAULT_CAP,
      Some("fable"),
      usage::TTL,
    );
    assert_eq!(chosen(&choice), Some("a-unshared"));
    assert_eq!(
      choice.accounts[0].reason,
      format!("soonest reset with headroom; {unknown}")
    );
    // With no model named, no share is read, and none is said to be unknown.
    let choice = choose(
      &[unshared(sooner)?, shared],
      now,
      DEFAULT_CAP,
      None,
      usage::TTL,
    );
    assert!(
      choice
        .accounts
        .iter()
        .all(|account| !account.reason.contains(unknown))
    );
    Ok(())
  }
}
