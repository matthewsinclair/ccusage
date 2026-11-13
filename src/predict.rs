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
use std::collections::BTreeMap;
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
  let counted: Vec<Sample> = samples
    .iter()
    .filter(|sample| on_plan(sample.account.as_str()))
    .cloned()
    .collect();
  let rate = rate(&counted);
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
  let Some(per_hour) = rate.per_hour else {
    return Ok(Projection {
      plan,
      rate,
      accounts: Vec::new(),
      left_out,
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

  /// A weekly sample of `account` at `percent`, on the window that resets at `reset`.
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
    assert_eq!(
      projection.left_out,
      [(
        "c".to_owned(),
        format!("another plan's allowance ({})", described(&pro))
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
}
