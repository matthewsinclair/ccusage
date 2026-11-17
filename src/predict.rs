//! Predict: from the usage history, hv's burn rate in percent of one weekly allowance
//! an hour; and from that rate, following the which rule from now, the percent of each
//! account's weekly allowance projected to go unused at its current reset. A percent of
//! one plan's allowance is not a percent of another's, so it counts one plan's accounts.
//! It is pure: the history, the reports, the time now, the cap and the active account
//! are its arguments. It is arithmetic on the samples and nothing else, so no model
//! inference is made.

pub mod view;

use crate::{
  Result,
  history::Sample,
  usage::{self, Kind, Plan, Report, Snapshot, WEEK},
  which,
};
use std::collections::{BTreeMap, BTreeSet};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

/// The least span of history a rate is drawn from: one day, so that it takes in a day's
/// working hours and idle ones at least once.
pub const MIN_SPAN: i64 = 24 * 60 * 60;
const SECONDS_PER_HOUR: f64 = 3600.0;

/// The burn rate, and what it rests on.
#[derive(Clone, Debug, PartialEq)]
pub struct Rate {
  /// Percent of one weekly allowance used per elapsed hour, idle hours included; none
  /// with too little history.
  pub per_hour: Option<f64>,
  /// The weekly samples it rests on.
  pub samples: usize,
  /// The seconds from the first of them to the last.
  pub span: i64,
}

/// The burn rate the history gives: the rises in the weekly windows across every
/// account, over the hours from the first weekly sample to the last. A rise is the climb
/// between consecutive samples of one account's one window, a fall counting as none;
/// across a reset it is the next window's first percent, since that window began at 0.
/// An account's first sample adds nothing, as what came before it is unknown. With a
/// span under `MIN_SPAN`, or no rise at all, there is too little to go on.
pub fn rate(samples: &[Sample]) -> Rate {
  let mut weekly: Vec<&Sample> = samples
    .iter()
    .filter(|sample| sample.window.kind == Kind::WeeklyAll)
    .collect();
  weekly.sort_by_key(|sample| sample.sampled_at);
  let span = match (weekly.first(), weekly.last()) {
    (Some(first), Some(last)) => last.sampled_at - first.sampled_at,
    _ => 0,
  };
  let mut risen = 0.0;
  let mut last: BTreeMap<&str, &Sample> = BTreeMap::new();
  for sample in weekly.iter().copied() {
    if let Some(previous) = last.insert(&sample.account, sample) {
      risen += if previous.window.reset_minute() == sample.window.reset_minute() {
        (sample.window.utilization - previous.window.utilization).max(0.0)
      } else {
        sample.window.utilization
      };
    }
  }
  Rate {
    per_hour: (span >= MIN_SPAN && risen > 0.0).then(|| risen / hours(span)),
    samples: weekly.len(),
    span,
  }
}

/// `seconds` in hours.
pub fn hours(seconds: i64) -> f64 {
  seconds as f64 / SECONDS_PER_HOUR
}

/// One account's projection.
#[derive(Clone, Debug, PartialEq)]
pub struct Projected {
  pub name: String,
  /// Its weekly percent now.
  pub percent: f64,
  /// Its current weekly reset, in seconds from the epoch.
  pub resets_at: i64,
  /// Its weekly percent projected at that reset.
  pub at_reset: f64,
  /// What is projected to go unused at that reset: 100 less `at_reset`, so it counts the
  /// share above the cap that the rule leaves.
  pub unused: f64,
}

/// What predict reports.
#[derive(Clone, Debug, PartialEq)]
pub struct Projection {
  /// The plan whose accounts are counted, as their logins record it; none only when
  /// there is no account at all.
  pub plan: Option<Plan>,
  pub rate: Rate,
  /// Each account on that plan that the rule can use now, in the order it ranks them
  /// now, projected; none with too little history.
  pub accounts: Vec<Projected>,
  /// The accounts left out of the arithmetic, each with why: those on another plan, and
  /// those the rule cannot use now, so any whose usage gives no weekly window, as a free
  /// plan's does.
  pub left_out: Vec<(String, String)>,
  /// Each weekly window whose newest sample is below an earlier one (`falls`).
  pub falls: Vec<Fall>,
}

/// A weekly window whose newest sample is below an earlier one in the same window, as the
/// usage endpoint reported them: a fall no use explains, which the rate counts as none.
#[derive(Clone, Debug, PartialEq)]
pub struct Fall {
  pub account: String,
  /// The window's reset, in seconds from the epoch, when its samples give one.
  pub resets_at: Option<i64>,
  /// The highest earlier sample's percent, and when it was taken.
  pub earlier: (f64, i64),
  /// The newest sample's percent, and when it was taken.
  pub newest: (f64, i64),
}

