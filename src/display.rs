use crate::{
  Result, auth,
  config::Store,
  usage::{Client, Report},
};
use crossterm::{
  cursor,
  event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
  execute,
  style::{Color, Print, ResetColor, SetForegroundColor},
  terminal::{self, Clear, ClearType},
};
use std::{
  io::{self, IsTerminal, Write},
  sync::mpsc,
  time::{Duration, Instant},
};

pub fn duration(seconds: i64) -> String {
  let minutes = seconds.max(0) / 60;
  if minutes < 1 {
    "<1m".into()
  } else if minutes < 60 {
    format!("{minutes}m")
  } else if minutes < 1440 {
    format!("{}h{:02}", minutes / 60, minutes % 60)
  } else {
    format!("{}d{:02}h", minutes / 1440, (minutes % 1440) / 60)
  }
}

fn reset_at(reset: Option<&str>, now: i64) -> String {
  reset
    .and_then(|text| {
      time::OffsetDateTime::parse(text, &time::format_description::well_known::Rfc3339).ok()
    })
    .map(|date| {
      if date.unix_timestamp() <= now {
        "due".into()
      } else {
        duration(date.unix_timestamp() - now)
      }
    })
    .unwrap_or_else(|| "—".into())
}

struct Span {
  text: String,
  color: Option<Color>,
}
type Line = Vec<Span>;
fn plain(text: impl Into<String>) -> Line {
  vec![Span {
    text: text.into(),
    color: None,
  }]
}
fn colored(text: impl Into<String>, color: Color) -> Line {
  vec![Span {
    text: text.into(),
    color: Some(color),
  }]
}

fn lines(
  reports: &[Report],
  active: Option<&str>,
  selected: Option<usize>,
  width: usize,
  now: i64,
  verbose: bool,
) -> Vec<Line> {
  let mut lines = vec![
    colored(
      "  CCUSAGE",
      Color::Rgb {
        r: 215,
        g: 119,
        b: 87,
      },
    ),
    plain(""),
  ];
  for (index, report) in reports.iter().enumerate() {
    let marker = if selected == Some(index) { "›" } else { " " };
    let active = if active == Some(report.name.as_str()) {
      " · active"
    } else {
      ""
    };
    lines.push(plain(format!(
      "{marker} {}{active}",
      crate::clean(&report.name)
    )));
    if let Some(snapshot) = &report.snapshot {
      lines.push(colored("  ▾ PLAN USAGE", Color::DarkGrey));
      let bar_width = width.saturating_sub(39).clamp(4, 36);
      let windows = match snapshot.windows() {
        Ok(windows) => windows,
        Err(error) => {
          lines.push(colored(
            format!("  {}", crate::clean(&error.to_string())),
            Color::Yellow,
          ));
          Vec::new()
        }
      };
      for (label, window) in windows {
        let label: String = label.chars().take(12).collect();
        let filled = ((window.utilization / 100.0) * bar_width as f64).round() as usize;
        let filled = filled.min(bar_width);
        let color = if window.utilization >= 90.0 {
          Color::Red
        } else if window.utilization >= 75.0 {
          Color::Yellow
        } else {
          Color::Green
        };
        let reset = reset_at(window.resets_at.as_deref(), now);
        let mut row = plain(format!("  {label:<12} "));
        if width >= 52 {
          row.push(Span {
            text: "━".repeat(filled),
            color: Some(color),
          });
          row.push(Span {
            text: "─".repeat(bar_width - filled),
            color: Some(Color::DarkGrey),
          });
          row.push(Span {
            text: "  ".into(),
            color: None,
          });
        }
        row.push(Span {
          text: format!("{:>3.0}%", window.utilization),
          color: Some(color),
        });
        row.push(Span {
          text: format!("  {reset:>8}"),
          color: Some(Color::DarkGrey),
        });
        lines.push(row);
      }
      if let Some(timestamp) = report.fetched_at {
        let date = time::OffsetDateTime::from_unix_timestamp(timestamp)
          .ok()
          .and_then(|date| {
            date
              .format(&time::format_description::well_known::Rfc3339)
              .ok()
          })
          .unwrap_or_else(|| timestamp.to_string());
        let status = if report.error.is_some() && report.cached {
          " · last known; refresh failed"
        } else if report.cached {
          " · cached"
        } else {
          ""
        };
        lines.push(colored(
          format!("  as of {date} · {} ago{status}", duration(now - timestamp)),
          Color::DarkGrey,
        ));
      }
    }
    if let Some(error) = &report.error {
      lines.push(colored(format!("  {}", crate::clean(error)), Color::Yellow));
    }
    if report.snapshot.is_none() && report.error.is_none() {
      lines.push(colored("  Loading usage…", Color::DarkGrey));
    }
    if verbose {
      append_details(&mut lines, report, width);
    }
    lines.push(plain(""));
  }
  lines
}

