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
/// Why the running sessions were not checked, as doctor and a move say it.
pub(crate) const UNLISTED: &str = "this system's process listing could not be read";

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
/// program. ps prints the environment after the arguments, so each variable is its last
/// `NAME=value`: the environment's, ahead of an argument that looks like one. A value
/// with a space in it is cut at the space. Pure.
fn parse(line: &str) -> Option<Session> {
  let mut words = line.split_whitespace();
  let pid = words.next()?.parse().ok()?;
  let started = words.by_ref().take(5).collect::<Vec<_>>().join(" ");
  let rest: Vec<&str> = words.collect();
  if Path::new(rest.first()?).file_name()? != CLAUDE {
    return None;
  }
  let environment = || rest.iter().rev().map(|word| word.as_bytes());
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

/// The directory whose credential store `session` reads its login from: its login
/// directory, else its config directory; None when it names neither. Pure.
fn login_store(session: &Session) -> Option<&Path> {
  session
    .login_dir
    .as_deref()
    .or(session.config_dir.as_deref())
    .map(Path::new)
}

/// The sessions that read their login from `dir`'s store, so are pinned to it: a move
/// that takes the login out of that store leaves them none. Pure.
pub(crate) fn pinned_to<'a>(sessions: &'a [Session], dir: &Path) -> Vec<&'a Session> {
  sessions
    .iter()
    .filter(|session| login_store(session) == Some(dir))
    .collect()
}

/// The sessions pinned to `name`'s own store, named by pid, or that there is none. Pure.
fn reading(name: &str, pinned: &[&Session]) -> String {
  let pids: Vec<String> = pinned
    .iter()
    .map(|session| session.pid.to_string())
    .collect();
  match pids.as_slice() {
    [] => format!("no running Claude session reads {name}'s own store"),
    [one] => format!("Claude session {one} reads {name}'s own store"),
    [rest @ .., last] => format!(
      "Claude sessions {} and {last} read {name}'s own store",
      rest.join(", ")
    ),
  }
}

/// The remedy that `ccusage login NAME` ends, in the order that works: while sessions
/// are pinned to `name`'s own store they restart first, from a shell without the login
/// directory, since the login would empty that store under them and a /login typed in
/// one of them puts the fault back; with none pinned, the login alone. `pinned` is None
/// when the running sessions could not be listed. Pure.
pub(crate) fn remedy(name: &str, pinned: Option<&[&Session]>) -> String {
  match pinned {
    None => format!(
      "the running Claude sessions could not be listed, so first restart any started with {LOGIN_DIR} naming {name}'s own store from a shell without it, and only then run ccusage login {name}"
    ),
    Some([]) => format!("{}, so run ccusage login {name}", reading(name, &[])),
    Some(pinned) => format!(
      "{}, so first restart {} from a shell without {LOGIN_DIR} (unset {LOGIN_DIR}), and only then run ccusage login {name}",
      reading(name, pinned),
      if pinned.len() == 1 { "it" } else { "each" }
    ),
  }
}

/// What `what`, a move that would empty a profile's own store, says on stderr when it
/// goes ahead with no running session checked, since none could be listed. Pure.
pub(crate) fn unchecked(what: &str) -> String {
  format!("running Claude sessions were not checked before {what}: {UNLISTED}")
}

/// What which, predict and the TUI say while running sessions read a login other than the
/// shared config's (`drift`), so that a switch does not move them: how many, and that
/// doctor names them. None when every one follows. Pure.
pub(crate) fn unfollowing(sessions: &[Session], shared: &Path) -> Option<String> {
  match sessions
    .iter()
    .filter(|session| drift(session, shared).is_some())
    .count()
  {
    0 => None,
    1 => Some(
      "1 running Claude session is pinned to a login other than the shared config's, so a switch does not move it; ccusage doctor names it"
        .to_owned(),
    ),
    count => Some(format!(
      "{count} running Claude sessions are pinned to logins other than the shared config's, so a switch does not move them; ccusage doctor names them"
    )),
  }
}

