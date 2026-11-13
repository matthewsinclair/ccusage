use crate::{
  Result, auth,
  config::{self, Config, Profile, Store},
  display, storage,
  usage::Client,
};
use std::path::PathBuf;

const HELP: &str = "Ccusage: named Claude Code accounts and plan usage

Usage: ccusage [--config PATH] [--no-color] [--verbose] [COMMAND]

  (no command) [--all] [--json] [--refresh]
                               Show plan usage; same as ccusage usage
  init                         Create an empty XDG config (never overwrite)
  add NAME [--email EMAIL] [--dir PATH]
                               Register a profile; optionally use an existing Claude directory
  login NAME                   Sign in through Claude's browser login
  logout NAME                  Sign out this profile through Claude
  list                         Show profiles and local credential status
  use NAME                     Select the account for future ccusage run launches
                               (in shared mode, for new Claude sessions too)
  run [NAME] [-- CLAUDE_ARGS...]
                               Launch Claude with the selected profile
  dir [NAME]                   In shared mode, print the active or named profile's
                               directory, for CLAUDE_SECURESTORAGE_CONFIG_DIR
  usage [NAME | --all] [--json] [--refresh] [--verbose]
                               Show plan usage (default: active account)
  usage --demo [--json] [--verbose]
                               Show plan usage from fixture data, with no account
  tui [--demo]                 Interactive account dashboard (demo works without setup)
  shell zsh|bash               Print an optional wrapper for plain claude
  paths                        Show resolved XDG directories
  doctor                       Check Claude, configuration, and private-file permissions
  help                         Show this help
  --version                    Show version

Config: $XDG_CONFIG_HOME/ccusage/config.toml (default ~/.config/ccusage/config.toml)
Overrides: CCUSAGE_CONFIG or --config PATH. Also honors XDG_DATA_HOME,
XDG_STATE_HOME, XDG_CACHE_HOME and NO_COLOR. Usage is cached for five minutes.
The usage endpoint is undocumented and may change. No model inference is made.";

/// Usage flags that may stand where the command would, which then runs usage.
const USAGE_FLAGS: &[&str] = &["--all", "--json", "--refresh"];

fn argument(args: &mut Vec<String>, flag: &str) -> Result<Option<String>> {
  if let Some(index) = args.iter().position(|arg| arg == flag) {
    args.remove(index);
    if index == args.len() || args[index].starts_with("--") {
      return Err(format!("{flag} requires a value").into());
    }
    return Ok(Some(args.remove(index)));
  }
  Ok(None)
}
fn switch(args: &mut Vec<String>, flag: &str) -> bool {
  if let Some(index) = args.iter().position(|arg| arg == flag) {
    args.remove(index);
    true
  } else {
    false
  }
}
fn one(args: &[String]) -> Result<&str> {
  if args.len() != 1 || args[0].starts_with('-') {
    return Err("expected exactly one profile name; see ccusage help".into());
  }
  Ok(&args[0])
}
fn empty(args: &[String]) -> Result<()> {
  if args.is_empty() {
    Ok(())
  } else {
    Err(format!("unexpected arguments: {}; see ccusage help", args.join(" ")).into())
  }
}

