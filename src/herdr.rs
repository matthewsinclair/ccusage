//! The Herdr mod ccusage ships, and `ccusage herdr install` and `remove`, which put it
//! into a local Herdr's mods and take it out. The mod's files are embedded at build
//! time, so every install of ccusage carries the mod that matches it. The verb writes
//! only `mods/ccusage/` under Herdr's home, and never a layout or `keys.json`.

use crate::{Result, usage};
use sha2::{Digest, Sha256};
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  process::Command,
};

/// The mod's folder under Herdr's mods.
pub const MOD: &str = "ccusage";
/// What install wrote, file by file with the SHA-256 of what it wrote, kept beside them.
const RECORD: &str = ".ccusage-install.json";
/// How Herdr opens the mod as a pane.
pub const OPEN: &str = r#"herdr-app do pane.split '{"direction":"right","mod":"ccusage"}'"#;

/// The mod's files as install writes them: its source, embedded at build time, and its
/// lifetime.js, written from the usage cache's lifetime, which the panel's timer never
/// runs faster than.
fn files() -> [(&'static str, String); 3] {
  [
    (
      "main.js",
      include_str!("../herdr/ccusage/main.js").to_owned(),
    ),
    (
      "mod.json",
      include_str!("../herdr/ccusage/mod.json").to_owned(),
    ),
    (
      "lifetime.js",
      format!(
        "// Written by ccusage herdr install: the usage cache's lifetime, in seconds.\nexport const LIFETIME = {};\n",
        usage::TTL
      ),
    ),
  ]
}

/// Herdr's home: `$HERDR_APP_HOME`, else `~/.config/herdr-app`; refused when it is not
/// there, since then there is no Herdr to put the mod into.
fn home() -> Result<PathBuf> {
  let home = match std::env::var_os("HERDR_APP_HOME").filter(|home| !home.is_empty()) {
    Some(home) => PathBuf::from(home),
    None => crate::config::home()?.join(".config/herdr-app"),
  };
  if !home.is_dir() {
    return Err(
      format!(
        "no Herdr home at {}; install Herdr, or set HERDR_APP_HOME to its home",
        home.display()
      )
      .into(),
    );
  }
  Ok(home)
}

fn digest(bytes: &[u8]) -> String {
  format!("{:x}", Sha256::digest(bytes))
}

/// Whether Herdr writes `name` into a mod's folder itself, so that it says nothing of
/// an edit: `jsconfig.json`, type files, hidden files and `node_modules`.
fn herdrs_own(name: &str) -> bool {
  name.starts_with('.')
    || name == "jsconfig.json"
    || name.ends_with(".d.ts")
    || name == "node_modules"
}

/// Whether `dir` holds a copy install wrote that nobody has edited since: its record
/// names every file in it but Herdr's own, and each still has what it was written with.
fn untouched(dir: &Path) -> Result<bool> {
  let Ok(text) = fs::read_to_string(dir.join(RECORD)) else {
    return Ok(false);
  };
  let Ok(record) = serde_json::from_str::<BTreeMap<String, String>>(&text) else {
    return Ok(false);
  };
  for entry in fs::read_dir(dir)? {
    let name = entry?.file_name().to_string_lossy().into_owned();
    if herdrs_own(&name) {
      continue;
    }
    match (record.get(&name), fs::read(dir.join(&name))) {
      (Some(written), Ok(bytes)) if digest(&bytes) == *written => {}
      _ => return Ok(false),
    }
  }
  Ok(record.keys().all(|name| dir.join(name).is_file()))
}

/// The refusal for a copy install did not write, or one edited since.
fn not_ours(dir: &Path, verb: &str) -> crate::Error {
  format!(
    "{} holds a copy ccusage did not write, or one edited since, so nothing was changed; run ccusage herdr {verb} --force to {verb} it anyway",
    dir.display()
  )
  .into()
}

/// What install did, and whether herdr-app's check passed or could not be run.
pub struct Installed {
  pub said: String,
  pub passed: bool,
}

/// Writes the mod into Herdr's mods, replacing only a copy install wrote that nobody has
/// edited, and anything there with `force`; then runs `herdr-app check` on it when that
/// is on PATH, and says how to open it.
pub fn install(force: bool) -> Result<Installed> {
  let dir = home()?.join("mods").join(MOD);
  if dir.exists() {
    if !force && !untouched(&dir)? {
      return Err(not_ours(&dir, "install"));
    }
    fs::remove_dir_all(&dir)?;
  }
  fs::create_dir_all(&dir)?;
  let mut record = BTreeMap::new();
  for (name, text) in files() {
    fs::write(dir.join(name), &text)?;
    record.insert(name.to_owned(), digest(text.as_bytes()));
  }
  // Written last, so a copy whose install stopped part way is not taken for one of ours.
  fs::write(dir.join(RECORD), serde_json::to_string_pretty(&record)?)?;
  let (verdict, passed) = check(&dir);
  Ok(Installed {
    said: format!(
      "Wrote the ccusage mod to {}.\n{verdict}\nThe mod runs ccusage and nothing else. Open it as a pane with:\n  {OPEN}",
      dir.display()
    ),
    passed,
  })
}

/// Takes the mod out of Herdr's mods, on install's terms: only a copy install wrote that
/// nobody has edited, and anything there with `force`.
pub fn remove(force: bool) -> Result<String> {
  let dir = home()?.join("mods").join(MOD);
  if !dir.exists() {
    return Ok(format!(
      "No ccusage mod at {}, so nothing was removed.",
      dir.display()
    ));
  }
  if !force && !untouched(&dir)? {
    return Err(not_ours(&dir, "remove"));
  }
  fs::remove_dir_all(&dir)?;
  Ok(format!("Removed the ccusage mod from {}.", dir.display()))
}

/// herdr-app check's verdict on `dir`, and whether it passed or could not be run.
fn check(dir: &Path) -> (String, bool) {
  match Command::new("herdr-app").arg("check").arg(dir).output() {
    Ok(output) if output.status.success() => ("herdr-app check passed.".into(), true),
    Ok(output) => {
      let said = String::from_utf8_lossy(&output.stderr);
      let said = if said.trim().is_empty() {
        String::from_utf8_lossy(&output.stdout)
      } else {
        said
      };
      (
        format!("herdr-app check failed: {}", crate::clean(said.trim())),
        false,
      )
    }
    Err(_) => (
      "herdr-app is not on PATH, so the mod was not checked.".into(),
      true,
    ),
  }
}
