//! Running Claude Code sessions, and the two variables of each one's launch environment
//! that decide whose login it reads: CLAUDE_CONFIG_DIR and
//! CLAUDE_SECURESTORAGE_CONFIG_DIR. Nothing else of any environment is kept, and on
//! Linux only a claude process's environment is read.

use std::{
  path::{Path, PathBuf},
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
  /// The stores its two directories name, which `named` gives it once it is listed.
  pub stores: Stores,
}

/// The stores a session's two directories name (`Named`), for comparing with a store's;
/// the directories as written are for showing.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Stores {
  pub config: Option<Named>,
  pub login: Option<Named>,
}

/// The ways a directory names the login store Claude Code reads from it: its path, which
/// is how a session's directory has always been compared with a store's, and a form of
/// it as Claude Code reaches that store on this platform (`form`). Two directories name
/// one store when either agrees, so a match is never narrower than a path comparison: a
/// wrong match only refuses a move, where a missed one could empty a store under a
/// running session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Named {
  path: PathBuf,
  form: Option<String>,
}

impl Named {
  /// How `dir` names its store, with `cwd` giving the working directory a relative path
  /// is read from, asked for only when one is needed.
  pub(crate) fn of(dir: &Path, cwd: impl FnOnce() -> Option<String>) -> Self {
    Self {
      path: dir.to_owned(),
      form: form(dir, cwd),
    }
  }

  /// How a store's own directory names it: ccusage gives that one as an absolute path.
  pub(crate) fn store(dir: &Path) -> Self {
    Self::of(dir, || None)
  }

  /// Whether `self` and `other` name one store. Pure.
  pub(crate) fn meets(&self, other: &Self) -> bool {
    self.path == other.path || (self.form.is_some() && self.form == other.form)
  }
}

/// The form `dir` names its store by on macOS, besides its path (`Named`): the Keychain
/// item it names, which Claude Code hashes from the variable's value as written, in NFC,
/// resolving nothing. So another Unicode spelling of a path names its item, while a
/// symlink or a relative path names another.
#[cfg(target_os = "macos")]
fn form(dir: &Path, _cwd: impl FnOnce() -> Option<String>) -> Option<String> {
  Some(crate::keychain::service(dir))
}

/// The form `dir` names its store by off macOS, besides its path (`Named`): the
/// directory the filesystem reaches from it, where Claude Code reads the credentials
/// file. So a symlink, or a relative path from the session's working directory, reaches
/// the same file.
#[cfg(not(target_os = "macos"))]
fn form(dir: &Path, cwd: impl FnOnce() -> Option<String>) -> Option<String> {
  reached(dir, cwd).map(|path| path.to_string_lossy().into_owned())
}

/// The directory the filesystem reaches from `dir`, through any symlink, and from `cwd`
/// when `dir` is relative; None when it cannot be resolved, as for a directory that does
/// not exist, or a relative one whose working directory is not known.
#[cfg(any(test, not(target_os = "macos")))]
fn reached(dir: &Path, cwd: impl FnOnce() -> Option<String>) -> Option<PathBuf> {
  let dir = if dir.is_relative() {
    Path::new(&cwd()?).join(dir)
  } else {
    dir.to_owned()
  };
  dir.canonicalize().ok()
}

/// `session` with the stores its directories name (`Stores`), its working directory read
/// for a relative one.
fn named(mut session: Session) -> Session {
  let pid = session.pid;
  let name = |dir: &Option<String>| {
    dir
      .as_deref()
      .map(|dir| Named::of(Path::new(dir), || cwd(pid)))
  };
  session.stores = Stores {
    config: name(&session.config_dir),
    login: name(&session.login_dir),
  };
  session
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
      .map(named)
      .collect(),
  )
}

/// One line of `ps -E -o pid=,lstart=,args=` as a Claude session, if it is one: its pid,
/// its start time (lstart's five words) and the two variables, from the words after its
/// program. ps prints the environment after the arguments, so each variable is its last
/// `NAME=value`: the environment's, ahead of an argument that looks like one. A value
/// with a space in it is cut at the space. The stores they name are left to `named`.
/// Pure.
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
    stores: Stores::default(),
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
    sessions.push(named(Session {
      pid,
      started: started_linux(pid),
      config_dir: variable(environment(), CONFIG_DIR),
      login_dir: variable(environment(), LOGIN_DIR),
      stores: Stores::default(),
    }));
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

