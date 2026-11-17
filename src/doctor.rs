use crate::{
  Result, clean,
  config::{self, Store},
  sessions::{Drift, Named, Session},
  shared, storage,
  usage::Whose,
};
use std::{fs, path::Path, process::Command};

fn line(status: &str, message: impl AsRef<str>) {
  println!("  {status:<4} {}", clean(message.as_ref()));
}

fn file_check(path: &Path, failed: &mut bool) {
  match storage::check_private_file(path) {
    Ok(true) => line("OK", format!("Private file: {}", path.display())),
    Ok(false) => {}
    Err(error) => {
      line("FAIL", error.to_string());
      *failed = true;
    }
  }
}

fn directory_check(path: &Path, failed: &mut bool) {
  match fs::metadata(path) {
    Ok(metadata) if metadata.is_dir() => {
      #[cfg(unix)]
      {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode() & 0o7777;
        if mode != 0o700 {
          line(
            "WARN",
            format!(
              "Directory {} has mode {mode:04o}; recommend chmod 700",
              path.display()
            ),
          );
          return;
        }
      }
      line("OK", format!("Private directory: {}", path.display()));
    }
    Ok(_) => {
      line("FAIL", format!("Expected a directory: {}", path.display()));
      *failed = true;
    }
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
      line("INFO", format!("Not created yet: {}", path.display()));
    }
    Err(error) => {
      line(
        "FAIL",
        format!("Cannot inspect {}: {error}", path.display()),
      );
      *failed = true;
    }
  }
}

/// What a credential store holds, for doctor's report. Reading it decrypts a Keychain
/// item, as every usage fetch does; nothing of it is shown.
#[derive(Debug, PartialEq)]
enum Holds {
  Login,
  Nothing,
  NoStore,
  Unreadable(String),
}

/// What the store Claude Code reads `dir`'s login from holds.
fn holds(dir: &Path) -> Holds {
  match crate::auth::load(dir) {
    Ok(Some((_, bytes))) => match crate::seat::holds_login(&bytes) {
      Ok(true) => Holds::Login,
      Ok(false) => Holds::Nothing,
      Err(error) => Holds::Unreadable(error.to_string()),
    },
    Ok(None) => Holds::NoStore,
    Err(error) => Holds::Unreadable(error.to_string()),
  }
}

/// One profile's own store, as doctor found it: what it holds; whose login that is, for
/// a profile that is not seated and holds one; and the running sessions pinned to it,
/// None when they could not be listed.
struct Own<'a> {
  name: String,
  holds: Holds,
  whose: Option<Whose>,
  pinned: Option<Vec<&'a Session>>,
}

/// Where each profile's login is, from what each store holds, whose its own login is
/// and which profile is seated: one finding for each profile, failing when it has no
/// login or two, or its own store holds another account's, with a remedy that restarts
/// the sessions pinned to that store before the login that ends it; and one for a shared
/// login that belongs to no profile. Pure.
fn where_logins_are(
  shared: &Holds,
  seated: Option<&str>,
  profiles: &[Own],
) -> Vec<(&'static str, String)> {
  let mut findings = Vec::new();
  match (shared, seated) {
    (Holds::Unreadable(error), _) => findings.push((
      "FAIL",
      format!("The shared config's store could not be read: {error}"),
    )),
    (Holds::Login, None) => findings.push((
      "WARN",
      "The shared config holds a login of its own, which belongs to no profile; the first ccusage use NAME replaces it once its account has another login in a profile".to_owned(),
    )),
    _ => {}
  }
  for own in profiles {
    let name = own.name.as_str();
    let remedy = || crate::sessions::remedy(name, own.pinned.as_deref());
    let active = seated == Some(name);
    let finding = match (active, shared, &own.holds) {
      (_, _, Holds::Unreadable(error)) => (
        "FAIL",
        format!("{name}'s own store could not be read: {error}"),
      ),
      (true, Holds::Login, Holds::Login) => (
        "FAIL",
        format!(
          "{name} has two logins, the shared config's and another in its own store, so a switch away from {name} is refused; {}, which leaves {name} one login",
          remedy()
        ),
      ),
      (true, Holds::Login, _) => (
        "OK",
        format!("{name}'s login is in the shared config, which every session reads"),
      ),
      // The shared store's own finding says why it cannot be told.
      (true, Holds::Unreadable(_), _) => continue,
      (true, _, _) => (
        "FAIL",
        format!(
          "{name} is active, but the shared config holds no login; {}",
          remedy()
        ),
      ),
      (false, _, Holds::Login) => match &own.whose {
        Some(Whose::Another) => (
          "FAIL",
          format!(
            "{name}'s own store holds another account's login, so {name}'s usage is refused and a switch to {name} would seat that login; {}",
            remedy()
          ),
        ),
        Some(Whose::Unchecked(why)) => {
          findings.push(("OK", format!("{name}'s login is in its own store")));
          (
            "INFO",
            format!("{name}'s own login was not checked against {name}'s account: {why}"),
          )
        }
        Some(Whose::Profile) | None => ("OK", format!("{name}'s login is in its own store")),
      },
      (false, _, _) => (
        "FAIL",
        format!("{name} has no login; run ccusage login {name}"),
      ),
    };
    findings.push(finding);
  }
  findings
}

