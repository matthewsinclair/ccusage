//! Usage history: for each account, a sample of each usage window's percent each time it
//! changes, kept for one weekly window in a private file in the state directory. Every
//! successful fetch records through `record`, and predict reads `samples` back.

use crate::{
  Result, storage,
  usage::{Kind, Snapshot, WEEK, Window},
};
use serde::{Deserialize, Serialize};
use std::{
  fmt::Display,
  fs,
  path::Path,
  thread,
  time::{Duration, Instant},
};

/// The history's file in the state directory: one sample a line, oldest first.
pub const FILE: &str = "history.jsonl";
/// The file a writer locks while it reads the history and writes it back. It is never
/// removed, so every writer locks the same file.
const LOCK: &str = "history.lock";
/// How long a writer waits for another to finish before its own fetch goes unrecorded.
const LOCK_WAIT: Duration = Duration::from_secs(5);
/// How often a waiting writer tries the lock again.
const LOCK_POLL: Duration = Duration::from_millis(10);

/// One window's percent for one account, as it stood when it was sampled.
#[derive(Clone, Debug)]
pub struct Sample {
  pub account: String,
  pub window: Window,
  pub sampled_at: i64,
}

/// A sample as the file holds it: the window's numbers, and nothing else from the
/// response. A reader passes over a field it does not know.
#[derive(Deserialize, Serialize)]
struct Line {
  account: String,
  kind: String,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  model: Option<String>,
  percent: f64,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  share: Option<f64>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  resets_at: Option<String>,
  sampled_at: i64,
}

impl Line {
  fn of(account: &str, window: &Window, sampled_at: i64) -> Result<Self> {
    Ok(Self {
      account: account.into(),
      kind: window.kind.name().into(),
      model: window.kind.model().map(Into::into),
      percent: window.utilization,
      share: window.share,
      resets_at: window
        .resets_at
        .map(|reset| reset.format(&time::format_description::well_known::Rfc3339))
        .transpose()?,
      sampled_at,
    })
  }
}

/// A line read back, with its window when this build knows the line's kind. A line of a
/// kind it does not know, as a later build may write, is kept as it is and never matched.
struct Kept {
  line: Line,
  window: Option<Window>,
}

/// Records `account`'s sample of each of `snapshot`'s windows at `sampled_at`: a
/// window's first, then one each time its percent differs from the last one kept. A
/// window is the same window while its kind, and its reset to the minute, are unchanged.
/// Samples go by age alone, once older than one weekly window, and never at a reset:
/// what was used on a window that has since reset still counts toward the burn rate.
pub fn record(state: &Path, account: &str, snapshot: &Snapshot, sampled_at: i64) -> Result<()> {
  let windows = snapshot.windows()?;
  storage::private_dir(state)?;
  let _lock = lock(&state.join(LOCK))?;
  let path = state.join(FILE);
  let mut kept = read(&path)?;
  let mut changed = false;
  for window in windows {
    let last = kept.iter().rev().find(|old| {
      old.line.account == account
        && old
          .window
          .as_ref()
          .is_some_and(|known| same_window(known, &window))
    });
    if last.is_none_or(|last| last.line.percent != window.utilization) {
      kept.push(Kept {
        line: Line::of(account, &window, sampled_at)?,
        window: Some(window),
      });
      changed = true;
    }
  }
  let count = kept.len();
  kept.retain(|old| sampled_at.saturating_sub(old.line.sampled_at) <= WEEK);
  if changed || kept.len() != count {
    kept.sort_by_key(|old| old.line.sampled_at);
    write(&path, &kept)?;
  }
  Ok(())
}

/// Every sample in the history under `state`, oldest first, of the kinds this build
/// knows; none before a first fetch has been recorded.
pub fn samples(state: &Path) -> Result<Vec<Sample>> {
  Ok(
    read(&state.join(FILE))?
      .into_iter()
      .filter_map(|kept| {
        kept.window.map(|window| Sample {
          account: kept.line.account,
          window,
          sampled_at: kept.line.sampled_at,
        })
      })
      .collect(),
  )
}

/// Whether two windows are the one window: the same kind, for the same model when
/// scoped, with the same reset to the minute.
fn same_window(a: &Window, b: &Window) -> bool {
  a.kind == b.kind && a.reset_minute() == b.reset_minute()
}

/// Every line of the history at `path`, oldest first; none when there is no file yet. A
/// line that cannot be read refuses the file, naming the line but never its contents.
fn read(path: &Path) -> Result<Vec<Kept>> {
  let Some(bytes) = storage::read_optional(path)? else {
    return Ok(Vec::new());
  };
  let text = String::from_utf8(bytes).map_err(|_| unreadable(path, "it is not text"))?;
  let mut kept = Vec::new();
  for (index, raw) in text.lines().enumerate() {
    if raw.trim().is_empty() {
      continue;
    }
    let number = index + 1;
    let line: Line = serde_json::from_str(raw)
      .map_err(|_| unreadable(path, format!("line {number} is not a sample")))?;
    let window = Kind::from_name(&line.kind, line.model.as_deref())
      .map(|kind| {
        Window::checked(
          kind,
          line.percent,
          line.resets_at.as_deref(),
          line.share,
          &format!("line {number}"),
        )
      })
      .transpose()
      .map_err(|error| unreadable(path, error))?;
    kept.push(Kept { line, window });
  }
  kept.sort_by_key(|kept| kept.line.sampled_at);
  Ok(kept)
}

