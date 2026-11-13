//! Running Claude Code sessions, and the two variables of each one's launch environment
//! that decide whose login it reads: CLAUDE_CONFIG_DIR and
//! CLAUDE_SECURESTORAGE_CONFIG_DIR. Nothing else of any environment is kept, and on
//! Linux only a claude process's environment is read.

use std::{
  path::Path,
  process::{Command, Stdio},
};

/// A running Claude Code session, as doctor reports it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Session {
  pub pid: u32,
  /// Its start time as ps gives it, when ps gives one.
  pub started: Option<String>,
  pub config_dir: Option<String>,
  pub login_dir: Option<String>,
}

const CONFIG_DIR: &str = "CLAUDE_CONFIG_DIR";
const LOGIN_DIR: &str = crate::shared::LOGIN_DIR;
/// The program a Claude Code session runs as.
const CLAUDE: &str = "claude";

/// ps, which shows each process's environment on macOS. CCUSAGE_PS names a stand-in,
/// which the CLI tests always set, as CCUSAGE_LAUNCHCTL does for launchctl, so no test
/// lists the machine's real sessions. None elsewhere, where /proc is read instead.
fn ps() -> Option<Command> {
  match std::env::var_os("CCUSAGE_PS") {
    Some(stand_in) => Some(Command::new(stand_in)),
    None if cfg!(target_os = "macos") => Some(Command::new("/bin/ps")),
    None => None,
  }
}

/// The running Claude Code sessions, or None when this system's process listing cannot
/// be read with its environments, so the check is skipped.
pub(crate) fn running() -> Option<Vec<Session>> {
  let Some(mut ps) = ps() else {
    return from_proc();
  };
  let output = ps
    .args(["-E", "-ww", "-A", "-o", "pid=,lstart=,args="])
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  Some(
    String::from_utf8_lossy(&output.stdout)
      .lines()
      .filter_map(parse)
      .collect(),
  )
}

/// One line of `ps -E -o pid=,lstart=,args=` as a Claude session, if it is one: its pid,
/// its start time (lstart's five words) and the two variables, from the words after its
/// arguments. A value with a space in it is cut at the space. Pure.
fn parse(line: &str) -> Option<Session> {
  let mut words = line.split_whitespace();
  let pid = words.next()?.parse().ok()?;
  let started = words.by_ref().take(5).collect::<Vec<_>>().join(" ");
  let rest: Vec<&str> = words.collect();
  if Path::new(rest.first()?).file_name()? != CLAUDE {
    return None;
  }
  let environment = || rest.iter().map(|word| word.as_bytes());
  Some(Session {
    pid,
    started: Some(started),
    config_dir: variable(environment(), CONFIG_DIR),
    login_dir: variable(environment(), LOGIN_DIR),
  })
}

/// The value of `name` among `entries`, each a `NAME=value` as an environment holds
/// them. Pure.
fn variable<'a>(mut entries: impl Iterator<Item = &'a [u8]>, name: &str) -> Option<String> {
  entries
    .find_map(|entry| entry.strip_prefix(name.as_bytes())?.strip_prefix(b"="))
    .map(|value| String::from_utf8_lossy(value).into_owned())
}

/// A session's working directory: from lsof on macOS (CCUSAGE_LSOF names a stand-in for
/// the tests), else from /proc. None when it cannot be read. Doctor reads it only for a
/// session it reports.
pub(crate) fn cwd(pid: u32) -> Option<String> {
  let lsof = match std::env::var_os("CCUSAGE_LSOF") {
    Some(stand_in) => stand_in,
    None if cfg!(target_os = "macos") => "/usr/sbin/lsof".into(),
    None => {
      return std::fs::read_link(format!("/proc/{pid}/cwd"))
        .ok()
        .map(|path| path.display().to_string());
    }
  };
  let output = Command::new(lsof)
    .args(["-a", "-d", "cwd", "-Fn", "-p"])
    .arg(pid.to_string())
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .output()
    .ok()?;
  String::from_utf8_lossy(&output.stdout)
    .lines()
    .find_map(|line| line.strip_prefix('n').map(str::to_owned))
}

