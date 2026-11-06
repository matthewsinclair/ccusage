use crate::{
  Result, clean,
  config::{self, Store},
  shared, storage,
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

/// Shared mode's checks. Each finding is a warning: sessions still start while one
/// stands, though they may show or use an account other than the active profile's.
fn shared_checks(store: &Store, shared: &Path) {
  line(
    "OK",
    format!("Shared mode: sessions use {}", shared.display()),
  );
  if holds_own_login(shared) {
    line(
      "WARN",
      format!(
        "{} holds a login of its own; retire it once every session uses a profile's",
        shared.display()
      ),
    );
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
  let names_a_profile = |value: &std::ffi::OsStr| dirs.iter().any(|dir| dir.as_os_str() == value);
  if let Some(value) = std::env::var_os(shared::LOGIN_DIR)
    && !names_a_profile(&value)
  {
    line(
      "WARN",
      format!(
        "{}={} names no profile's directory",
        shared::LOGIN_DIR,
        Path::new(&value).display()
      ),
    );
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
  {
    let active_dir = active
      .as_deref()
      .and_then(|name| store.profile_dir(name).ok());
    match shared::launchd_dir() {
      Ok(value) if value.is_empty() => line(
        "WARN",
        format!(
          "launchd has no {}, so Claude Code started from a GUI app uses the shared config's own login; run ccusage use NAME",
          shared::LOGIN_DIR
        ),
      ),
      Ok(value) if active_dir.is_some_and(|dir| dir.as_os_str() == value.as_str()) => line(
        "OK",
        "launchd gives GUI-started Claude Code the active profile's login",
      ),
      Ok(value) if names_a_profile(std::ffi::OsStr::new(&value)) => line(
        "WARN",
        "launchd gives GUI-started Claude Code another profile's login than the active one; run ccusage use NAME",
      ),
      Ok(value) => line(
        "WARN",
        format!(
          "launchd's {}={value} names no profile's directory",
          shared::LOGIN_DIR
        ),
      ),
      Err(error) => line("WARN", error.to_string()),
    }
  }
}

/// Whether the shared config directory holds a login of its own: its credentials
/// file, or on macOS its Keychain item, looked up by name alone.
fn holds_own_login(shared: &Path) -> bool {
  if shared.join(".credentials.json").exists() {
    return true;
  }
  #[cfg(target_os = "macos")]
  {
    let (service, account) = (crate::keychain::service(shared), crate::keychain::account());
    let item = crate::keychain::Item {
      service: &service,
      account: &account,
      keychain: None,
    };
    if item.exists().unwrap_or(false) {
      return true;
    }
  }
  false
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
          Ok(Some(shared)) => shared_checks(&store, &shared),
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
    "Authentication and usage were not queried; this command makes no login changes",
  );
  if failed {
    Err("environment checks failed; see FAIL lines above".into())
  } else {
    Ok(())
  }
}