/// Every weekly window in `samples` whose newest sample is below the highest earlier one
/// in the same window, by account and then by reset. Pure.
pub fn falls(samples: &[Sample]) -> Vec<Fall> {
  let mut windows: BTreeMap<(&str, Option<i128>), Vec<&Sample>> = BTreeMap::new();
  for sample in samples
    .iter()
    .filter(|sample| sample.window.kind == Kind::WeeklyAll)
  {
    windows
      .entry((sample.account.as_str(), sample.window.reset_minute()))
      .or_default()
      .push(sample);
  }
  windows
    .into_values()
    .filter_map(|mut window| {
      window.sort_by_key(|sample| sample.sampled_at);
      let (newest, earlier) = window.split_last()?;
      let peak = earlier
        .iter()
        .copied()
        .max_by(|one, other| one.window.utilization.total_cmp(&other.window.utilization))?;
      (newest.window.utilization < peak.window.utilization).then(|| Fall {
        account: newest.account.clone(),
        resets_at: newest.window.resets_at.map(OffsetDateTime::unix_timestamp),
        earlier: (peak.window.utilization, peak.sampled_at),
        newest: (newest.window.utilization, newest.sampled_at),
      })
    })
    .collect()
}

impl Projection {
  /// The unused percents summed, in percent of one weekly allowance, which means the
  /// same on every account counted since they are on one plan; none with too little
  /// history.
  pub fn unused(&self) -> Option<f64> {
    self
      .rate
      .per_hour
      .map(|_| self.accounts.iter().map(|account| account.unused).sum())
  }
}

/// An account as the simulation carries it.
struct Simulated {
  name: String,
  percent_now: f64,
  resets_now: i64,
  percent: f64,
  resets_at: i64,
  at_reset: Option<f64>,
}

impl Simulated {
  /// The account as a fresh report at `at`, for the rule to read.
  fn report(&self, at: i64) -> Result<Report> {
    let reset = OffsetDateTime::from_unix_timestamp(self.resets_at)?.format(&Rfc3339)?;
    let snapshot: Snapshot = serde_json::from_value(serde_json::json!({"limits": [
      {"kind": Kind::WeeklyAll.name(), "percent": self.percent, "resets_at": reset}
    ]}))?;
    Ok(Report {
      name: self.name.clone(),
      fetched_at: Some(at),
      cached: true,
      snapshot: Some(snapshot),
      error: None,
      plan: Default::default(),
      note: None,
    })
  }
}

/// A plan as predict compares it: its subscription type and its rate-limit tier.
type Key<'a> = (Option<&'a str>, Option<&'a str>);

/// A plan as predict compares it: its two values, whole, and never parsed.
fn key(plan: &Plan) -> Key<'_> {
  (
    plan.subscription_type.as_deref(),
    plan.rate_limit_tier.as_deref(),
  )
}

/// A plan in words: its two values as the login records them, each unrecorded when the
/// login records none.
pub fn described(plan: &Plan) -> String {
  format!(
    "plan {}, tier {}",
    plan.subscription_type.as_deref().unwrap_or("unrecorded"),
    plan.rate_limit_tier.as_deref().unwrap_or("unrecorded")
  )
}

/// The plan whose accounts predict counts, from `plans`, those of the accounts the rule
/// can use: the one most of them record, compared whole. A tie goes to `active`, the
/// active account's plan, when it is one of those tied, else to the lexically first, a
/// value no login records sorting first.
fn plan_counted<'a>(
  plans: impl Iterator<Item = &'a Plan>,
  active: Option<&'a Plan>,
) -> Option<&'a Plan> {
  let mut counts: BTreeMap<Key<'a>, (usize, &Plan)> = BTreeMap::new();
  for plan in plans {
    counts.entry(key(plan)).or_insert((0, plan)).0 += 1;
  }
  let most = counts.values().map(|(count, _)| *count).max()?;
  match active {
    Some(active)
      if counts
        .get(&key(active))
        .is_some_and(|(count, _)| *count == most) =>
    {
      Some(active)
    }
    // The map is in lexical order, so the first plan tied is the lexically first.
    _ => counts
      .values()
      .find(|(count, _)| *count == most)
      .map(|(_, plan)| *plan),
  }
}

/// The plan `name`'s login records, as its report gives it.
fn plan_of<'a>(reports: &'a [Report], name: &str) -> Option<&'a Plan> {
  reports
    .iter()
    .find(|report| report.name == name)
    .map(|report| &report.plan)
}