fn heading(key: &str) -> String {
  match key {
    "five_hour" => "5 hour window".into(),
    "seven_day" => "7 day window".into(),
    "seven_day_breakdown" => "Weekly breakdown".into(),
    "limits" => "Quota windows".into(),
    "extra_usage" => "Extra usage".into(),
    "spend" => "Usage credits".into(),
    "weekly_scoped_shares" => "Weekly model shares".into(),
    "fetched_at" => "Retrieved at".into(),
    "error" => "Refresh error".into(),
    "amount_minor" => "Amount (minor units)".into(),
    other if other.starts_with("seven_day_") => {
      format!("7 day {}", heading(other.trim_start_matches("seven_day_")))
    }
    other => {
      let text = crate::clean(other).replace('_', " ");
      let mut chars = text.chars();
      chars.next().map_or_else(String::new, |first| {
        format!("{}{}", first.to_uppercase(), chars.as_str())
      })
    }
  }
}

fn scalar(key: &str, value: &serde_json::Value) -> String {
  match value {
    serde_json::Value::Null => "Not provided".into(),
    serde_json::Value::Bool(true) => "Yes".into(),
    serde_json::Value::Bool(false) => "No".into(),
    serde_json::Value::String(text) => crate::clean(text),
    serde_json::Value::Number(number) => {
      if key == "fetched_at"
        && let Some(timestamp) = number.as_i64()
        && let Ok(date) = time::OffsetDateTime::from_unix_timestamp(timestamp)
        && let Ok(date) = date.format(&time::format_description::well_known::Rfc3339)
      {
        return format!("{date} ({timestamp})");
      }
      if key.contains("percent") || key == "utilization" {
        format!("{number}%")
      } else {
        number.to_string()
      }
    }
    _ => "None".into(),
  }
}

/// Wrap long metadata instead of cutting it off at the terminal's right edge.
fn wrap(text: &str, width: usize) -> Vec<String> {
  let mut chars: Vec<char> = text.chars().collect();
  let width = width.max(1);
  let mut output = Vec::new();
  while chars.len() > width {
    let split = chars[..width]
      .iter()
      .rposition(|c| c.is_whitespace())
      .filter(|index| *index > 0)
      .unwrap_or(width);
    output.push(chars[..split].iter().collect::<String>().trim_end().into());
    chars.drain(..split);
    while chars.first().is_some_and(|c| c.is_whitespace()) {
      chars.remove(0);
    }
  }
  output.push(chars.iter().collect());
  output
}

fn detail_value(
  lines: &mut Vec<Line>,
  key: &str,
  value: &serde_json::Value,
  depth: usize,
  width: usize,
) {
  let indent = (4 + depth * 2).min(width / 3);
  let label = heading(key);
  let prefix = " ".repeat(indent);
  match value {
    serde_json::Value::Object(fields) if !fields.is_empty() => {
      lines.push(colored(format!("{prefix}{label}"), Color::DarkGrey));
      for (key, value) in fields {
        detail_value(lines, key, value, depth + 1, width);
      }
    }
    serde_json::Value::Array(values) if !values.is_empty() => {
      lines.push(colored(format!("{prefix}{label}"), Color::DarkGrey));
      for (index, value) in values.iter().enumerate() {
        detail_value(
          lines,
          &format!("Entry {}", index + 1),
          value,
          depth + 1,
          width,
        );
      }
    }
    _ => {
      let label_width = 24.min(width.saturating_sub(indent + 1) / 2);
      let text = format!("{label:<label_width$} {}", scalar(key, value));
      for chunk in wrap(&text, width.saturating_sub(indent + 1)) {
        lines.push(plain(format!("{prefix}{chunk}")));
      }
    }
  }
}

