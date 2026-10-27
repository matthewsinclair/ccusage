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
  #[serde(default)]
  pub profiles: BTreeMap<String, Profile>,
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
    let bytes = storage::read_optional(&path)?
      .ok_or_else(|| format!("no config at {}; run ccusage init", path.display()))?;
    // Decoder diagnostics may quote a misplaced secret from the source text.
    let config: Config = toml::from_str(std::str::from_utf8(&bytes)?)
      .map_err(|_| "invalid Ccusage config; check TOML syntax and supported profile fields (contents redacted)")?;
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
    for name in store.config.profiles.keys() {
      let dir = store.profile_dir(name)?;
      if let Some(other) = dirs.insert(dir, name) {
        return Err(format!("profiles {other} and {name} use the same config directory").into());
      }
    }
    Ok(store)
  }

  pub fn profile(&self, name: &str) -> Result<&Profile> {
    self
      .config
      .profiles
      .get(name)
      .ok_or_else(|| format!("unknown profile: {name}").into())
  }

  pub fn profile_dir(&self, name: &str) -> Result<PathBuf> {
    let profile = self.profile(name)?;
    let path = match &profile.config_dir {
      Some(path) if path.is_absolute() || path.starts_with("~") => absolute(path)?,
      Some(path) => absolute(&self.root.join(path))?,
      None => self.data.join("profiles").join(name),
    };
    if path.exists() {
      Ok(path.canonicalize()?)
    } else {
      Ok(path)
    }
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
}