/// The plan an `account`'s samples that record none are counted on, as an older ccusage
/// wrote them: the plan its report gives, unless that is the empty plan of a report that
/// could not read its login, when the plan its latest sample recorded stands in. None
/// for an account with no report, which is no longer a profile, and whose samples
/// `project` neither counts nor names. Pure.
fn account_plan<'a>(
  reports: &'a [Report],
  samples: &'a [Sample],
  account: &str,
) -> Option<&'a Plan> {
  let reported = plan_of(reports, account)?;
  if *reported != Plan::default() {
    return Some(reported);
  }
  Some(
    samples
      .iter()
      .filter(|sample| sample.account == account && sample.plan.is_some())
      .max_by_key(|sample| sample.sampled_at)
      .and_then(|sample| sample.plan.as_ref())
      .unwrap_or(reported),
  )
}

/// The plan `sample` is counted on: the one it was taken on, else its account's, which
/// `fallback` holds (`account_plan`). Pure.
fn taken_on<'a>(
  sample: &'a Sample,
  fallback: &BTreeMap<&str, Option<&'a Plan>>,
) -> Option<&'a Plan> {
  sample
    .plan
    .as_ref()
    .or_else(|| fallback.get(sample.account.as_str()).copied().flatten())
}

/// One account's weekly samples, as the rate counts them: how many there are, how many
/// it leaves out, and when the latest of those was taken and on what plan.
#[derive(Default)]
struct Counted<'a> {
  weekly: usize,
  left: usize,
  latest: Option<(i64, Option<&'a Plan>)>,
}

/// What predict says of an account's weekly samples that the rate leaves out: how many,
/// and the plan the latest of them was taken on, or that none can be told. Pure.
fn samples_left_out(counted: &Counted) -> String {
  let which = if counted.left == counted.weekly {
    format!("its {} weekly samples", counted.left)
  } else {
    format!("{} of its {} weekly samples", counted.left, counted.weekly)
  };
  match counted
    .latest
    .and_then(|(_, plan)| plan)
    .filter(|plan| **plan != Plan::default())
  {
    Some(plan) => format!("the rate leaves out {which}, on {}", described(plan)),
    None => {
      format!("the rate leaves out {which}, since none records a plan and its usage now gives none")
    }
  }
}

/// When `account`'s weekly window resets, when the rule read one.
fn weekly_reset(account: &which::Account) -> Option<OffsetDateTime> {
  account.weekly.as_ref().and_then(|weekly| weekly.resets_at)
}

