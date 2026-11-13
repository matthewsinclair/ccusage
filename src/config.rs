use crate::{Result, storage};
use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  env,
  path::{Path, PathBuf},
};

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
  /// Set, it turns on shared mode: sessions keep this Claude config directory and
  /// borrow the active profile's login. Unset, ccusage behaves as it always has.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub claude_config_dir: Option<PathBuf>,
  /// The which rule's settings. A config that sets none is written back without the
  /// table, because every ccusage older than the rule refuses a config that has it.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub which: Option<Which>,
  #[serde(default)]
  pub profiles: BTreeMap<String, Profile>,
}

/// `[which]` in config.toml.
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Which {
  /// The percent of an account's weekly allowance at which the rule moves on to the
  /// next account; absent, the rule's one default.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub cap: Option<f64>,
}

impl Config {
  /// The cap in force: the config's own, else `which::DEFAULT_CAP`.
  pub fn cap(&self) -> f64 {
    self
      .which
      .as_ref()
      .and_then(|which| which.cap)
      .unwrap_or(crate::which::DEFAULT_CAP)
  }

  /// Refuses a setting no rule could use. A cap is a percent of the weekly allowance:
  /// above 0, so the rule can choose at all, and at most 100.
  fn check(&self) -> Result<()> {
    let cap = self.cap();
    if !cap.is_finite() || cap <= 0.0 || cap > 100.0 {
      return Err(format!("invalid [which] cap {cap}; set it above 0 and at most 100").into());
    }
    Ok(())
  }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
  pub email: Option<String>,
  pub config_dir: Option<PathBuf>,
}

pub struct Store {
  pub path: PathBuf,
  pub root: PathBuf,
  pub data: PathBuf,
  pub state: PathBuf,
  pub cache: PathBuf,
  pub config: Config,
}

/// No config file exists yet; `ccusage init` creates one.
#[derive(Debug)]
pub struct NoConfig(PathBuf);

impl std::fmt::Display for NoConfig {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      formatter,
      "no config at {}; run ccusage init",
      self.0.display()
    )
  }
}

impl std::error::Error for NoConfig {}