fn append_details(lines: &mut Vec<Line>, report: &Report, width: usize) {
  let value = match serde_json::to_value(report) {
    Ok(value) => crate::usage::redact(&value, None),
    Err(error) => {
      lines.push(colored(
        format!("  Could not format details: {error}"),
        Color::Yellow,
      ));
      return;
    }
  };
  let Some(fields) = value.as_object() else {
    return;
  };
  lines.push(plain(""));
  lines.push(colored("  ▾ RETRIEVAL", Color::DarkGrey));
  for (key, value) in fields.iter().filter(|(key, _)| *key != "snapshot") {
    detail_value(lines, key, value, 0, width);
  }
  if let Some(snapshot) = value.get("snapshot").and_then(serde_json::Value::as_object) {
    let preferred = [
      "limits",
      "seven_day_breakdown",
      "spend",
      "extra_usage",
      "weekly_scoped_shares",
      "five_hour",
      "seven_day",
    ];
    let ordered = preferred
      .iter()
      .filter_map(|key| snapshot.get_key_value(*key))
      .chain(
        snapshot
          .iter()
          .filter(|(key, _)| !preferred.contains(&key.as_str())),
      );
    for (key, value) in ordered.filter(|(_, value)| value.is_object() || value.is_array()) {
      lines.push(plain(""));
      lines.push(colored(
        format!("  ▾ {}", heading(key).to_uppercase()),
        Color::DarkGrey,
      ));
      match value {
        serde_json::Value::Object(fields) if !fields.is_empty() => {
          for (key, value) in fields {
            detail_value(lines, key, value, 0, width);
          }
        }
        serde_json::Value::Array(entries) if !entries.is_empty() => {
          for (index, entry) in entries.iter().enumerate() {
            detail_value(lines, &format!("Entry {}", index + 1), entry, 0, width);
          }
        }
        _ => detail_value(lines, key, value, 0, width),
      }
    }
    let mut scalars = snapshot
      .iter()
      .filter(|(_, value)| !value.is_object() && !value.is_array())
      .peekable();
    if scalars.peek().is_some() {
      lines.push(plain(""));
      lines.push(colored("  ▾ OTHER DATA", Color::DarkGrey));
      for (key, value) in scalars {
        detail_value(lines, key, value, 0, width);
      }
    }
  }
}

fn page_lines(
  reports: &[Report],
  active: Option<&str>,
  selected: usize,
  detail: bool,
  width: usize,
  now: i64,
) -> Vec<Line> {
  if detail {
    lines(
      &reports[selected..=selected],
      active,
      None,
      width,
      now,
      true,
    )
  } else {
    lines(reports, active, Some(selected), width, now, false)
  }
}

fn write_line(output: &mut impl Write, line: &Line, width: usize, color: bool) -> io::Result<()> {
  let mut remaining = width;
  for span in line {
    let text: String = span.text.chars().take(remaining).collect();
    remaining = remaining.saturating_sub(text.chars().count());
    if color && let Some(color) = span.color {
      execute!(output, SetForegroundColor(color))?;
    }
    execute!(output, Print(text))?;
    if color {
      execute!(output, ResetColor)?;
    }
  }
  Ok(())
}

pub fn print(
  reports: &[Report],
  active: Option<&str>,
  no_color: bool,
  verbose: bool,
) -> Result<()> {
  let mut out = io::stdout().lock();
  let color = !no_color && io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none();
  let width = terminal::size().map_or(80, |(width, _)| usize::from(width));
  for line in lines(reports, active, None, width, crate::now(), verbose) {
    write_line(&mut out, &line, width, color)?;
    writeln!(out)?;
  }
  Ok(())
}

/// RAII restoration on errors and normal exits; panic hook handles unwinding too.
struct Terminal;
impl Terminal {
  fn enter() -> Result<Self> {
    terminal::enable_raw_mode()?;
    let guard = Self;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
    Ok(guard)
  }
}
impl Drop for Terminal {
  fn drop(&mut self) {
    let screen = execute!(
      io::stdout(),
      ResetColor,
      cursor::Show,
      terminal::LeaveAlternateScreen
    );
    let raw = terminal::disable_raw_mode();
    if let Err(error) = screen.and(raw) {
      eprintln!("ccusage: terminal restoration failed: {error}");
    }
  }
}

fn worker(path: std::path::PathBuf, names: Vec<String>, refresh: bool) -> mpsc::Receiver<Report> {
  let (sender, receiver) = mpsc::channel();
  std::thread::spawn(move || {
    let store = Store::load(path);
    let client = Client::default();
    for name in names {
      let report = match &store {
        Ok(store) => client.report(store, &name, refresh),
        Err(error) => Report::failed(&name, error),
      };
      if sender.send(report).is_err() {
        return;
      }
    }
  });
  receiver
}