/// `unfollowing` for the sessions running now; None as well when they cannot be listed,
/// which doctor reports.
pub(crate) fn unfollowing_now(shared: &Path) -> Option<String> {
  unfollowing(&running()?, shared)
}

/// Why `what`, a move that would empty `name`'s own store, is refused while `pinned`
/// sessions read their login there. Pure.
pub(crate) fn refusal(name: &str, pinned: &[&Session], what: &str) -> String {
  let them = if pinned.len() == 1 { "it" } else { "them" };
  format!(
    "{}, and {what} would leave it without a login under {them}; nothing was moved; restart {them} from a shell without {LOGIN_DIR} (unset {LOGIN_DIR}), then try again",
    reading(name, pinned)
  )
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
    // ps prints the environment after the arguments, so an argument that looks like one
    // of the variables gives way to the environment's own.
    let argued = parse(
      " 105 Fri Nov 13 08:00:04 2026 claude --add-dir CLAUDE_CONFIG_DIR=/argument CLAUDE_CONFIG_DIR=/environment",
    )
    .ok_or("a claude line is a session")?;
    assert_eq!(argued.config_dir.as_deref(), Some("/environment"));
    // An environment as /proc holds it, entries split at nul bytes; a longer name that
    // starts with a variable's name is not that variable.
    let environ = b"CLAUDE_CONFIG_DIRS=/no\0CLAUDE_CONFIG_DIR=/yes\0".as_slice();
    assert_eq!(
      variable(environ.split(|byte| *byte == 0), CONFIG_DIR).as_deref(),
      Some("/yes")
    );
    Ok(())
  }

  // ISSUE:0008 Acceptance 2 and 3: a session is pinned to the store it reads its login
  // from, its login directory's, else its config directory's; a remedy puts the restart
  // of the pinned sessions before ccusage login NAME, and a refusal names them.
  #[test]
  fn a_session_is_pinned_to_the_store_it_reads_its_login_from() -> crate::Result<()> {
    let (shared, own) = ("/home/u/.config/claude", "/data/profiles/work");
    let lines = [
      format!(" 201 Mon Nov 16 16:30:00 2026 claude {CONFIG_DIR}={shared} {LOGIN_DIR}={own}"),
      format!(" 202 Mon Nov 16 16:30:01 2026 claude {CONFIG_DIR}={own}/"),
      format!(" 203 Mon Nov 16 16:30:02 2026 claude {CONFIG_DIR}={shared}"),
      format!(" 204 Mon Nov 16 16:30:03 2026 claude {CONFIG_DIR}={own} {LOGIN_DIR}={shared}"),
      " 205 Mon Nov 16 16:30:04 2026 claude".to_owned(),
    ];
    let sessions: Vec<Session> = lines.iter().filter_map(|line| parse(line)).collect();
    assert_eq!(sessions.len(), lines.len());
    let pinned = pinned_to(&sessions, Path::new(own));
    let pids: Vec<u32> = pinned.iter().map(|session| session.pid).collect();
    assert_eq!(pids, [201, 202]);
    let one = &pinned[..1];
    for (remedy, restart) in [
      (
        remedy("work", Some(&pinned)),
        "Claude sessions 201 and 202 read work's own store, so first restart each from a shell without CLAUDE_SECURESTORAGE_CONFIG_DIR",
      ),
      (
        remedy("work", Some(one)),
        "Claude session 201 reads work's own store, so first restart it from a shell without CLAUDE_SECURESTORAGE_CONFIG_DIR",
      ),
    ] {
      assert!(remedy.starts_with(restart), "{remedy}");
      assert!(
        remedy.ends_with("and only then run ccusage login work"),
        "{remedy}"
      );
    }
    assert_eq!(
      remedy("work", Some(&[])),
      "no running Claude session reads work's own store, so run ccusage login work"
    );
    assert!(remedy("work", None).ends_with("and only then run ccusage login work"));
    let refused = refusal("work", &pinned, "switching to work");
    assert!(
      refused.starts_with("Claude sessions 201 and 202 read work's own store, and switching to work would leave it without a login under them; nothing was moved"),
      "{refused}"
    );
    Ok(())
  }
}