fn unreadable(path: &Path, why: impl Display) -> crate::Error {
  format!(
    "could not read the usage history {}: {why}; remove it to start the history again",
    path.display()
  )
  .into()
}

/// Writes `kept` back as the whole file, staged beside it so a reader never sees part.
fn write(path: &Path, kept: &[Kept]) -> Result<()> {
  let mut text = String::new();
  for kept in kept {
    text.push_str(&serde_json::to_string(&kept.line)?);
    text.push('\n');
  }
  storage::write_atomic(path, text.as_bytes())
}

/// Holds the history's lock until it is dropped, waiting at most `LOCK_WAIT` for
/// another writer to finish.
fn lock(path: &Path) -> Result<fs::File> {
  let mut options = fs::OpenOptions::new();
  options.create(true).truncate(false).write(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
  }
  let file = options.open(path)?;
  let deadline = Instant::now() + LOCK_WAIT;
  loop {
    match file.try_lock() {
      Ok(()) => return Ok(file),
      Err(fs::TryLockError::WouldBlock) if Instant::now() < deadline => thread::sleep(LOCK_POLL),
      Err(fs::TryLockError::WouldBlock) => {
        return Err("another ccusage held the history's lock too long".into());
      }
      Err(fs::TryLockError::Error(error)) => return Err(error.into()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::scratch::Scratch;

  fn at(text: &str) -> Result<i64> {
    Ok(
      time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339)?
        .unix_timestamp(),
    )
  }

  /// Usage whose limits hold a weekly window at `weekly` resetting at `reset`, and a
  /// session window at `session` when one is given.
  fn usage(weekly: f64, reset: &str, session: Option<f64>) -> Result<Snapshot> {
    let mut limits =
      vec![serde_json::json!({"kind": "weekly_all", "percent": weekly, "resets_at": reset})];
    if let Some(session) = session {
      limits.push(
        serde_json::json!({"kind": "session", "percent": session, "resets_at": "2026-11-13T10:00:00Z"}),
      );
    }
    Ok(serde_json::from_value(
      serde_json::json!({"limits": limits}),
    )?)
  }

  /// Each sample as (account, kind, percent, sampled_at), oldest first.
  fn kept(state: &Path) -> Result<Vec<(String, Kind, f64, i64)>> {
    Ok(
      samples(state)?
        .into_iter()
        .map(|sample| {
          (
            sample.account,
            sample.window.kind,
            sample.window.utilization,
            sample.sampled_at,
          )
        })
        .collect(),
    )
  }

  // ST0002 AT-04.2: a window's first sample, then one for each changed percent.
  #[test]
  fn a_window_is_sampled_first_and_then_only_when_its_percent_changes() -> Result<()> {
    let scratch = Scratch::new()?;
    let state = scratch.path();
    let start = at("2026-11-13T05:27:00Z")?;
    let reset = "2026-11-16T10:00:00Z";
    let (weekly, session) = (20.0, 10.0);
    record(state, "work", &usage(weekly, reset, Some(session))?, start)?;
    let mut expected = vec![
      ("work".to_owned(), Kind::WeeklyAll, weekly, start),
      ("work".to_owned(), Kind::Session, session, start),
    ];
    assert_eq!(kept(state)?, expected);
    // An unchanged percent adds nothing.
    record(
      state,
      "work",
      &usage(weekly, reset, Some(session))?,
      start + 60,
    )?;
    assert_eq!(kept(state)?, expected);
    // A changed one adds its window's sample alone.
    let used = session + 5.0;
    record(
      state,
      "work",
      &usage(weekly, reset, Some(used))?,
      start + 120,
    )?;
    expected.push(("work".to_owned(), Kind::Session, used, start + 120));
    assert_eq!(kept(state)?, expected);
    // Another account's windows are its own.
    record(state, "personal", &usage(weekly, reset, None)?, start + 180)?;
    expected.push(("personal".to_owned(), Kind::WeeklyAll, weekly, start + 180));
    assert_eq!(kept(state)?, expected);
    Ok(())
  }

  // ST0002 AT-04.5: samples go after one weekly window, and not at a reset.
  #[test]
  fn samples_go_by_age_after_one_weekly_window_and_never_at_a_reset() -> Result<()> {
    let scratch = Scratch::new()?;
    let state = scratch.path();
    let start = at("2026-11-13T05:27:00Z")?;
    let percent = 90.0;
    // The last sample of a window that resets an hour later.
    record(
      state,
      "work",
      &usage(percent, "2026-11-13T06:27:00Z", None)?,
      start,
    )?;
    // After its reset: the next window, at the same percent, is a window of its own, and
    // the sample of the one that reset stays.
    let next = start + 2 * 3600;
    let following = "2026-11-20T06:27:00Z";
    record(state, "work", &usage(percent, following, None)?, next)?;
    let both = vec![
      ("work".to_owned(), Kind::WeeklyAll, percent, start),
      ("work".to_owned(), Kind::WeeklyAll, percent, next),
    ];
    assert_eq!(kept(state)?, both);
    // One weekly window on, the first sample is still kept; a second later it goes.
    record(
      state,
      "work",
      &usage(percent, following, None)?,
      start + WEEK,
    )?;
    assert_eq!(kept(state)?, both);
    record(
      state,
      "work",
      &usage(percent, following, None)?,
      start + WEEK + 1,
    )?;
    assert_eq!(kept(state)?, both[1..].to_vec());
    Ok(())
  }

  // ST0002 AT-04.3: the history is a 0600 file in the state directory.
  #[cfg(unix)]
  #[test]
  fn the_history_is_a_private_file_in_the_state_directory() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new()?;
    let state = scratch.path().join("state");
    record(
      &state,
      "work",
      &usage(1.0, "2026-11-16T10:00:00Z", None)?,
      at("2026-11-13T05:27:00Z")?,
    )?;
    let path = state.join(FILE);
    assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o7777, 0o600);
    assert!(storage::check_private_file(&path)?);
    Ok(())
  }

  #[test]
  fn writers_at_once_lose_no_sample() -> Result<()> {
    let scratch = Scratch::new()?;
    let state = scratch.path().to_path_buf();
    let start = at("2026-11-13T05:27:00Z")?;
    let (writers, rounds) = (8_usize, 25_u32);
    let threads: Vec<_> = (0..writers)
      .map(|writer| {
        let state = state.clone();
        thread::spawn(move || -> Result<()> {
          for round in 0..rounds {
            // Each round's percent differs from the last, so each keeps a sample.
            let snapshot = usage(f64::from(round), "2026-11-16T10:00:00Z", None)?;
            record(
              &state,
              &format!("account-{writer}"),
              &snapshot,
              start + i64::from(round),
            )?;
          }
          Ok(())
        })
      })
      .collect();
    for thread in threads {
      thread.join().map_err(|_| "a writer panicked")??;
    }
    assert_eq!(samples(&state)?.len(), writers * usize::try_from(rounds)?);
    Ok(())
  }

  #[test]
  fn a_line_holds_the_windows_numbers_and_nothing_else_from_the_response() -> Result<()> {
    let scratch = Scratch::new()?;
    let state = scratch.path();
    let snapshot: Snapshot =
      serde_json::from_str(include_str!("../tests/fixtures/usage-with-breakdown.json"))?;
    record(state, "work", &snapshot, at("2026-11-13T05:27:00Z")?)?;
    let text = fs::read_to_string(state.join(FILE))?;
    let fields = [
      "account",
      "kind",
      "model",
      "percent",
      "share",
      "resets_at",
      "sampled_at",
    ];
    for line in text.lines() {
      let value: serde_json::Value = serde_json::from_str(line)?;
      let object = value.as_object().ok_or("each line is an object")?;
      assert!(object.keys().all(|key| fields.contains(&key.as_str())));
    }
    assert_eq!(text.lines().count(), snapshot.windows()?.len());
    Ok(())
  }

  #[test]
  fn a_kind_this_build_does_not_know_is_kept_and_a_line_it_cannot_read_refuses() -> Result<()> {
    let scratch = Scratch::new()?;
    let state = scratch.path();
    let start = at("2026-11-13T05:27:00Z")?;
    let later =
      serde_json::json!({"account": "work", "kind": "monthly", "percent": 5, "sampled_at": start});
    storage::write_atomic(&state.join(FILE), format!("{later}\n").as_bytes())?;
    record(
      state,
      "work",
      &usage(1.0, "2026-11-16T10:00:00Z", None)?,
      start,
    )?;
    let text = fs::read_to_string(state.join(FILE))?;
    assert!(text.lines().any(|line| line.contains("\"monthly\"")));
    assert_eq!(kept(state)?.len(), text.lines().count() - 1);
    storage::write_atomic(&state.join(FILE), b"PRIVATE_TEXT\n")?;
    let error = record(
      state,
      "work",
      &usage(2.0, "2026-11-16T10:00:00Z", None)?,
      start,
    )
    .err()
    .ok_or("an unreadable history refuses the record")?
    .to_string();
    assert!(error.contains(FILE) && error.contains("line 1"));
    assert!(!error.contains("PRIVATE_TEXT"));
    Ok(())
  }
}