/// The running Claude sessions from /proc, on Linux; None where there is no /proc.
fn from_proc() -> Option<Vec<Session>> {
  let entries = std::fs::read_dir("/proc").ok()?;
  let mut sessions = Vec::new();
  for entry in entries.flatten() {
    let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
      continue;
    };
    let dir = entry.path();
    if !std::fs::read_to_string(dir.join("comm")).is_ok_and(|comm| comm.trim_end() == CLAUDE) {
      continue;
    }
    // Another user's process cannot be read, and is not this user's session.
    let Ok(environ) = std::fs::read(dir.join("environ")) else {
      continue;
    };
    let environment = || environ.split(|byte| *byte == 0);
    sessions.push(Session {
      pid,
      started: started_linux(pid),
      config_dir: variable(environment(), CONFIG_DIR),
      login_dir: variable(environment(), LOGIN_DIR),
    });
  }
  Some(sessions)
}

/// A process's start time on Linux, from ps; None when ps gives none.
fn started_linux(pid: u32) -> Option<String> {
  let output = Command::new("ps")
    .args(["-o", "lstart=", "-p"])
    .arg(pid.to_string())
    .stdin(Stdio::null())
    .stderr(Stdio::null())
    .output()
    .ok()?;
  let started = String::from_utf8_lossy(&output.stdout).trim().to_owned();
  (!started.is_empty()).then_some(started)
}

/// What makes `session` read a login other than the shared config's: a login directory,
/// or a config directory, other than the shared config. None when it follows a switch.
/// Pure.
pub(crate) fn drift(session: &Session, shared: &Path) -> Option<String> {
  fn other<'a>(value: &'a Option<String>, shared: &Path) -> Option<&'a str> {
    value.as_deref().filter(|value| Path::new(value) != shared)
  }
  match (
    other(&session.login_dir, shared),
    other(&session.config_dir, shared),
  ) {
    (Some(dir), _) => Some(format!("{LOGIN_DIR}={dir}")),
    (None, Some(dir)) => Some(format!("{CONFIG_DIR}={dir}")),
    (None, None) => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // ST0003 AT-04.3: a ps line is a Claude session only for the claude program, and only
  // its two variables are kept; a session drifts when either names a directory other
  // than the shared config.
  #[test]
  fn sessions_keep_only_their_two_variables_and_drift_off_the_shared_config() -> crate::Result<()> {
    let shared = Path::new("/home/u/.config/claude");
    let line = " 102 Fri Nov 13 08:00:01 2026 /opt/claude --resume CLAUDE_CONFIG_DIR=/home/u/.config/claude/ ANTHROPIC_API_KEY=SECRET CLAUDE_SECURESTORAGE_CONFIG_DIR=/p/work";
    let session = parse(line).ok_or("a claude line is a session")?;
    assert_eq!(
      session,
      Session {
        pid: 102,
        started: Some("Fri Nov 13 08:00:01 2026".into()),
        config_dir: Some("/home/u/.config/claude/".into()),
        login_dir: Some("/p/work".into()),
      }
    );
    assert_eq!(
      drift(&session, shared).as_deref(),
      Some("CLAUDE_SECURESTORAGE_CONFIG_DIR=/p/work")
    );
    let follows =
      parse(" 101 Fri Nov 13 08:00:00 2026 claude CLAUDE_CONFIG_DIR=/home/u/.config/claude")
        .ok_or("a claude line is a session")?;
    assert_eq!(drift(&follows, shared), None);
    let elsewhere = parse(" 103 Fri Nov 13 08:00:02 2026 claude CLAUDE_CONFIG_DIR=/elsewhere")
      .ok_or("a claude line is a session")?;
    assert_eq!(
      drift(&elsewhere, shared).as_deref(),
      Some("CLAUDE_CONFIG_DIR=/elsewhere")
    );
    assert_eq!(
      parse(" 104 Fri Nov 13 08:00:03 2026 vim CLAUDE_CONFIG_DIR=/elsewhere"),
      None
    );
    // An environment as /proc holds it, entries split at nul bytes; a longer name that
    // starts with a variable's name is not that variable.
    let environ = b"CLAUDE_CONFIG_DIRS=/no\0CLAUDE_CONFIG_DIR=/yes\0".as_slice();
    assert_eq!(
      variable(environ.split(|byte| *byte == 0), CONFIG_DIR).as_deref(),
      Some("/yes")
    );
    Ok(())
  }
}