/// The shared store's document against what one `security -i` line can carry, for a
/// Keychain item only, as two lengths and nothing else of it. None for a file store.
#[cfg(target_os = "macos")]
fn line_room(shared: &Path) -> Option<(&'static str, String)> {
  let Ok(Some((
    crate::auth::Location::Keychain {
      service,
      account,
      keychain,
    },
    bytes,
  ))) = crate::auth::load(shared)
  else {
    return None;
  };
  let item = crate::keychain::Item {
    service: &service,
    account: &account,
    keychain: keychain.as_deref(),
  };
  let length = bytes.len();
  Some(match item.capacity() {
    Ok(capacity) if length <= capacity => (
      "OK",
      format!(
        "The shared login's document is {length} bytes; one security -i line carries {capacity}"
      ),
    ),
    Ok(capacity) => (
      "FAIL",
      format!(
        "The shared login's document is {length} bytes, more than the {capacity} one security -i line carries, so a switch would be refused"
      ),
    ),
    Err(error) => (
      "WARN",
      format!("The shared login's room on a security -i line was not checked: {error}"),
    ),
  })
}

/// Claude Code's two flags a switch depends on, from a `.claude.json`'s flag cache, as
/// Claude Code reads it (cachedGrowthBookFeatures, else cachedFeatureFlags): the
/// session-owner prompt and the sibling-login watch. Each is None when the cache does
/// not hold it, so it stands at its default: the prompt off and the watch on. Pure.
fn switch_flags(text: &str) -> (Option<bool>, Option<bool>) {
  let Ok(document) = serde_json::from_str::<serde_json::Value>(text) else {
    return (None, None);
  };
  let cache = ["cachedGrowthBookFeatures", "cachedFeatureFlags"]
    .into_iter()
    .find_map(|key| document.get(key).filter(|cache| !cache.is_null()));
  let flag = |name: &str| {
    cache
      .and_then(|cache| cache.get(name))
      .and_then(serde_json::Value::as_bool)
  };
  (
    flag("tengu_bubbly_fountain"),
    flag("tengu_streamed_thimble"),
  )
}