/// What holds a session to a login other than the shared config's: the variable, and the
/// value the session was started with.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Drift<'a> {
  pub variable: &'static str,
  pub value: &'a str,
}

impl std::fmt::Display for Drift<'_> {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{}={}", self.variable, self.value)
  }
}

/// What makes `session` read a login other than the shared config's: a login directory,
/// or a config directory, that names a store other than `shared` names. None when it
/// follows a switch. Pure.
pub(crate) fn drift<'a>(session: &'a Session, shared: &Named) -> Option<Drift<'a>> {
  let other = |named: &Option<Named>| named.as_ref().is_some_and(|named| !named.meets(shared));
  let held = |variable: &'static str, value: &'a Option<String>| {
    value.as_deref().map(|value| Drift { variable, value })
  };
  if other(&session.stores.login) {
    held(LOGIN_DIR, &session.login_dir)
  } else if other(&session.stores.config) {
    held(CONFIG_DIR, &session.config_dir)
  } else {
    None
  }
}

/// The store `session` reads its login from: its login directory's, else its config
/// directory's; None when it names neither. Pure.
fn login_store(session: &Session) -> Option<&Named> {
  session
    .stores
    .login
    .as_ref()
    .or(session.stores.config.as_ref())
}

/// The sessions that read their login from `store`, so are pinned to it: a move that
/// takes the login out of that store leaves them none. Pure.
pub(crate) fn pinned_to<'a>(sessions: &'a [Session], store: &Named) -> Vec<&'a Session> {
  sessions
    .iter()
    .filter(|session| login_store(session).is_some_and(|named| named.meets(store)))
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
      "the running Claude sessions could not be listed, so first restart, from a shell without {LOGIN_DIR} (unset {LOGIN_DIR}), any Claude session started with it naming {name}'s own store, and only then run ccusage login {name}"
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
/// shared config's (`drift`), so do not follow a switch: how many, and that doctor names
/// them, in the words doctor and the remedies use. None when every one follows. Pure.
pub(crate) fn unfollowing(sessions: &[Session], shared: &Named) -> Option<String> {
  match sessions
    .iter()
    .filter(|session| drift(session, shared).is_some())
    .count()
  {
    0 => None,
    1 => Some(
      "1 running Claude session reads a login other than the shared config's, so it does not follow a switch; ccusage doctor names it"
        .to_owned(),
    ),
    count => Some(format!(
      "{count} running Claude sessions read a login other than the shared config's, so they do not follow a switch; ccusage doctor names them"
    )),
  }
}

