//! The one way a profile becomes the active account. The CLI's `use`, `run NAME` in
//! shared mode, the TUI's `a` key and `which --use` all call `to`, so a switch means
//! the same thing wherever it starts.

use crate::{Result, auth, config::Store, shared};

/// What a switch did, which decides what the verb that asked for it says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Switched {
  /// Shared mode: the shared config records the account, and new sessions use it.
  Shared,
  /// Shared mode off: future `ccusage run` launches use the account.
  Selected,
}

impl Switched {
  /// The one line a verb shows for the switch. Pure.
  pub fn message(self, name: &str) -> String {
    match self {
      Self::Shared => format!(
        "Active: {name}. New Claude sessions use this account; running ones switch when restarted."
      ),
      Self::Selected => format!("Active: {name}. Future ccusage run launches use this account."),
    }
  }
}

/// Make `name` the active account. A profile with no stored login is refused before
/// anything changes. In shared mode the shared config records its account first and,
/// on macOS, launchd points at its login after; either way it becomes the selected
/// profile. Running sessions keep their login until they are restarted.
pub fn to(store: &Store, name: &str) -> Result<Switched> {
  auth::read(&store.profile_dir(name)?)?;
  let Some(shared) = store.shared_dir()? else {
    store.activate(name)?;
    return Ok(Switched::Selected);
  };
  shared::record(store, &shared, name)?;
  store.activate(name)?;
  #[cfg(target_os = "macos")]
  shared::set_launchd(&store.profile_dir(name)?)?;
  Ok(Switched::Shared)
}
