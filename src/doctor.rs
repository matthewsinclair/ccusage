use crate::{
  Result, clean,
  config::{self, Store},
  storage,
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