/// Shared mode's checks. A login that is missing or doubled, or a switch left
/// unfinished, fails; the rest warn, since sessions still start while one stands,
/// though they may show or use an account other than the active profile's.
fn shared_checks(store: &Store, shared: &Path, failed: &mut bool) {
  line(
    "OK",
    format!("Shared mode: sessions use {}", shared.display()),
  );
  let mut report = |status: &'static str, message: String| {
    *failed |= status == "FAIL";
    line(status, message);
  };
  match crate::seat::pending(store) {
    Ok(Some(journal)) => report("FAIL", crate::seat::unfinished(&journal.request)),
    Ok(None) => {}
    Err(error) => report("FAIL", error.to_string()),
  }
  // The running sessions, each read from its own process's environment, once: None when
  // this system's process listing cannot be read.
  let sessions = crate::sessions::running();
  match crate::seat::seated(store) {
    Ok(seated) => {
      let profiles: Vec<Own> = store
        .config
        .profiles
        .keys()
        .map(|name| {
          let dir = store.profile_dir(name);
          let holds = dir.as_ref().map_or_else(
            |error| Holds::Unreadable(error.to_string()),
            |dir| holds(dir),
          );
          // Whose login a profile that is not seated holds, checked as the shared
          // login is, so a /login of another account in a session pinned there shows.
          let whose = match (&dir, &holds) {
            (Ok(dir), Holds::Login) if seated.as_deref() != Some(name.as_str()) => {
              Some(crate::usage::whose(store, name, dir))
            }
            _ => None,
          };
          let pinned = dir
            .ok()
            .zip(sessions.as_deref())
            .map(|(dir, sessions)| crate::sessions::pinned_to(sessions, &Named::store(&dir)));
          Own {
            name: name.clone(),
            holds,
            whose,
            pinned,
          }
        })
        .collect();
      let shared_holds = holds(shared);
      for (status, message) in where_logins_are(&shared_holds, seated.as_deref(), &profiles) {
        report(status, message);
      }
      // Whose the shared login is: checked when the endpoint can be reached, and
      // never reported as passed when it was not.
      if let Some(name) = seated.as_deref()
        && shared_holds == Holds::Login
      {
        match crate::usage::whose(store, name, shared) {
          Whose::Profile => report("OK", format!("The shared login is verified as {name}'s")),
          Whose::Another => {
            let pinned = profiles
              .iter()
              .find(|own| own.name == name)
              .and_then(|own| own.pinned.as_deref());
            report(
              "FAIL",
              format!(
                "The shared login is not {name}'s: another account's login is in the shared config; {}",
                crate::sessions::remedy(name, pinned)
              ),
            )
          }
          Whose::Unchecked(why) => report(
            "INFO",
            format!("The shared login was not checked against {name}'s account: {why}"),
          ),
        }
      }
    }
    Err(error) => report(
      "FAIL",
      format!("The record of whose login the shared config holds could not be read: {error}"),
    ),
  }
  if let Ok(Some(text)) = shared::settings(shared) {
    let (owner_prompt, sibling_watch) = switch_flags(&text);
    if owner_prompt == Some(true) {
      report(
        "WARN",
        "Claude Code's session-owner prompt (tengu_bubbly_fountain) is on, so after a switch a running session refuses its next request and asks whether to exit, log in again or copy the conversation".to_owned(),
      );
    }
    if sibling_watch == Some(false) {
      report(
        "WARN",
        "Claude Code's sibling-login watch (tengu_streamed_thimble) is off, so a session showing its login notice does not take up the login a switch seats until it restarts".to_owned(),
      );
    }
  }
  #[cfg(target_os = "macos")]
  if let Some((status, message)) = line_room(shared) {
    report(status, message);
  }
  // Sessions that will not follow a switch.
  match &sessions {
    None => report(
      "INFO",
      format!(
        "Running Claude sessions were not checked: {}",
        crate::sessions::UNLISTED
      ),
    ),
    Some(sessions) => {
      let shared = Named::store(shared);
      for session in sessions {
        if let Some(drift) = crate::sessions::drift(session, &shared) {
          let cwd = crate::sessions::cwd(session.pid);
          report("WARN", unfollowed(session, &drift, cwd.as_deref()));
        }
      }
    }
  }
  let active = match store.active() {
    Ok(Some(name)) if store.config.profiles.contains_key(&name) => Some(name),
    _ => None,
  };
  if let Some(name) = &active {
    let recorded = shared::settings(shared).and_then(|text| {
      text
        .as_deref()
        .map(shared::account)
        .transpose()
        .map(Option::flatten)
    });
    match (recorded, shared::profile_account(store, name)) {
      (Ok(Some(recorded)), Ok(Some(own))) if shared::same_account(&recorded, &own) => line(
        "OK",
        format!("The shared .claude.json records {name}'s account"),
      ),
      (Ok(_), Ok(_)) => line(
        "WARN",
        format!("The shared .claude.json does not record {name}'s account; run ccusage use {name}"),
      ),
      (Err(error), _) | (_, Err(error)) => line("WARN", format!("Account records: {error}")),
    }
  }
  let dirs: Vec<_> = store
    .config
    .profiles
    .keys()
    .filter_map(|name| store.profile_dir(name).ok())
    .collect();
  if let Some(warning) = shared::shell_pin(std::env::var_os(shared::LOGIN_DIR).as_deref(), shared) {
    line("WARN", warning);
  }
  if let Ok(entries) = fs::read_dir(store.data.join("profiles")) {
    for path in entries.flatten().map(|entry| entry.path()) {
      let known = path.canonicalize().is_ok_and(|real| dirs.contains(&real));
      if path.is_dir() && !known {
        line(
          "WARN",
          format!("{} has no profile in the config", path.display()),
        );
      }
    }
  }
  #[cfg(target_os = "macos")]
  match shared::launchd_dir() {
    Ok(value) if value.is_empty() => line(
      "OK",
      "launchd gives GUI-started Claude Code the shared config's login",
    ),
    Ok(value) => line(
      "WARN",
      format!(
        "launchd's {}={value} makes GUI-started Claude Code read that directory's login instead of the shared config's; run ccusage use NAME to clear it",
        shared::LOGIN_DIR
      ),
    ),
    Err(error) => line("WARN", error.to_string()),
  }
}

