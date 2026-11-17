//! The Herdr mod ccusage ships, and `ccusage herdr install` and `remove`, which put it
//! into a local Herdr's mods and take it out, and `launch`, which opens it as a tab of
//! its own. The mod's files are embedded at build time, so every install of ccusage
//! carries the mod that matches it. The verb writes only `mods/ccusage/` under Herdr's
//! home, and never a layout or `keys.json`.

use crate::{Result, display, usage};
use sha2::{Digest, Sha256};
use std::{
  collections::BTreeMap,
  fs,
  io::{self, Read},
  path::{Path, PathBuf},
  process::{Child, Command, Output, Stdio},
  thread,
  time::{Duration, Instant},
};

/// The mod's folder under Herdr's mods.
pub const MOD: &str = "ccusage";
/// What install wrote, file by file with the SHA-256 of what it wrote, kept beside them.
const RECORD: &str = ".ccusage-install.json";
/// How Herdr opens the mod as a pane.
pub const OPEN: &str = r#"herdr-app do pane.split '{"direction":"right","mod":"ccusage"}'"#;
/// What launch asks herdr-app for: a tab named ccusage whose only pane is the mod.
const LAUNCH: [&str; 3] = ["do", "tab.new", r#"{"name":"ccusage","mod":"ccusage"}"#];
/// How long install waits for herdr-app check, which Herdr runs as a background job: well
/// past the two minutes one took under heavy load, so it stops only a check that hangs.
const CHECK_BOUND: Duration = Duration::from_secs(5 * 60);
/// How often the bounded wait looks to see whether the check has ended.
const CHECK_POLL: Duration = Duration::from_millis(50);

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

/// Whether Herdr writes the entry `name`, a directory when `is_dir`, into a mod's folder
/// itself, so that it says nothing of an edit: `jsconfig.json`, type files and hidden
/// files, and the `node_modules` directory. Any other directory, a hidden one such as a
/// `.git` included, is no file Herdr writes, and install and remove would take it.
fn herdrs_own(name: &str, is_dir: bool) -> bool {
  if is_dir {
    name == "node_modules"
  } else {
    name.starts_with('.') || name == "jsconfig.json" || name.ends_with(".d.ts")
  }
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
    let entry = entry?;
    let name = entry.file_name().to_string_lossy().into_owned();
    if herdrs_own(&name, entry.file_type()?.is_dir()) {
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
      "Wrote the ccusage mod to {}.\n{verdict}\nThe mod runs ccusage and nothing else. Open it as a tab of its own with ccusage herdr launch, or as a pane with:\n  {OPEN}",
      dir.display()
    ),
    passed,
  })
}