pub fn valid_name(name: &str) -> bool {
  !name.is_empty()
    && name.len() <= 64
    && name
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn home() -> Result<PathBuf> {
  env::var_os("HOME")
    .or_else(|| env::var_os("USERPROFILE"))
    .map(PathBuf::from)
    .ok_or_else(|| "HOME or USERPROFILE must be set".into())
}

pub fn default_path() -> Result<PathBuf> {
  if let Some(path) = env::var_os("CCUSAGE_CONFIG") {
    return Ok(path.into());
  }
  Ok(xdg_path("XDG_CONFIG_HOME", ".config")?.join("config.toml"))
}

pub fn absolute(path: &Path) -> Result<PathBuf> {
  let expanded = if path == Path::new("~") {
    home()?
  } else if let Ok(rest) = path.strip_prefix("~/") {
    home()?.join(rest)
  } else {
    path.to_path_buf()
  };
  let full = if expanded.is_absolute() {
    expanded
  } else {
    env::current_dir()?.join(expanded)
  };
  // Lexical normalization works before the directory exists and keeps hashing stable.
  let mut result = PathBuf::new();
  for part in full.components() {
    match part {
      std::path::Component::CurDir => {}
      std::path::Component::ParentDir => {
        result.pop();
      }
      other => result.push(other.as_os_str()),
    }
  }
  // Resolve the existing ancestor before appending a new profile directory.
  // This keeps macOS /var -> /private/var aliases stable before and after login.
  let mut ancestor = result.as_path();
  let mut suffix = Vec::new();
  while !ancestor.exists() {
    suffix.push(
      ancestor
        .file_name()
        .ok_or("path has no existing ancestor")?
        .to_owned(),
    );
    ancestor = ancestor.parent().ok_or("path has no existing parent")?;
  }
  let mut resolved = ancestor.canonicalize()?;
  for part in suffix.into_iter().rev() {
    resolved.push(part);
  }
  Ok(resolved)
}

impl Store {
  pub fn load(path: PathBuf) -> Result<Self> {
    let path = absolute(&path)?;
    let bytes = storage::read_optional(&path)?.ok_or_else(|| NoConfig(path.clone()))?;
    // Decoder diagnostics may quote a misplaced secret from the source text.
    let config: Config = toml::from_str(std::str::from_utf8(&bytes)?)
      .map_err(|_| "invalid Ccusage config; check TOML syntax and supported profile fields (contents redacted)")?;
    config.check()?;
    let root = path.parent().ok_or("config has no parent")?.to_path_buf();
    for name in config.profiles.keys() {
      if !valid_name(name) {
        return Err(
          format!("invalid profile name {name:?}; use letters, digits, - or _ (max 64)").into(),
        );
      }
    }
    let store = Self {
      path,
      root,
      config,
      data: xdg_path("XDG_DATA_HOME", ".local/share")?,
      state: xdg_path("XDG_STATE_HOME", ".local/state")?,
      cache: xdg_path("XDG_CACHE_HOME", ".cache")?,
    };
    // Sharing a credential directory would silently alias two named accounts.
    let mut dirs = BTreeMap::new();
    let shared = store.shared_dir()?;
    for name in store.config.profiles.keys() {
      let dir = store.profile_dir(name)?;
      // The shared config's own login is the one shared mode retires, so no profile
      // may be it.
      if shared.as_ref() == Some(&dir) {
        return Err(
          format!("profile {name} uses the shared Claude config directory; give it its own").into(),
        );
      }
      if let Some(other) = dirs.insert(dir, name) {
        return Err(format!("profiles {other} and {name} use the same config directory").into());
      }
    }
    Ok(store)
  }

  /// The shared Claude config directory when shared mode is on, resolved as a
  /// profile's config_dir is.
  pub fn shared_dir(&self) -> Result<Option<PathBuf>> {
    self
      .config
      .claude_config_dir
      .as_deref()
      .map(|path| existing(self.configured(path)?))
      .transpose()
  }

  /// A directory the config names: absolute or from ~ as written, else from the
  /// config's own directory.
  fn configured(&self, path: &Path) -> Result<PathBuf> {
    if path.is_absolute() || path.starts_with("~") {
      absolute(path)
    } else {
      absolute(&self.root.join(path))
    }
  }

  pub fn profile(&self, name: &str) -> Result<&Profile> {
    self
      .config
      .profiles
      .get(name)
      .ok_or_else(|| format!("unknown profile: {name}").into())
  }

  pub fn profile_dir(&self, name: &str) -> Result<PathBuf> {
    // Claude Code names a login's Keychain item after this path, so it must never move.
    let path = match &self.profile(name)?.config_dir {
      Some(path) => self.configured(path)?,
      None => self.data.join("profiles").join(name),
    };
    existing(path)
  }

  pub fn save(&self) -> Result<()> {
    storage::write_atomic(&self.path, toml::to_string_pretty(&self.config)?.as_bytes())
  }

  pub fn active(&self) -> Result<Option<String>> {
    storage::read_optional(&self.state.join("active.json"))?
      .map(|bytes| serde_json::from_slice(&bytes).map_err(Into::into))
      .transpose()
  }

  pub fn activate(&self, name: &str) -> Result<()> {
    self.profile(name)?;
    storage::write_atomic(&self.state.join("active.json"), &serde_json::to_vec(name)?)
  }

  pub fn select(&self, name: Option<&str>) -> Result<String> {
    let selected = match name {
      Some(name) => Some(name.to_owned()),
      None => self.active()?,
    }
    .ok_or("no active profile; run ccusage use NAME")?;
    self.profile(&selected)?;
    Ok(selected)
  }
}

/// The canonical form of a directory that exists, and the path as given otherwise.
fn existing(path: PathBuf) -> Result<PathBuf> {
  if path.exists() {
    Ok(path.canonicalize()?)
  } else {
    Ok(path)
  }
}

pub fn xdg_path(variable: &str, fallback: &str) -> Result<PathBuf> {
  // XDG requires absolute paths; empty or relative overrides are ignored.
  let base = env::var_os(variable)
    .map(PathBuf::from)
    .filter(|p| p.is_absolute());
  let base = match base {
    Some(base) => base,
    None => home()?.join(fallback),
  };
  absolute(&base.join("ccusage"))
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn profile_names_cannot_escape_storage() {
    for bad in ["", "..", "../work", "work/home", "a b", "a\x1b[31m"] {
      assert!(!valid_name(bad));
    }
    assert!(valid_name("work-2_personal"));
  }
  #[test]
  fn typos_and_inline_secrets_are_rejected() {
    assert!(toml::from_str::<Config>("[profiles.work]\npassword='secret'").is_err());
    assert!(toml::from_str::<Config>("[profile.work]").is_err());
  }

  #[test]
  fn the_cap_defaults_once_and_is_written_back_only_where_it_was_set() -> crate::Result<()> {
    let plain: Config = toml::from_str("[profiles.work]\nemail = 'work@example.com'\n")?;
    assert_eq!(plain.cap(), crate::which::DEFAULT_CAP);
    let written = toml::to_string_pretty(&plain)?;
    assert!(!written.contains("which") && !written.contains("cap"));
    let cap = crate::which::DEFAULT_CAP / 2.0;
    let set: Config = toml::from_str(&format!("[which]\ncap = {cap}\n"))?;
    set.check()?;
    assert_eq!(set.cap(), cap);
    let again: Config = toml::from_str(&toml::to_string_pretty(&set)?)?;
    assert_eq!(again.cap(), cap);
    // An empty table keeps its place and takes the default.
    let empty: Config = toml::from_str("[which]\n")?;
    assert_eq!(empty.cap(), crate::which::DEFAULT_CAP);
    assert!(toml::to_string_pretty(&empty)?.contains("[which]"));
    Ok(())
  }

  #[test]
  fn a_cap_that_is_not_a_percent_above_0_is_refused_as_is_an_unknown_which_key() -> crate::Result<()>
  {
    for cap in ["0", "-1", "100.5", "nan", "inf"] {
      let config: Config = toml::from_str(&format!("[which]\ncap = {cap}\n"))?;
      assert!(config.check().is_err(), "cap = {cap} passed");
    }
    let whole: Config = toml::from_str("[which]\ncap = 100\n")?;
    whole.check()?;
    assert!(toml::from_str::<Config>("[which]\ncaps = 90\n").is_err());
    Ok(())
  }
}