/// `unfollowing` for the sessions running now; None as well when they cannot be listed,
/// which doctor reports.
pub(crate) fn unfollowing_now(shared: &Path) -> Option<String> {
  unfollowing(&running()?, &Named::store(shared))
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
    let shared = Named::store(Path::new("/home/u/.config/claude"));
    let line = " 102 Fri Nov 13 08:00:01 2026 /opt/claude --resume CLAUDE_CONFIG_DIR=/home/u/.config/claude/ ANTHROPIC_API_KEY=SECRET CLAUDE_SECURESTORAGE_CONFIG_DIR=/p/work";
    let session = parse(line).ok_or("a claude line is a session")?;
    assert_eq!(
      session,
      Session {
        pid: 102,
        started: Some("Fri Nov 13 08:00:01 2026".into()),
        config_dir: Some("/home/u/.config/claude/".into()),
        login_dir: Some("/p/work".into()),
        stores: Stores::default(),
      }
    );
    let drifted = |line: &str| -> crate::Result<Option<String>> {
      let session = named(parse(line).ok_or("a claude line is a session")?);
      Ok(drift(&session, &shared).map(|drift| drift.to_string()))
    };
    assert_eq!(
      drifted(line)?.as_deref(),
      Some("CLAUDE_SECURESTORAGE_CONFIG_DIR=/p/work")
    );
    assert_eq!(
      drifted(" 101 Fri Nov 13 08:00:00 2026 claude CLAUDE_CONFIG_DIR=/home/u/.config/claude")?,
      None
    );
    assert_eq!(
      drifted(" 103 Fri Nov 13 08:00:02 2026 claude CLAUDE_CONFIG_DIR=/elsewhere")?.as_deref(),
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
    let sessions: Vec<Session> = lines
      .iter()
      .filter_map(|line| parse(line))
      .map(named)
      .collect();
    assert_eq!(sessions.len(), lines.len());
    let pinned = pinned_to(&sessions, &Named::store(Path::new(own)));
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
    // ISSUE:0010 item 8(c): with the sessions unlisted, the shell to restart from comes
    // first, so "without" cannot attach to "started", and says what to unset.
    assert_eq!(
      remedy("work", None),
      "the running Claude sessions could not be listed, so first restart, from a shell without CLAUDE_SECURESTORAGE_CONFIG_DIR (unset CLAUDE_SECURESTORAGE_CONFIG_DIR), any Claude session started with it naming work's own store, and only then run ccusage login work"
    );
    let refused = refusal("work", &pinned, "switching to work");
    assert!(
      refused.starts_with("Claude sessions 201 and 202 read work's own store, and switching to work would leave it without a login under them; nothing was moved"),
      "{refused}"
    );
    Ok(())
  }

  // ISSUE:0010 item 2: a session's directory names a profile's store as Claude Code
  // reaches that store. By its path everywhere, as before; on macOS also by the Keychain
  // item, which another Unicode spelling of the path names too, while a symlink or a
  // relative path names another; elsewhere also through the filesystem, by a symlink or
  // from the session's working directory, to the credentials file.
  #[test]
  fn a_directory_names_a_store_as_claude_code_reaches_it() -> crate::Result<()> {
    let scratch = crate::scratch::Scratch::new()?;
    let real = scratch.path().join("profiles").join("work");
    std::fs::create_dir_all(&real)?;
    let link = scratch.path().join("link");
    std::os::unix::fs::symlink(&real, &link)?;
    let store = Named::store(&real);
    let macos = cfg!(target_os = "macos");
    let from_scratch = || Some(scratch.path().display().to_string());
    // The same path with a trailing slash, as before, everywhere.
    let slashed = PathBuf::from(format!("{}/", real.display()));
    assert!(Named::store(&slashed).meets(&store));
    // A symlink, and a path relative to the session's working directory.
    assert_eq!(Named::store(&link).meets(&store), !macos);
    let relative = Named::of(Path::new("profiles/work"), from_scratch);
    assert_eq!(relative.meets(&store), !macos);
    // Another Unicode spelling of the same path.
    let (composed, decomposed) = ("/data/caf\u{e9}", "/data/cafe\u{301}");
    assert_eq!(
      Named::store(Path::new(decomposed)).meets(&Named::store(Path::new(composed))),
      macos
    );
    // What the filesystem reaches, which the rule off macOS rests on, on every platform.
    let canonical = real.canonicalize()?;
    assert_eq!(reached(&link, || None), Some(canonical.clone()));
    assert_eq!(
      reached(Path::new("profiles/work"), from_scratch),
      Some(canonical)
    );
    assert_eq!(reached(Path::new("profiles/work"), || None), None);
    assert_eq!(reached(&scratch.path().join("absent"), || None), None);
    Ok(())
  }

  // ISSUE:0010 item 7: which, predict and the TUI count the running sessions that do not
  // follow a switch in the words doctor and the remedies use, and say nothing when every
  // one follows.
  #[test]
  fn the_count_of_sessions_that_do_not_follow_a_switch_is_worded_as_doctor_words_them()
  -> crate::Result<()> {
    let shared_dir = "/home/u/.config/claude";
    let shared = Named::store(Path::new(shared_dir));
    // The first session follows the shared config, and each after it reads another login.
    let dirs = [shared_dir, "/data/profiles/a", "/data/profiles/b"];
    let sessions = dirs
      .iter()
      .zip(601..)
      .map(|(dir, pid)| {
        parse(&format!(
          " {pid} Mon Nov 16 16:30:00 2026 claude {CONFIG_DIR}={dir}"
        ))
        .map(named)
        .ok_or("a claude line is a session")
      })
      .collect::<std::result::Result<Vec<_>, _>>()?;
    // How many of the first `n` sessions read a login other than the shared config's.
    let drifting = |n: usize| dirs[..n].iter().filter(|dir| **dir != shared_dir).count();
    assert_eq!(unfollowing(&sessions[..1], &shared), None);
    assert_eq!(
      unfollowing(&sessions[..2], &shared),
      Some(format!(
        "{} running Claude session reads a login other than the shared config's, so it does not follow a switch; ccusage doctor names it",
        drifting(2)
      ))
    );
    assert_eq!(
      unfollowing(&sessions, &shared),
      Some(format!(
        "{} running Claude sessions read a login other than the shared config's, so they do not follow a switch; ccusage doctor names them",
        drifting(sessions.len())
      ))
    );
    Ok(())
  }
}