pub fn run(path: &Path) -> Result<()> {
  println!("Ccusage {} · doctor", env!("CARGO_PKG_VERSION"));
  line(
    "OK",
    format!(
      "Platform: {} / {}",
      std::env::consts::OS,
      std::env::consts::ARCH
    ),
  );
  let mut failed = false;
  match Command::new("claude").arg("--version").output() {
    Ok(output) if output.status.success() => {
      line(
        "OK",
        format!(
          "Claude on PATH: {}",
          String::from_utf8_lossy(&output.stdout).trim()
        ),
      );
    }
    Ok(output) => {
      line(
        "FAIL",
        format!(
          "claude --version exited with {}; check your Claude installation",
          output.status
        ),
      );
      failed = true;
    }
    Err(error) => {
      line(
        "FAIL",
        format!("Cannot run claude from PATH: {error}; install Claude Code and add it to PATH"),
      );
      failed = true;
    }
  }
  match storage::check_private_file(path) {
    Ok(false) => {
      line(
        "INFO",
        "Config not initialized; run ccusage init, then ccusage add NAME and ccusage login NAME",
      );
    }
    Ok(true) => match Store::load(path.to_path_buf()) {
      Ok(store) => {
        line(
          "OK",
          format!(
            "Config: {} ({} profiles)",
            store.path.display(),
            store.config.profiles.len()
          ),
        );
        for root in [&store.data, &store.state, &store.cache] {
          directory_check(root, &mut failed);
        }
        file_check(&store.state.join("active.json"), &mut failed);
        file_check(&store.state.join(crate::history::FILE), &mut failed);
        for name in store.config.profiles.keys() {
          match store.profile_dir(name) {
            Ok(dir) => {
              directory_check(&dir, &mut failed);
              file_check(&dir.join(".credentials.json"), &mut failed);
              file_check(&store.cache.join(format!("{name}.json")), &mut failed);
            }
            Err(error) => {
              line("FAIL", format!("Profile {name}: {error}"));
              failed = true;
            }
          }
        }
        if store.config.profiles.is_empty() {
          line(
            "INFO",
            "No profiles; run ccusage add NAME, then ccusage login NAME",
          );
        }
        match store.shared_dir() {
          Ok(Some(shared)) => shared_checks(&store, &shared, &mut failed),
          Ok(None) => {}
          Err(error) => {
            line("FAIL", format!("Shared Claude config: {error}"));
            failed = true;
          }
        }
        match store.active() {
          Ok(Some(name)) if store.config.profiles.contains_key(&name) => {
            line("OK", format!("Active profile: {name}"))
          }
          Ok(Some(_)) => line(
            "WARN",
            "Active profile is no longer configured; run ccusage use NAME",
          ),
          Ok(None) => line(
            "INFO",
            "No active profile; after login, run ccusage use NAME",
          ),
          Err(error) => {
            line("FAIL", format!("Active selection: {error}"));
            failed = true;
          }
        }
      }
      Err(error) => {
        line("FAIL", error.to_string());
        failed = true;
      }
    },
    Err(error) => {
      line("FAIL", error.to_string());
      failed = true;
    }
  }
  // Resolve XDG paths even before initialization to catch invalid environments.
  for (variable, fallback) in [
    ("XDG_DATA_HOME", ".local/share"),
    ("XDG_STATE_HOME", ".local/state"),
    ("XDG_CACHE_HOME", ".cache"),
  ] {
    if let Err(error) = config::xdg_path(variable, fallback) {
      line("FAIL", format!("{variable}: {error}"));
      failed = true;
    }
  }
  line(
    "INFO",
    "Usage was not queried, and this command makes no login changes; in shared mode it asks Claude's profile endpoint whose the shared login is, unless a usage fetch already verified it",
  );
  if failed {
    Err("environment checks failed; see FAIL lines above".into())
  } else {
    Ok(())
  }
}