/// `version` gives the binary's --version line (devbin's T4), which the binary reads,
/// since only it knows its own name and tree.
pub fn run(mut args: Vec<String>, version: fn() -> String) -> Result<()> {
  // Only parse global flags before the command, so Claude arguments pass through intact.
  let mut path = config::default_path()?;
  let mut no_color = false;
  let mut verbose = false;
  while args.first().is_some_and(|arg| arg.starts_with("--")) {
    match args[0].as_str() {
      "--config" => {
        args.remove(0);
        if args.is_empty() {
          return Err("--config requires a path".into());
        }
        path = PathBuf::from(args.remove(0));
      }
      "--no-color" => {
        args.remove(0);
        no_color = true;
      }
      "--verbose" => {
        args.remove(0);
        verbose = true;
      }
      "--help" => {
        println!("{HELP}");
        return Ok(());
      }
      "--version" => {
        println!("{}", version());
        return Ok(());
      }
      flag if USAGE_FLAGS.contains(&flag) => break,
      _ => return Err(format!("unknown option: {}", args[0]).into()),
    }
  }
  // No command, or a usage flag in its place, runs usage on the rest of the line.
  let implied = args
    .first()
    .is_none_or(|arg| USAGE_FLAGS.contains(&arg.as_str()));
  let command = if implied {
    "usage".to_owned()
  } else {
    args.remove(0)
  };
  if verbose && command != "usage" {
    return Err(
      "--verbose is available for usage commands; try ccusage usage --all --verbose".into(),
    );
  }
  if command == "help" {
    empty(&args)?;
    println!("{HELP}");
    return Ok(());
  }
  if command == "doctor" {
    empty(&args)?;
    return crate::doctor::run(&config::absolute(&path)?);
  }
  if command == "shell" {
    let shell = one(&args)?;
    if shell != "zsh" && shell != "bash" {
      return Err("supported shells: zsh, bash".into());
    }
    println!(
      "# Add to ~/.{shell}rc after installing ccusage on PATH.\nclaude() {{ command ccusage run -- \"$@\"; }}"
    );
    return Ok(());
  }
  if command == "paths" {
    empty(&args)?;
    println!(
      "config  {}\ndata    {}\nstate   {}\ncache   {}",
      config::absolute(&path)?.display(),
      config::xdg_path("XDG_DATA_HOME", ".local/share")?.display(),
      config::xdg_path("XDG_STATE_HOME", ".local/state")?.display(),
      config::xdg_path("XDG_CACHE_HOME", ".cache")?.display()
    );
    return Ok(());
  }
  if command == "init" {
    empty(&args)?;
    let path = config::absolute(&path)?;
    if path.exists() {
      return Err(format!("config already exists: {}", path.display()).into());
    }
    storage::create_config(
      &path,
      toml::to_string_pretty(&Config::default())?.as_bytes(),
    )?;
    println!(
      "Created {}. Next: ccusage add work --email you@example.com",
      path.display()
    );
    return Ok(());
  }
  if command == "tui" && switch(&mut args, "--demo") {
    empty(&args)?;
    return display::tui(None, true, no_color);
  }
  if command == "usage" && switch(&mut args, "--demo") {
    let json = switch(&mut args, "--json");
    verbose |= switch(&mut args, "--verbose");
    empty(&args)?;
    let reports = crate::usage::demo();
    if json {
      println!("{}", serde_json::to_string_pretty(&reports)?);
    } else {
      display::print(&reports, None, no_color, verbose)?;
    }
    return Ok(());
  }
  let mut store = match Store::load(path) {
    // Bare ccusage is where a first run starts, so it also points to the help.
    Err(error) if implied && error.is::<config::NoConfig>() => {
      return Err(format!("{error}, or see ccusage help").into());
    }
    loaded => loaded?,
  };
  match command.as_str() {
    "add" => {
      let email = argument(&mut args, "--email")?;
      let dir = argument(&mut args, "--dir")?.map(PathBuf::from);
      let name = one(&args)?.to_owned();
      if !config::valid_name(&name) {
        return Err("name must use letters, digits, - or _ (max 64)".into());
      }
      if store.config.profiles.contains_key(&name) {
        return Err(format!("profile {name} already exists").into());
      }
      store.config.profiles.insert(
        name.clone(),
        Profile {
          email,
          config_dir: dir,
        },
      );
      let new_dir = store.profile_dir(&name)?;
      for other in store.config.profiles.keys().filter(|other| *other != &name) {
        if store.profile_dir(other)? == new_dir {
          return Err(format!("profile {other} already uses that directory").into());
        }
      }
      store.save()?;
      println!("Added {name}. Next: ccusage login {name}");
    }
    "login" => {
      auth::login(&store, one(&args)?)?;
    }
    "logout" => {
      let name = one(&args)?;
      let status = auth::command(&store.profile_dir(name)?)
        .args(["auth", "logout"])
        .status()?;
      if !status.success() {
        return Err(format!("Claude logout exited with {status}").into());
      }
      clear_cache(&store, name)?;
      println!("Signed out {name}.");
    }
    "list" => {
      empty(&args)?;
      let active = store.active()?;
      if store.config.profiles.is_empty() {
        println!("No profiles. Run ccusage add NAME.");
      }
      for name in store.config.profiles.keys() {
        let marker = if active.as_deref() == Some(name) {
          '*'
        } else {
          ' '
        };
        let status = match auth::read(&store.profile_dir(name)?) {
          Ok(token)
            if token
              .expires_at
              .is_some_and(|expiry| expiry / 1000 <= crate::now()) =>
          {
            "login stored; token expired (launch Claude to refresh)".into()
          }
          Ok(_) => "authenticated".into(),
          Err(error) => crate::clean(&error.to_string()),
        };
        println!("{marker} {name:<20} {status}");
      }
    }
    "use" => {
      let name = one(&args)?;
      println!("{}", crate::switch::to(&store, name)?.message(name));
    }
    "dir" => {
      if args.len() > 1 {
        return Err("dir accepts one profile name; see ccusage help".into());
      }
      if store.shared_dir()?.is_none() {
        return Err(
          "shared mode is off; set claude_config_dir in the config to use ccusage dir".into(),
        );
      }
      let name = store.select(args.first().map(String::as_str))?;
      let dir = store.profile_dir(&name)?;
      // The exact bytes Claude Code hashes into the login's Keychain name.
      #[cfg(unix)]
      {
        use std::{io::Write, os::unix::ffi::OsStrExt};
        let mut out = std::io::stdout().lock();
        out.write_all(dir.as_os_str().as_bytes())?;
        out.write_all(b"\n")?;
      }
      #[cfg(not(unix))]
      println!("{}", dir.display());
    }
    "run" => {
      let name = if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        Some(args.remove(0))
      } else {
        None
      };
      if args.first().is_some_and(|arg| arg == "--") {
        args.remove(0);
      }
      let name = store.select(name.as_deref())?;
      let dir = store.profile_dir(&name)?;
      // Require an authenticated slot: an accidental fresh login must be explicit.
      auth::read(&dir)?;
      let mut child = match store.shared_dir()? {
        Some(shared) => {
          // The shared config's account record must match the login it is given.
          if store.active()?.as_deref() != Some(name.as_str()) {
            crate::switch::to(&store, &name)?;
          }
          auth::shared_command(&shared, &dir)
        }
        None => auth::command(&dir),
      };
      child.args(args);
      #[cfg(unix)]
      {
        use std::os::unix::process::CommandExt;
        return Err(child.exec().into());
      }
      #[cfg(not(unix))]
      {
        let status = child.status()?;
        std::process::exit(status.code().unwrap_or(1));
      }
    }
    "usage" => {
      verbose |= switch(&mut args, "--verbose");
      let all = switch(&mut args, "--all");
      let json = switch(&mut args, "--json");
      let refresh = switch(&mut args, "--refresh");
      let names = if all {
        empty(&args)?;
        store.config.profiles.keys().cloned().collect::<Vec<_>>()
      } else {
        if args.len() > 1 {
          return Err("usage accepts one name or --all".into());
        }
        vec![store.select(args.first().map(String::as_str))?]
      };
      if names.is_empty() {
        return Err("no profiles; run ccusage add NAME".into());
      }
      let client = Client::default();
      let reports: Vec<_> = names
        .iter()
        .map(|name| client.report(&store, name, refresh))
        .collect();
      if json {
        println!("{}", serde_json::to_string_pretty(&reports)?);
      } else {
        display::print(&reports, store.active()?.as_deref(), no_color, verbose)?;
      }
      if reports.iter().any(|report| report.error.is_some()) {
        return Err("some accounts could not refresh; see per-account errors".into());
      }
    }
    "tui" => {
      empty(&args)?;
      display::tui(Some(&store), false, no_color)?;
    }
    _ => {
      return Err(format!("unknown command: {command}; see ccusage help").into());
    }
  }
  Ok(())
}

fn clear_cache(store: &Store, name: &str) -> Result<()> {
  match std::fs::remove_file(store.cache.join(format!("{name}.json"))) {
    Ok(()) => Ok(()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
    Err(error) => Err(error.into()),
  }
}