/// Opens the panel as a Herdr tab of its own, by asking herdr-app for one, once install
/// has put the mod into Herdr's mods; refused, with the reason, when it cannot.
pub fn launch() -> Result<String> {
  let dir = home()?.join("mods").join(MOD);
  if !dir.join("main.js").is_file() {
    return Err(
      format!(
        "no ccusage mod at {}, so the panel was not opened; run ccusage herdr install first",
        dir.display()
      )
      .into(),
    );
  }
  match Command::new("herdr-app").args(LAUNCH).output() {
    Ok(output) if output.status.success() => {
      Ok("Opened the ccusage panel in a new Herdr tab.".into())
    }
    Ok(output) => Err(format!("herdr-app could not open the panel: {}", said(&output)).into()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
      Err("herdr-app is not on PATH, so the panel was not opened".into())
    }
    Err(error) => {
      Err(format!("herdr-app could not be run, so the panel was not opened: {error}").into())
    }
  }
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

/// herdr-app check's verdict on `dir`, and whether it passed or was not made: when
/// herdr-app is not on PATH, cannot be run, or is still running at `CHECK_BOUND`, each
/// said with its cause. While the check runs, install says on stderr what it waits for.
fn check(dir: &Path) -> (String, bool) {
  let spawned = Command::new("herdr-app")
    .arg("check")
    .arg(dir)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn();
  match spawned {
    Ok(child) => {
      eprintln!("ccusage: {}", waiting(CHECK_BOUND));
      verdict(wait_within(child, CHECK_BOUND), dir, CHECK_BOUND)
    }
    Err(error) if error.kind() == io::ErrorKind::NotFound => (
      "herdr-app is not on PATH, so the mod was not checked.".into(),
      true,
    ),
    Err(error) => (
      format!("herdr-app could not be run, so the mod was not checked: {error}"),
      true,
    ),
  }
}

/// What a thread that reads one of a child's pipes to its end gives back.
type Drain = thread::JoinHandle<io::Result<Vec<u8>>>;

/// `child`'s output once it exits, or None when it is still running at `bound`, when it is
/// killed and reaped. Each pipe the caller gave it is read on a thread of its own
/// meanwhile, so a child that writes more than a pipe holds is not stalled; a stopped
/// child's readers are left to end with its pipes.
fn wait_within(mut child: Child, bound: Duration) -> io::Result<Option<Output>> {
  let (stdout, stderr) = (drain(child.stdout.take()), drain(child.stderr.take()));
  let deadline = Instant::now() + bound;
  loop {
    if let Some(status) = child.try_wait()? {
      return Ok(Some(Output {
        status,
        stdout: drained(stdout)?,
        stderr: drained(stderr)?,
      }));
    }
    if Instant::now() >= deadline {
      child.kill()?;
      child.wait()?;
      return Ok(None);
    }
    thread::sleep(CHECK_POLL);
  }
}

/// A thread that reads `pipe` to its end, or None for a pipe the child was not given.
fn drain(pipe: Option<impl Read + Send + 'static>) -> Option<Drain> {
  pipe.map(|mut pipe| {
    thread::spawn(move || {
      let mut bytes = Vec::new();
      pipe.read_to_end(&mut bytes).map(|_| bytes)
    })
  })
}

/// What `drain`'s thread read, or nothing for a pipe the child was not given.
fn drained(reader: Option<Drain>) -> io::Result<Vec<u8>> {
  match reader {
    Some(reader) => reader
      .join()
      .map_err(|_| io::Error::other("a thread reading a child's output panicked"))?,
    None => Ok(Vec::new()),
  }
}

/// What install says on stderr while herdr-app check runs, which `bound` stops. Pure.
fn waiting(bound: Duration) -> String {
  format!(
    "waiting for herdr-app check, which Herdr runs as a background job and can take a minute or two; ccusage waits up to {}",
    shown(bound)
  )
}

/// `span` as ccusage shows a span of time, such as 5m; saturating, for a span past what
/// the seconds of an i64 hold. Pure.
fn shown(span: Duration) -> String {
  display::duration(i64::try_from(span.as_secs()).unwrap_or(i64::MAX))
}

/// herdr-app check's verdict on `dir` from its run, which is None when `bound` stopped it,
/// and whether it passed or was not made. Pure.
fn verdict(run: io::Result<Option<Output>>, dir: &Path, bound: Duration) -> (String, bool) {
  match run {
    Ok(Some(output)) if output.status.success() => ("herdr-app check passed.".into(), true),
    Ok(Some(output)) => (format!("herdr-app check failed: {}", said(&output)), false),
    Ok(None) => (
      format!(
        "herdr-app check had not finished after {}, so ccusage stopped waiting and the mod was not checked; check it with herdr-app check {}",
        shown(bound),
        dir.display()
      ),
      true,
    ),
    Err(error) => (
      format!("herdr-app check could not be waited for, so the mod was not checked: {error}"),
      true,
    ),
  }
}

/// What herdr-app said when it refused: the text of the `{ "error": ... }` that
/// `herdr-app do` prints, else its stderr, else its stdout, made safe for a terminal.
fn said(output: &Output) -> String {
  let stdout = String::from_utf8_lossy(&output.stdout);
  let error = serde_json::from_str::<serde_json::Value>(&stdout)
    .ok()
    .and_then(|reply| reply["error"].as_str().map(str::to_owned));
  let stderr = String::from_utf8_lossy(&output.stderr);
  let said = match error {
    Some(error) => error,
    None if stderr.trim().is_empty() => stdout.into_owned(),
    None => stderr.into_owned(),
  };
  crate::clean(said.trim())
}

#[cfg(test)]
mod tests {
  use super::*;

  // ISSUE:0011 Acceptance 4: the bounded wait stops a child still running at its bound, so
  // it ends long before the child would have, and no test waits for the check's bound.
  #[test]
  fn the_bounded_wait_stops_a_child_still_running_at_its_bound() -> Result<()> {
    let sleep = Duration::from_secs(5);
    let child = Command::new("sleep")
      .arg(sleep.as_secs().to_string())
      .spawn()?;
    let started = Instant::now();
    assert!(wait_within(child, sleep / 20)?.is_none());
    assert!(started.elapsed() < sleep);
    Ok(())
  }

  // ISSUE:0011: a child that ends within its bound gives its exit status and what it wrote
  // to each pipe, as Command::output gave them.
  #[test]
  fn the_bounded_wait_gives_what_a_child_that_ends_in_time_wrote() -> Result<()> {
    let code = 3;
    let child = Command::new("sh")
      .args(["-c", &format!("printf out; printf err >&2; exit {code}")])
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()?;
    let output =
      wait_within(child, Duration::from_secs(60))?.ok_or("the child ended within its bound")?;
    assert_eq!(output.status.code(), Some(code));
    assert_eq!(output.stdout, b"out");
    assert_eq!(output.stderr, b"err");
    Ok(())
  }

  // ISSUE:0011 Acceptance 2: a check stopped at its bound is said as not made, with how
  // long install waited and the command that makes it by hand, and it passes, so install
  // exits 0.
  #[test]
  fn a_check_stopped_at_its_bound_is_reported_as_not_checked() -> Result<()> {
    let dir = Path::new("/home/u/.config/herdr-app/mods/ccusage");
    let bound = Duration::from_secs(120);
    let (said, passed) = verdict(Ok(None), dir, bound);
    assert!(passed);
    let waited = crate::display::duration(i64::try_from(bound.as_secs())?);
    assert!(said.contains(&format!("had not finished after {waited}")));
    assert!(said.contains("the mod was not checked"));
    assert!(said.ends_with(&format!("herdr-app check {}", dir.display())));
    Ok(())
  }
}