pub fn tui(store: Option<&Store>, demo: bool, no_color: bool) -> Result<()> {
  if !io::stdout().is_terminal() || !io::stdin().is_terminal() {
    return Err("TUI needs an interactive terminal; use ccusage usage --all instead".into());
  }
  let names: Vec<String> = store
    .map(|store| store.config.profiles.keys().cloned().collect())
    .unwrap_or_default();
  if !demo && names.is_empty() {
    return Err("no profiles; run ccusage add NAME".into());
  }
  let mut reports: Vec<Report> = if demo {
    crate::usage::demo()
  } else {
    names
      .iter()
      .map(|name| Report {
        name: name.clone(),
        fetched_at: None,
        cached: false,
        snapshot: None,
        error: None,
      })
      .collect()
  };
  let mut receiver = store.map(|store| worker(store.path.clone(), names.clone(), false));
  let mut active = store.map(Store::active).transpose()?.flatten();
  let mut selected = 0;
  let mut detail = false;
  let mut overview_scroll = 0;
  let mut scroll: usize = 0;
  let mut notice = if demo {
    "Demo · no credentials or network requests".to_owned()
  } else {
    String::new()
  };
  let color = !no_color && std::env::var_os("NO_COLOR").is_none();
  let old_hook = std::panic::take_hook();
  std::panic::set_hook(Box::new(|info| {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(
      io::stdout(),
      ResetColor,
      cursor::Show,
      terminal::LeaveAlternateScreen
    );
    eprintln!("{info}");
  }));
  let result = (|| -> Result<()> {
    let _terminal = Terminal::enter()?;
    let mut last_paint = Instant::now() - Duration::from_secs(1);
    let mut dirty = true;
    loop {
      let mut finished = false;
      if let Some(channel) = &receiver {
        loop {
          match channel.try_recv() {
            Ok(report) => {
              if let Some(slot) = reports.iter_mut().find(|item| item.name == report.name) {
                *slot = report;
              }
              dirty = true;
            }
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
              finished = true;
              break;
            }
          }
        }
      }
      if finished {
        receiver = None;
      }
      if dirty || last_paint.elapsed() >= Duration::from_secs(30) {
        let (width, height) = terminal::size()?;
        let width = usize::from(width);
        let content_height = usize::from(height.saturating_sub(3));
        let frame = page_lines(
          &reports,
          active.as_deref(),
          selected,
          detail,
          width,
          crate::now(),
        );
        scroll = scroll.min(frame.len().saturating_sub(content_height));
        let mut out = io::stdout().lock();
        execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        for (row, line) in frame.iter().skip(scroll).take(content_height).enumerate() {
          execute!(out, cursor::MoveTo(0, row as u16))?;
          write_line(&mut out, line, width.saturating_sub(1), color)?;
        }
        if height >= 3 {
          let status = if detail && notice.is_empty() {
            format!(
              "  {} · details · lines {}–{} of {}",
              reports[selected].name,
              scroll + 1,
              (scroll + content_height).min(frame.len()),
              frame.len()
            )
          } else {
            notice.clone()
          };
          execute!(out, cursor::MoveTo(0, height - 3))?;
          write_line(
            &mut out,
            &colored(status, Color::Yellow),
            width.saturating_sub(1),
            color,
          )?;
          execute!(out, cursor::MoveTo(0, height - 2))?;
          write_line(
            &mut out,
            &colored(
              if detail {
                "  ↑↓ scroll · esc back · a activate · r refresh · q quit"
              } else {
                "  ↑↓ select · enter details · a activate · r refresh · q quit"
              },
              Color::DarkGrey,
            ),
            width.saturating_sub(1),
            color,
          )?;
        }
        out.flush()?;
        dirty = false;
        last_paint = Instant::now();
      }
      if !event::poll(Duration::from_millis(100))? {
        continue;
      }
      match event::read()? {
        Event::Key(key) if key.kind != KeyEventKind::Release => {
          match key.code {
            KeyCode::Char('q') => break,
            KeyCode::Esc if !detail => break,
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('b') if detail => {
              detail = false;
              scroll = overview_scroll;
            }
            KeyCode::Enter if !detail => {
              detail = true;
              overview_scroll = scroll;
              scroll = 0;
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
            KeyCode::Down | KeyCode::Char('j') => {
              if detail {
                scroll = scroll.saturating_add(1);
              } else {
                selected = (selected + 1).min(reports.len().saturating_sub(1));
                scroll_to(&reports, selected, &mut scroll);
              }
            }
            KeyCode::Up | KeyCode::Char('k') => {
              if detail {
                scroll = scroll.saturating_sub(1);
              } else {
                selected = selected.saturating_sub(1);
                scroll_to(&reports, selected, &mut scroll);
              }
            }
            KeyCode::PageDown => {
              scroll = scroll.saturating_add(10);
            }
            KeyCode::PageUp => {
              scroll = scroll.saturating_sub(10);
            }
            KeyCode::Home => {
              scroll = 0;
            }
            KeyCode::End => {
              scroll = usize::MAX;
            }
            KeyCode::Char('a') if !demo => {
              if let Some(store) = store {
                let name = &reports[selected].name;
                let result = store
                  .profile_dir(name)
                  .and_then(|dir| auth::read(&dir))
                  .and_then(|_| store.activate(name));
                match result {
                  Ok(()) => {
                    active = Some(name.clone());
                    notice = format!("{name} selected for future launches");
                  }
                  Err(error) => {
                    notice = crate::clean(&error.to_string());
                  }
                }
              }
            }
            KeyCode::Char('r') if !demo => {
              if receiver.is_some() {
                notice = "Refresh already in progress".into();
              } else if let Some(store) = store {
                let targets = if detail {
                  vec![reports[selected].name.clone()]
                } else {
                  names.clone()
                };
                receiver = Some(worker(store.path.clone(), targets, true));
                notice = if detail {
                  "Refreshing this account"
                } else {
                  "Refreshing accounts"
                }
                .into();
              }
            }
            _ => {}
          }
          dirty = true;
        }
        Event::Resize(_, _) => {
          dirty = true;
        }
        _ => {}
      }
    }
    Ok(())
  })();
  std::panic::set_hook(old_hook);
  result
}