/// Projects each account's weekly percent at its current reset, following the which
/// rule from `now` at the history's burn rate, for the accounts of one plan: the one most
/// of the accounts the rule can use record, else most of all the accounts, a tie going to
/// `active`'s (`plan_counted`), so that every percent is of the same allowance. The rate
/// counts that plan's accounts' samples alone, and a usable account on another plan is
/// left out. Event by event up to the last of the current resets: at each step
/// `which::choose` itself picks the account, which climbs at the rate until it reaches
/// `cap` or a reset comes, and a window that resets starts again at 0 with its reset a
/// week on.
pub fn project(
  samples: &[Sample],
  reports: &[Report],
  now: i64,
  cap: f64,
  active: Option<&str>,
) -> Result<Projection> {
  // An account with no report is no longer a profile: its samples are neither counted
  // nor named, whatever plan they record.
  let samples: Vec<Sample> = samples
    .iter()
    .filter(|sample| plan_of(reports, &sample.account).is_some())
    .cloned()
    .collect();
  let samples = samples.as_slice();
  let falls = falls(samples);
  let choice = which::choose(reports, now, cap, None, usage::TTL);
  let active_plan = active.and_then(|name| plan_of(reports, name));
  let plan = plan_counted(
    choice
      .accounts
      .iter()
      .filter(|account| account.rank.is_some() && weekly_reset(account).is_some())
      .filter_map(|account| plan_of(reports, &account.name)),
    active_plan,
  )
  // With no account the rule can use, the plan is chosen the same way from every account,
  // so the rate is still one plan's percent.
  .or_else(|| plan_counted(reports.iter().map(|report| &report.plan), active_plan))
  .cloned();
  // With no plan, as when there is no account at all, no sample counts.
  let on_plan = |name: &str| {
    plan
      .as_ref()
      .is_some_and(|counted| plan_of(reports, name) == Some(counted))
  };
  // A sample counts on the plan it was taken on, so an account whose usage fails now
  // keeps its history; one an older ccusage wrote counts on its account's plan.
  let fallback: BTreeMap<&str, Option<&Plan>> = samples
    .iter()
    .map(|sample| sample.account.as_str())
    .collect::<BTreeSet<_>>()
    .into_iter()
    .map(|account| (account, account_plan(reports, samples, account)))
    .collect();
  let counts = |sample: &Sample| {
    plan
      .as_ref()
      .is_some_and(|counted| taken_on(sample, &fallback) == Some(counted))
  };
  let counted: Vec<Sample> = samples
    .iter()
    .filter(|sample| counts(sample))
    .cloned()
    .collect();
  let rate = rate(&counted);
  let mut by_account: BTreeMap<&str, Counted> = BTreeMap::new();
  for sample in samples
    .iter()
    .filter(|sample| sample.window.kind == Kind::WeeklyAll)
  {
    let account = by_account.entry(sample.account.as_str()).or_default();
    account.weekly += 1;
    if !counts(sample) {
      account.left += 1;
      if account.latest.is_none_or(|(at, _)| sample.sampled_at >= at) {
        account.latest = Some((sample.sampled_at, taken_on(sample, &fallback)));
      }
    }
  }
  let mut left_out = Vec::new();
  let mut accounts = Vec::new();
  for account in &choice.accounts {
    match (account.rank, weekly_reset(account)) {
      (Some(_), Some(reset)) if on_plan(account.name.as_str()) => accounts.push(Simulated {
        name: account.name.clone(),
        percent_now: account
          .weekly
          .as_ref()
          .map_or(0.0, |weekly| weekly.utilization),
        resets_now: reset.unix_timestamp(),
        percent: account
          .weekly
          .as_ref()
          .map_or(0.0, |weekly| weekly.utilization),
        resets_at: reset.unix_timestamp(),
        at_reset: None,
      }),
      (Some(_), Some(_)) => {
        let other = plan_of(reports, &account.name).cloned().unwrap_or_default();
        left_out.push((
          account.name.clone(),
          format!("another plan's allowance ({})", described(&other)),
        ));
      }
      (Some(_), None) => {
        left_out.push((account.name.clone(), "no weekly reset to project to".into()))
      }
      (None, _) => left_out.push((account.name.clone(), account.reason.clone())),
    }
  }
  // Every account whose weekly samples the rate leaves out is named, with how many,
  // beside any other reason it is left out.
  for (name, counted) in by_account.iter().filter(|(_, counted)| counted.left > 0) {
    let note = samples_left_out(counted);
    match left_out.iter_mut().find(|(left, _)| left == name) {
      Some((_, why)) => *why = format!("{why}; {note}"),
      None => left_out.push(((*name).to_owned(), note)),
    }
  }
  let Some(per_hour) = rate.per_hour else {
    return Ok(Projection {
      plan,
      rate,
      accounts: Vec::new(),
      left_out,
      falls,
    });
  };
  if let Some(horizon) = accounts.iter().map(|account| account.resets_now).max() {
    let mut at = now;
    loop {
      for account in &mut accounts {
        while account.resets_at <= at {
          account.at_reset.get_or_insert(account.percent);
          account.percent = 0.0;
          account.resets_at += WEEK;
        }
      }
      if at >= horizon {
        break;
      }
      let reports = accounts
        .iter()
        .map(|account| account.report(at))
        .collect::<Result<Vec<_>>>()?;
      let chosen = which::choose(&reports, at, cap, None, usage::TTL)
        .chosen()
        .and_then(|chosen| {
          accounts
            .iter()
            .position(|account| account.name == chosen.name)
        });
      let mut next = accounts
        .iter()
        .map(|account| account.resets_at)
        .min()
        .map_or(horizon, |reset| reset.min(horizon));
      if let Some(index) = chosen {
        let to_cap = ((cap - accounts[index].percent) / per_hour * SECONDS_PER_HOUR).ceil();
        next = next.min(at.saturating_add((to_cap as i64).max(1)));
        let account = &mut accounts[index];
        account.percent = (account.percent + per_hour * hours(next - at)).min(cap);
      }
      at = next;
    }
  }
  Ok(Projection {
    plan,
    rate,
    accounts: accounts
      .into_iter()
      .map(|account| {
        let at_reset = account.at_reset.unwrap_or(account.percent);
        Projected {
          name: account.name,
          percent: account.percent_now,
          resets_at: account.resets_now,
          at_reset,
          unused: 100.0 - at_reset,
        }
      })
      .collect(),
    left_out,
    falls,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::usage::Window;

  const HOUR: i64 = 3600;

  fn at(text: &str) -> Result<i64> {
    Ok(OffsetDateTime::parse(text, &Rfc3339)?.unix_timestamp())
  }

  /// A weekly sample of `account` at `percent`, on the window that resets at `reset`,
  /// recording no plan, as an older ccusage wrote one.
  fn sample(account: &str, percent: f64, reset: i64, sampled_at: i64) -> Result<Sample> {
    Ok(Sample {
      account: account.into(),
      window: Window {
        kind: Kind::WeeklyAll,
        utilization: percent,
        resets_at: Some(OffsetDateTime::from_unix_timestamp(reset)?),
        share: None,
      },
      sampled_at,
      plan: None,
    })
  }

  /// A fresh report at `now` whose weekly window is at `percent` and resets at `reset`.
  fn report(name: &str, now: i64, percent: f64, reset: i64) -> Result<Report> {
    Simulated {
      name: name.into(),
      percent_now: percent,
      resets_now: reset,
      percent,
      resets_at: reset,
      at_reset: None,
    }
    .report(now)
  }

  // ST0002 AT-05.1: the rate is the rises over the history's span.
  #[test]
  fn the_rate_is_the_rises_across_accounts_and_resets_over_the_hours_the_history_spans()
  -> Result<()> {
    let start = at("2026-11-13T05:00:00Z")?;
    let (first, next) = (start + 6 * HOUR, start + 6 * HOUR + WEEK);
    let (a1, a2, a3) = (10.0, 30.0, 5.0);
    let (b1, b2, b3) = (50.0, 45.0, 60.0);
    let samples = [
      sample("a", a1, first, start)?,
      sample("a", a2, first, start + 2 * HOUR)?,
      // After a's reset, its next window's first percent is a rise from 0.
      sample("a", a3, next, start + 8 * HOUR)?,
      sample("b", b1, next, start + HOUR)?,
      // A fall is no rise.
      sample("b", b2, next, start + 3 * HOUR)?,
      sample("b", b3, next, start + 30 * HOUR)?,
    ];
    let rate = rate(&samples);
    let risen = (a2 - a1) + a3 + (b3 - b2);
    let span = 30 * HOUR;
    assert_eq!(rate.span, span);
    assert_eq!(rate.samples, samples.len());
    assert_eq!(rate.per_hour, Some(risen / hours(span)));
    Ok(())
  }

  #[test]
  fn under_a_day_of_history_or_with_no_rise_there_is_no_rate() -> Result<()> {
    let start = at("2026-11-13T05:00:00Z")?;
    let reset = start + WEEK;
    let short = [
      sample("a", 10.0, reset, start)?,
      sample("a", 20.0, reset, start + MIN_SPAN - 1)?,
    ];
    assert_eq!(rate(&short).per_hour, None);
    let flat = [
      sample("a", 10.0, reset, start)?,
      sample("a", 10.0, reset, start + MIN_SPAN)?,
    ];
    assert_eq!(rate(&flat).per_hour, None);
    let enough = [
      sample("a", 10.0, reset, start)?,
      sample("a", 20.0, reset, start + MIN_SPAN)?,
    ];
    assert!(rate(&enough).per_hour.is_some());
    Ok(())
  }

  // ST0002 AT-05.2: following the rule at the rate, each account's unused share at its reset.
  #[test]
  fn the_projection_follows_the_rule_to_each_reset_and_counts_what_goes_unused() -> Result<()> {
    let now = at("2026-11-13T05:00:00Z")?;
    let cap = which::DEFAULT_CAP;
    // A history whose rate is `per_hour`: one rise over exactly one day.
    let per_hour = 3.5;
    let risen = per_hour * hours(MIN_SPAN);
    let history = [
      sample("soon", 0.0, now + WEEK, now - MIN_SPAN)?,
      sample("soon", risen, now + WEEK, now)?,
    ];
    // soon resets before it can reach the cap; later then reaches it before its own
    // reset, the horizon, after which soon's next window takes the rest.
    let (soon, later) = (50.0, 0.0);
    // At this rate later's climb to the cap ends part way through a second, so the step
    // to it overshoots, and a climb not held at the cap would show.
    assert!(((cap - later) / per_hour * SECONDS_PER_HOUR).fract() > 0.0);
    let (soon_reset, later_reset) = (now + 10 * HOUR, now + 100 * HOUR);
    let reports = [
      report("soon", now, soon, soon_reset)?,
      report("later", now, later, later_reset)?,
    ];
    let projection = project(&history, &reports, now, cap, None)?;
    assert_eq!(projection.rate.per_hour, Some(per_hour));
    let soon_at_reset = (soon + per_hour * hours(soon_reset - now)).min(cap);
    let later_at_reset = (later + per_hour * hours(later_reset - soon_reset)).min(cap);
    let projected: Vec<_> = projection
      .accounts
      .iter()
      .map(|account| (account.name.as_str(), account.at_reset, account.unused))
      .collect();
    assert_eq!(
      projected,
      [
        ("soon", soon_at_reset, 100.0 - soon_at_reset),
        ("later", later_at_reset, 100.0 - later_at_reset)
      ]
    );
    assert_eq!(
      projection.unused(),
      Some(200.0 - soon_at_reset - later_at_reset)
    );
    Ok(())
  }

  #[test]
  fn with_too_little_history_nothing_is_projected_and_an_unusable_account_is_left_out() -> Result<()>
  {
    let now = at("2026-11-13T05:00:00Z")?;
    let reports = [
      report("usable", now, 10.0, now + WEEK)?,
      Report::failed("broken", "usage endpoint returned HTTP 500"),
    ];
    let projection = project(&[], &reports, now, which::DEFAULT_CAP, None)?;
    assert_eq!(projection.rate.per_hour, None);
    assert!(projection.accounts.is_empty());
    assert_eq!(projection.unused(), None);
    assert_eq!(
      projection.left_out,
      [(
        "broken".to_owned(),
        "usage endpoint returned HTTP 500".to_owned()
      )]
    );
    Ok(())
  }

  /// A plan as a login records it.
  fn plan(subscription_type: &str, rate_limit_tier: &str) -> Plan {
    Plan {
      subscription_type: Some(subscription_type.into()),
      rate_limit_tier: Some(rate_limit_tier.into()),
    }
  }

  // ST0002 AT-05.4: predict counts one plan, the one most of the usable accounts record.
  #[test]
  fn the_projection_counts_the_plan_most_usable_accounts_record_and_leaves_out_the_rest()
  -> Result<()> {
    let now = at("2026-11-13T05:00:00Z")?;
    let cap = which::DEFAULT_CAP;
    let (max, pro) = (
      plan("max", "default_claude_max_20x"),
      plan("pro", "default_claude_ai"),
    );
    // A fresh report on `plan` whose weekly window is unused and resets `after` now.
    let on = |name: &str, after: i64, plan: &Plan| -> Result<Report> {
      let mut planned = report(name, now, 0.0, now + after)?;
      planned.plan = plan.clone();
      Ok(planned)
    };
    let reports = [
      on("a", 10 * HOUR, &max)?,
      on("b", 20 * HOUR, &max)?,
      on("c", 30 * HOUR, &pro)?,
    ];
    // A day's rise on a max account, and a larger one on the pro account, which is a
    // percent of another allowance and so no part of the rate.
    let (max_risen, pro_risen) = (24.0, 48.0);
    let history = [
      sample("a", 0.0, now + WEEK, now - MIN_SPAN)?,
      sample("a", max_risen, now + WEEK, now)?,
      sample("c", 0.0, now + WEEK, now - MIN_SPAN)?,
      sample("c", pro_risen, now + WEEK, now)?,
    ];
    let projection = project(&history, &reports, now, cap, None)?;
    assert_eq!(projection.plan.as_ref(), Some(&max));
    assert_eq!(projection.rate.per_hour, Some(max_risen / hours(MIN_SPAN)));
    let names: Vec<_> = projection
      .accounts
      .iter()
      .map(|account| account.name.as_str())
      .collect();
    assert_eq!(names, ["a", "b"]);
    // c's samples are named as left out of the rate too, with how many, and its plan once
    // (ISSUE:0010 item 9).
    let c_samples = history
      .iter()
      .filter(|sample| sample.account == "c")
      .count();
    assert_eq!(
      projection.left_out,
      [(
        "c".to_owned(),
        format!(
          "another plan's allowance ({}), with its {c_samples} weekly samples",
          described(&pro)
        )
      )]
    );
    // One account on each plan: the tie goes to the active account's plan, else to the
    // lexically first.
    let tied = [reports[0].clone(), reports[2].clone()];
    assert_eq!(
      project(&history, &tied, now, cap, Some("c"))?.plan.as_ref(),
      Some(&pro)
    );
    assert_eq!(
      project(&history, &tied, now, cap, None)?.plan.as_ref(),
      Some(&max)
    );
    // With no account the rule can use, the plan is chosen the same way from them all, and
    // the rate is still that plan's alone.
    let failed = |name: &str, plan: &Plan| -> Report {
      let mut planned = Report::failed(name, "usage endpoint returned HTTP 500");
      planned.plan = plan.clone();
      planned
    };
    let unusable = [failed("a", &max), failed("c", &pro), failed("d", &pro)];
    let projection = project(&history, &unusable, now, cap, None)?;
    assert_eq!(projection.plan.as_ref(), Some(&pro));
    assert_eq!(projection.rate.per_hour, Some(pro_risen / hours(MIN_SPAN)));
    assert!(projection.accounts.is_empty());
    Ok(())
  }

  // ISSUE:0008 Acceptance 4: the rate counts each sample on the plan it was taken on, so
  // it is the same whether or not an account's usage fails now; and an account whose
  // samples no plan can be told for is named, with how many the rate leaves out.
  #[test]
  fn a_failed_accounts_samples_count_on_the_plan_they_were_taken_on() -> Result<()> {
    let now = at("2026-11-13T05:00:00Z")?;
    let cap = which::DEFAULT_CAP;
    let max = plan("max", "default_claude_max_20x");
    let taken = |account: &str, percent: f64, sampled_at: i64, on: Option<&Plan>| {
      sample(account, percent, now + WEEK, sampled_at).map(|kept| Sample {
        plan: on.cloned(),
        ..kept
      })
    };
    let (a_risen, b_risen) = (24.0, 12.0);
    let history = [
      taken("a", 0.0, now - MIN_SPAN, Some(&max))?,
      taken("a", a_risen, now, Some(&max))?,
      taken("b", 0.0, now - MIN_SPAN, Some(&max))?,
      taken("b", b_risen, now, Some(&max))?,
      // As an older ccusage wrote them.
      taken("c", 0.0, now - MIN_SPAN, None)?,
      taken("c", 6.0, now, None)?,
      // gone's profile was taken out of the config since, so its samples are neither
      // counted nor named, whatever plan they record.
      taken("gone", 0.0, now - MIN_SPAN, Some(&max))?,
      taken("gone", 30.0, now, Some(&max))?,
    ];
    let reading = |name: &str| -> Result<Report> {
      let mut planned = report(name, now, 0.0, now + WEEK)?;
      planned.plan = max.clone();
      Ok(planned)
    };
    let read = project(&history, &[reading("a")?, reading("b")?], now, cap, None)?;
    assert_eq!(
      read.rate.per_hour,
      Some((a_risen + b_risen) / hours(MIN_SPAN))
    );
    // b's login is another account's now, and c has none, so their usage fails.
    let failing = [
      reading("a")?,
      Report::failed(
        "b",
        "stored Claude login does not match this profile's email",
      ),
      Report::failed("c", "not authenticated; run ccusage login NAME"),
    ];
    let failed = project(&history, &failing, now, cap, None)?;
    assert_eq!(failed.rate, read.rate);
    for projection in [&read, &failed] {
      assert!(
        projection.left_out.iter().all(|(name, _)| name != "gone"),
        "{:?}",
        projection.left_out
      );
    }
    let why = |name: &str| {
      failed
        .left_out
        .iter()
        .find(|(left, _)| left == name)
        .map(|(_, why)| why.clone())
        .unwrap_or_default()
    };
    assert!(!why("b").contains("weekly samples"), "{}", why("b"));
    // ISSUE:0010 item 9: c's reason can end in a remedy, so the text gives its samples a
    // line of their own, right after c's; the JSON keeps them in c's reason, as before.
    assert!(!why("c").contains("weekly samples"), "{}", why("c"));
    let c_samples = history
      .iter()
      .filter(|sample| sample.account == "c")
      .count();
    let note = format!(
      "the rate leaves out its {c_samples} weekly samples, since none records a plan and its usage now gives none"
    );
    let said = view::text(&failed, now);
    let lines: Vec<&str> = said.lines().collect();
    let c = lines
      .iter()
      .position(|line| line.starts_with("c: left out, "))
      .ok_or("c is left out")?;
    let c_line = format!("c: {note}");
    assert_eq!(lines.get(c + 1).copied(), Some(c_line.as_str()), "{said}");
    let json: serde_json::Value = serde_json::from_str(&view::json(&failed, now, cap)?)?;
    let reason = json["left_out"]
      .as_array()
      .ok_or("a left_out array")?
      .iter()
      .find(|account| account["name"] == "c")
      .and_then(|account| account["reason"].as_str())
      .ok_or("c in left_out")?;
    assert!(reason.ends_with(&format!("; {note}")), "{reason}");
    Ok(())
  }

  // ISSUE:0010 items 9 and 10: an account the rule projects is never left out, even when
  // the rate leaves out some of its samples, which get a line of their own after the
  // total; an account on another plan says its samples once, after its plan. The JSON
  // names only the accounts it leaves out.
  #[test]
  fn samples_the_rate_leaves_out_never_leave_a_projected_account_out() -> Result<()> {
    let now = at("2026-11-13T05:00:00Z")?;
    let cap = which::DEFAULT_CAP;
    let (max, pro) = (
      plan("max", "default_claude_max_20x"),
      plan("pro", "default_claude_ai"),
    );
    let taken = |account: &str, percent: f64, sampled_at: i64, on: &Plan| {
      sample(account, percent, now + WEEK, sampled_at).map(|kept| Sample {
        plan: Some(on.clone()),
        ..kept
      })
    };
    // x was on pro two days ago and is on max now, as a is; c is on pro throughout.
    let history = [
      taken("x", 0.0, now - 2 * MIN_SPAN, &pro)?,
      taken("x", 10.0, now - MIN_SPAN, &max)?,
      taken("x", 20.0, now, &max)?,
      taken("a", 0.0, now - MIN_SPAN, &max)?,
      taken("a", 24.0, now, &max)?,
      taken("c", 0.0, now - MIN_SPAN, &pro)?,
      taken("c", 30.0, now, &pro)?,
    ];
    let on = |name: &str, plan: &Plan| -> Result<Report> {
      let mut planned = report(name, now, 0.0, now + WEEK)?;
      planned.plan = plan.clone();
      Ok(planned)
    };
    let reports = [on("x", &max)?, on("a", &max)?, on("c", &pro)?];
    let projection = project(&history, &reports, now, cap, None)?;
    assert!(
      projection
        .accounts
        .iter()
        .any(|account| account.name == "x"),
      "{:?}",
      projection.accounts
    );
    assert_eq!(
      projection.left_out,
      [(
        "c".to_owned(),
        format!(
          "another plan's allowance ({}), with its 2 weekly samples",
          described(&pro)
        )
      )]
    );
    let said = view::text(&projection, now);
    let lines: Vec<&str> = said.lines().collect();
    let total = lines
      .iter()
      .position(|line| line.starts_with("Projected unused"))
      .ok_or("a total")?;
    let x_line = format!(
      "x: the rate leaves out 1 of its 3 weekly samples, taken on another plan ({})",
      described(&pro)
    );
    assert_eq!(
      lines.get(total + 1).copied(),
      Some(x_line.as_str()),
      "{said}"
    );
    assert!(!said.contains("x: left out"), "{said}");
    let json: serde_json::Value = serde_json::from_str(&view::json(&projection, now, cap)?)?;
    let left: Vec<_> = json["left_out"]
      .as_array()
      .ok_or("a left_out array")?
      .iter()
      .map(|account| account["name"].clone())
      .collect();
    assert_eq!(left, ["c"]);
    Ok(())
  }

  // ISSUE:0008 Acceptance 6: predict says when a weekly window's newest sample is below an
  // earlier one in the same window, as the usage endpoint reported them, and nothing of it
  // otherwise.
  #[test]
  fn predict_says_when_a_windows_newest_sample_is_below_an_earlier_one() -> Result<()> {
    let now = at("2026-11-16T05:00:00Z")?;
    let cap = which::DEFAULT_CAP;
    // A window that reset three hours ago, and the one that followed it.
    let (ended, current) = (now - 3 * HOUR, now - 3 * HOUR + WEEK);
    let history = [
      // a's current window rises.
      sample("a", 2.0, current, now - 2 * HOUR)?,
      sample("a", 5.0, current, now - HOUR)?,
      // a's last window fell from 38% to 9%, and read no more before it reset.
      sample("a", 38.0, ended, now - 20 * HOUR)?,
      sample("a", 9.0, ended, now - 17 * HOUR)?,
      // b's fell, then rose past where it had been.
      sample("b", 50.0, ended, now - 20 * HOUR)?,
      sample("b", 10.0, ended, now - 10 * HOUR)?,
      sample("b", 60.0, ended, now - 5 * HOUR)?,
      // c's current window fell.
      sample("c", 30.0, current, now - 2 * HOUR)?,
      sample("c", 20.0, current, now - HOUR)?,
    ];
    let reports = [
      report("a", now, 5.0, current)?,
      report("b", now, 0.0, current)?,
      report("c", now, 20.0, current)?,
    ];
    let said = view::text(&project(&history, &reports, now, cap, None)?, now);
    let fell: Vec<&str> = said.lines().filter(|line| line.contains("fell")).collect();
    // ISSUE:0010 item 11: in durations, as predict's other lines give times, for a window
    // that has reset and for one that has not.
    assert_eq!(
      fell,
      [
        "a: weekly usage fell from 38% 20h00 ago to 9% 17h00 ago, in the window that reset 3h00 ago, as the usage endpoint reported it; the rate counts a fall as no use",
        "c: weekly usage fell from 30% 2h00 ago to 20% 1h00 ago, in the window that resets in 6d21h, as the usage endpoint reported it; the rate counts a fall as no use"
      ],
      "{said}"
    );
    let steady = view::text(&project(&history[..2], &reports, now, cap, None)?, now);
    assert!(!steady.contains("fell"), "{steady}");
    Ok(())
  }
}
