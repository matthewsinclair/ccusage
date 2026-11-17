//! The predict verb's two faces of one `Projection`: the text hv reads, and the JSON the
//! Herdr panel reads, versioned as which's is. Both are pure.

use super::{Fall, Projected, Projection, described, hours};
use crate::{
  Result,
  display::{self, stamp_unix, tenth, whole},
  usage::Plan,
};
use serde::Serialize;

/// The JSON's version. A field added keeps it; a field removed, renamed or given a new
/// meaning raises it.
pub const VERSION: u32 = 1;

/// The text: what the rate rests on, then each account's percent now, at its reset and
/// unused there, then the total and the plan it is counted on; or, with too little
/// history, a line saying so. The accounts left out follow, each with why, and then each
/// weekly window whose usage fell. The JSON gives no falls: its shape is ic's to change.
pub fn text(projection: &Projection, now: i64) -> String {
  let rate = &projection.rate;
  let history = format!(
    "{} samples over {} of history",
    rate.samples,
    display::duration(rate.span)
  );
  let mut lines = Vec::new();
  match (rate.per_hour, projection.unused()) {
    (Some(per_hour), Some(unused)) => {
      lines.push(format!(
        "Burn rate: {per_hour:.1}% of one weekly allowance an hour, from {history}."
      ));
      lines.extend(projection.accounts.iter().map(|account| {
        format!(
          "{}: {}% used now, {}% at its reset in {}, {}% unused",
          crate::clean(&account.name),
          whole(account.percent),
          whole(account.at_reset),
          display::duration(account.resets_at - now),
          whole(account.unused)
        )
      }));
      let plan = projection
        .plan
        .as_ref()
        .map(|plan| format!(" on {}", described(plan)))
        .unwrap_or_default();
      lines.push(format!(
        "Projected unused at the current resets: {}% of one weekly allowance{plan}.",
        whole(unused)
      ));
    }
    _ => lines.push(format!(
      "Too little history to project: {history}; predict needs a day of it, with some use in it."
    )),
  }
  lines.extend(
    projection
      .left_out
      .iter()
      .map(|(name, why)| format!("{}: left out, {why}", crate::clean(name))),
  );
  lines.extend(projection.falls.iter().map(fell));
  lines.join("\n")
}

/// A weekly window whose usage fell, in words: from what to what, when, in which window,
/// and that the figures are the usage endpoint's own.
fn fell(fall: &Fall) -> String {
  let at = |seconds: i64| stamp_unix(seconds).unwrap_or_else(|_| seconds.to_string());
  let window = fall
    .resets_at
    .map(|reset| format!(", in the window ending {}", at(reset)))
    .unwrap_or_default();
  format!(
    "{}: weekly usage fell from {}% at {} to {}% at {}{window}, as the usage endpoint reported it; the rate counts a fall as no use",
    crate::clean(&fall.account),
    whole(fall.earlier.0),
    at(fall.earlier.1),
    whole(fall.newest.0),
    at(fall.newest.1)
  )
}

/// The JSON, version 1: the plan counted, the rate and what it rests on, each account's
/// projection, the total unused, and the accounts left out. With too little history the
/// rate's percent_per_hour and the total are null, and no account is projected; the plan
/// is null only when there is no account at all.
pub fn json(projection: &Projection, now: i64, cap: f64) -> Result<String> {
  let document = Document {
    version: VERSION,
    now: stamp_unix(now)?,
    cap,
    plan: projection.plan.as_ref(),
    rate: Rate {
      percent_per_hour: projection.rate.per_hour.map(tenth),
      samples: projection.rate.samples,
      hours: tenth(hours(projection.rate.span)),
    },
    accounts: projection
      .accounts
      .iter()
      .map(entry)
      .collect::<Result<_>>()?,
    unused: projection.unused().map(tenth),
    left_out: projection
      .left_out
      .iter()
      .map(|(name, reason)| LeftOut { name, reason })
      .collect(),
  };
  Ok(serde_json::to_string_pretty(&document)?)
}

fn entry(account: &Projected) -> Result<Entry<'_>> {
  Ok(Entry {
    name: &account.name,
    percent: account.percent,
    resets_at: stamp_unix(account.resets_at)?,
    projected: tenth(account.at_reset),
    unused: tenth(account.unused),
  })
}

#[derive(Serialize)]
struct Document<'a> {
  version: u32,
  now: String,
  cap: f64,
  plan: Option<&'a Plan>,
  rate: Rate,
  accounts: Vec<Entry<'a>>,
  unused: Option<f64>,
  left_out: Vec<LeftOut<'a>>,
}

#[derive(Serialize)]
struct Rate {
  percent_per_hour: Option<f64>,
  samples: usize,
  hours: f64,
}

#[derive(Serialize)]
struct Entry<'a> {
  name: &'a str,
  percent: f64,
  resets_at: String,
  projected: f64,
  unused: f64,
}

#[derive(Serialize)]
struct LeftOut<'a> {
  name: &'a str,
  reason: &'a str,
}