fn scroll_to(reports: &[Report], selected: usize, scroll: &mut usize) {
  // Keep the selected account header visible even with a small terminal.
  let preceding = lines(&reports[..selected], None, None, 80, crate::now(), false).len();
  *scroll = preceding;
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn durations_and_unknown_resets() {
    assert_eq!(duration(-10), "<1m");
    assert_eq!(duration(10800), "3h00");
    assert_eq!(duration(489600), "5d16h");
    assert_eq!(reset_at(None, 0), "—");
  }
  #[test]
  fn narrow_rows_preserve_percent_and_reset() -> Result<()> {
    let frame = lines(&crate::usage::demo(), None, None, 40, crate::now(), false);
    let mut output = Vec::new();
    for line in frame {
      write_line(&mut output, &line, 40, false)?;
      output.push(b'\n');
    }
    let text = String::from_utf8(output)?;
    assert!(text.contains("100%"));
    assert!(!text.contains('\x1b'));
    Ok(())
  }
  fn text(frame: Vec<Line>, width: usize) -> Result<String> {
    let mut output = Vec::new();
    for line in frame {
      write_line(&mut output, &line, width, false)?;
      output.push(b'\n');
    }
    Ok(String::from_utf8(output)?)
  }

  #[test]
  fn detail_page_contains_only_selected_account_and_uses_verbose_renderer() -> Result<()> {
    let mut reports = crate::usage::demo();
    let mut other = reports[0].clone();
    other.name = "personal-only".into();
    other.snapshot = Some(serde_json::from_str(include_str!(
      "../tests/fixtures/usage-with-breakdown.json"
    ))?);
    reports.push(other);
    let now = crate::now();
    let detail = text(page_lines(&reports, None, 1, true, 80, now), 80)?;
    assert!(detail.contains("personal-only"));
    assert!(!detail.contains("work (demo)"));
    assert!(detail.contains("WEEKLY BREAKDOWN"));
    assert!(detail.contains("Example model"));
    assert_eq!(
      detail,
      text(lines(&reports[1..], None, None, 80, now, true), 80)?
    );
    let overview = text(page_lines(&reports, None, 1, false, 80, now), 80)?;
    assert!(overview.contains("work (demo)"));
    assert!(overview.contains("personal-only"));
    assert!(!overview.contains("WEEKLY BREAKDOWN"));
    Ok(())
  }

  #[test]
  fn details_wrap_long_values_and_preserve_false_zero_and_null() -> Result<()> {
    let report = Report {
      name: "work".into(),
      fetched_at: Some(1),
      cached: false,
      error: None,
      snapshot: Some(serde_json::from_value(serde_json::json!({
        "five_hour":{"utilization":0,"resets_at":null},
        "metadata":{"enabled":false,"note":"A long explanatory message that must wrap and remain completely readable", "zero":0,"absent":null}
      }))?),
    };
    let frame = lines(&[report], None, None, 40, 1, true);
    for line in &frame {
      assert!(
        line
          .iter()
          .map(|span| span.text.chars().count())
          .sum::<usize>()
          <= 40
      );
    }
    let rendered = text(frame, 40)?;
    assert!(rendered.contains("No"));
    let words = rendered.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(words.contains("Not provided"));
    assert!(words.contains("completely readable"));
    assert!(rendered.contains("0%"));
    Ok(())
  }
}