/// Doctor's warning for `session`, which `drift` holds to a login other than the shared
/// config's, so it does not follow a switch: which session, since when and in what
/// directory (`cwd`), what holds it, and the restart that ends it, from a shell without
/// that variable, named as the remedies name it. Pure.
fn unfollowed(session: &Session, drift: &Drift, cwd: Option<&str>) -> String {
  format!(
    "Claude session {} (started {}, in {}) runs with {drift}, so it does not follow a switch; restart it from a shell without {variable} (unset {variable})",
    session.pid,
    session.started.as_deref().unwrap_or("at an unknown time"),
    cwd.unwrap_or("an unknown directory"),
    variable = drift.variable
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  // ST0003 AT-04.1: doctor says where each profile's login is, and fails a profile with
  // no login or two; a shared login that belongs to no profile only warns.
  #[test]
  fn each_login_is_found_once_and_a_missing_or_doubled_one_fails() {
    let profiles = |a: Holds, b: Holds| vec![own("a", a, None, &[]), own("b", b, None, &[])];
    let statuses = |findings: Vec<(&'static str, String)>| -> Vec<&'static str> {
      findings.into_iter().map(|(status, _)| status).collect()
    };
    // a seated, b parked: each login found once.
    let seated = where_logins_are(
      &Holds::Login,
      Some("a"),
      &profiles(Holds::Nothing, Holds::Login),
    );
    assert_eq!(statuses(seated), ["OK", "OK"]);
    // a seated, with a login in its own store as well.
    let doubled = where_logins_are(
      &Holds::Login,
      Some("a"),
      &profiles(Holds::Login, Holds::Login),
    );
    assert_eq!(statuses(doubled), ["FAIL", "OK"]);
    // a seated, but the shared store empty; b never logged in.
    let missing = where_logins_are(
      &Holds::Nothing,
      Some("a"),
      &profiles(Holds::Nothing, Holds::NoStore),
    );
    assert_eq!(statuses(missing), ["FAIL", "FAIL"]);
    // 0.1.8's layout: nothing seated, and the shared config keeps its own login.
    let adopting = where_logins_are(&Holds::Login, None, &profiles(Holds::Login, Holds::Login));
    assert_eq!(statuses(adopting), ["WARN", "OK", "OK"]);
  }

  // ST0003 AT-04.2: the two flags a switch depends on are read from Claude Code's flag
  // cache as Claude Code reads it, and stand at their defaults when it does not hold them.
  #[test]
  fn switch_flags_come_from_claude_codes_flag_cache() {
    assert_eq!(switch_flags("{}"), (None, None));
    assert_eq!(switch_flags("not json"), (None, None));
    let growthbook = r#"{"cachedGrowthBookFeatures":{"tengu_bubbly_fountain":true,"tengu_streamed_thimble":false},"cachedFeatureFlags":{"tengu_bubbly_fountain":false}}"#;
    assert_eq!(switch_flags(growthbook), (Some(true), Some(false)));
    let legacy =
      r#"{"cachedGrowthBookFeatures":null,"cachedFeatureFlags":{"tengu_streamed_thimble":true}}"#;
    assert_eq!(switch_flags(legacy), (None, Some(true)));
  }

  /// One profile's own store as doctor finds it, with `pinned` the sessions reading it.
  fn own<'a>(name: &str, holds: Holds, whose: Option<Whose>, pinned: &[&'a Session]) -> Own<'a> {
    Own {
      name: name.to_owned(),
      holds,
      whose,
      pinned: Some(pinned.to_vec()),
    }
  }

  // ISSUE:0008 Acceptance 1 and 2: a parked profile whose own store holds another
  // account's login fails, and one whose login could not be checked says so beside its
  // OK; each remedy restarts the sessions pinned to the store before the login that ends
  // the fault, as the seated profile's second login has it too.
  #[test]
  fn another_accounts_login_fails_and_each_remedy_restarts_the_pinned_sessions_first() {
    let pinned = Session {
      pid: 401,
      started: None,
      config_dir: None,
      login_dir: Some("/data/profiles/b".into()),
      stores: Default::default(),
    };
    let pinned = [&pinned];
    let findings = where_logins_are(
      &Holds::Login,
      Some("a"),
      &[
        own("a", Holds::Login, None, &pinned),
        own("b", Holds::Login, Some(Whose::Another), &pinned),
        own(
          "c",
          Holds::Login,
          Some(Whose::Unchecked("its token is expired".into())),
          &[],
        ),
        own("d", Holds::Login, Some(Whose::Profile), &[]),
      ],
    );
    let statuses: Vec<&str> = findings.iter().map(|(status, _)| *status).collect();
    assert_eq!(statuses, ["FAIL", "FAIL", "OK", "INFO", "OK"]);
    for (index, name, fault) in [
      (0, "a", "a has two logins"),
      (1, "b", "b's own store holds another account's login"),
    ] {
      let said = &findings[index].1;
      assert!(said.starts_with(fault), "{said}");
      let restart = said.find(&format!(
        "Claude session 401 reads {name}'s own store, so first restart it"
      ));
      let login = said.find(&format!("ccusage login {name}"));
      assert!(
        restart.is_some() && login.is_some() && restart < login,
        "{said}"
      );
    }
    assert_eq!(
      findings[3].1,
      "c's own login was not checked against c's account: its token is expired"
    );
    // ISSUE:0010 item 8(a): after a remedy that names a session, the finding says whose
    // single login is left, by name.
    assert!(
      findings[0]
        .1
        .ends_with("and only then run ccusage login a, which leaves a one login"),
      "{}",
      findings[0].1
    );
  }

  // ISSUE:0010 item 8(b): the warning for a session that does not follow a switch names
  // the variable to leave out of the shell it restarts from, as the remedies do.
  #[test]
  fn a_session_that_does_not_follow_a_switch_is_told_which_variable_to_unset() {
    let (variable, value) = ("CLAUDE_SECURESTORAGE_CONFIG_DIR", "/data/profiles/b");
    let session = Session {
      pid: 402,
      started: Some("Mon Nov 16 16:30:00 2026".into()),
      config_dir: None,
      login_dir: Some(value.into()),
      stores: Default::default(),
    };
    assert_eq!(
      unfollowed(&session, &Drift { variable, value }, Some("/work")),
      "Claude session 402 (started Mon Nov 16 16:30:00 2026, in /work) runs with CLAUDE_SECURESTORAGE_CONFIG_DIR=/data/profiles/b, so it does not follow a switch; restart it from a shell without CLAUDE_SECURESTORAGE_CONFIG_DIR (unset CLAUDE_SECURESTORAGE_CONFIG_DIR)"
    );
  }
}
