//! Ccusage's release tooling, run as `cargo xtask <command>`. It never ships: the root
//! package stays the workspace's default member.

mod archive;
mod checksums;
#[cfg(test)]
mod fake;
mod finalize;
mod manifest;
mod release;
mod runner;
mod sign;

use std::{env, ffi::OsString, fmt, path::PathBuf, process::ExitCode};

use anyhow::{Context, Result};

use crate::{finalize::Outcome, release::Tag, runner::System};

/// Set to a notarytool credential profile on this Mac, it turns on notarization.
const NOTARY_PROFILE: &str = "CCUSAGE_NOTARY_PROFILE";

const USAGE: &str = "Usage: cargo xtask sign-macos TAG INPUT_DIRECTORY NEW_OUTPUT_DIRECTORY
       cargo xtask finalize-release TAG
       cargo xtask formula-checksums TAG ASSET_DIRECTORY";

#[derive(Debug, PartialEq, Eq)]
enum Command {
  /// Sign the Mac archives in a directory into a new one.
  SignMacos {
    tag: Tag,
    input: PathBuf,
    output: PathBuf,
  },
  /// Sign CI's draft, verify what GitHub then holds, and publish it.
  FinalizeRelease { tag: Tag },
  /// Print the Mac archives' checksums, Apple silicon then Intel, for tools/formula.
  FormulaChecksums { tag: Tag, directory: PathBuf },
}

impl Command {
  fn name(&self) -> &'static str {
    match self {
      Command::SignMacos { .. } => "sign-macos",
      Command::FinalizeRelease { .. } => "finalize-release",
      Command::FormulaChecksums { .. } => "formula-checksums",
    }
  }
}

fn main() -> ExitCode {
  let args: Vec<OsString> = env::args_os().skip(1).collect();
  let command = match parse(&args) {
    Ok(command) => command,
    Err(usage) => {
      eprintln!("{usage}");
      return ExitCode::from(2);
    }
  };
  match run(&command) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      // Never what a signing tool printed: the runner keeps that out of every error.
      eprintln!("{}: {error:#}", command.name());
      ExitCode::FAILURE
    }
  }
}

/// Why a command line cannot run. main prints it and exits 2.
#[derive(Debug, PartialEq, Eq)]
enum Refused {
  /// Not one of the commands, or the wrong number of arguments for it.
  Usage,
  /// A tag that is not a stable release tag.
  Tag(String),
}

impl fmt::Display for Refused {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Refused::Usage => formatter.write_str(USAGE),
      Refused::Tag(tag) => write!(
        formatter,
        "Expected a stable release tag such as v0.1.0, not {tag}\n{USAGE}"
      ),
    }
  }
}

fn parse(args: &[OsString]) -> Result<Command, Refused> {
  let Some((name, rest)) = args.split_first() else {
    return Err(Refused::Usage);
  };
  match (name.to_str(), rest) {
    (Some("sign-macos"), [tag, input, output]) => Ok(Command::SignMacos {
      tag: parse_tag(tag)?,
      input: input.into(),
      output: output.into(),
    }),
    (Some("finalize-release"), [tag]) => Ok(Command::FinalizeRelease {
      tag: parse_tag(tag)?,
    }),
    (Some("formula-checksums"), [tag, directory]) => Ok(Command::FormulaChecksums {
      tag: parse_tag(tag)?,
      directory: directory.into(),
    }),
    _ => Err(Refused::Usage),
  }
}

fn parse_tag(arg: &OsString) -> Result<Tag, Refused> {
  arg
    .to_str()
    .and_then(Tag::parse)
    .ok_or_else(|| Refused::Tag(arg.to_string_lossy().into_owned()))
}

fn run(command: &Command) -> Result<()> {
  let os = env::consts::OS;
  match command {
    Command::SignMacos { tag, input, output } => sign::sign_macos(
      &mut System,
      os,
      tag,
      input,
      output,
      notary_profile()?.as_deref(),
    ),
    Command::FinalizeRelease { tag } => {
      match finalize::finalize(&mut System, os, tag, notary_profile()?.as_deref())? {
        Outcome::Verified => println!("Published release already has verified Mac signatures"),
        Outcome::Published => println!("Published {tag} with verified Mac signatures"),
      }
      Ok(())
    }
    Command::FormulaChecksums { tag, directory } => {
      for digest in manifest::mac_checksums(tag, directory)? {
        println!("{digest}");
      }
      Ok(())
    }
  }
}

/// The notarytool profile, when one is set; empty counts as unset.
fn notary_profile() -> Result<Option<String>> {
  match env::var(NOTARY_PROFILE) {
    Ok(profile) => Ok(Some(profile).filter(|profile| !profile.is_empty())),
    Err(env::VarError::NotPresent) => Ok(None),
    Err(error) => Err(error).context(NOTARY_PROFILE),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn args(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
  }

  #[test]
  fn each_command_takes_its_tag_first() -> anyhow::Result<()> {
    let tag = fake::fixture_tag()?;
    let expected = [
      (
        vec!["sign-macos", "v0.1.4", "in", "out"],
        Command::SignMacos {
          tag: tag.clone(),
          input: "in".into(),
          output: "out".into(),
        },
      ),
      (
        vec!["finalize-release", "v0.1.4"],
        Command::FinalizeRelease { tag: tag.clone() },
      ),
      (
        vec!["formula-checksums", "v0.1.4", "assets"],
        Command::FormulaChecksums {
          tag,
          directory: "assets".into(),
        },
      ),
    ];
    for (words, command) in expected {
      assert_eq!(parse(&args(&words)), Ok(command), "{words:?}");
    }
    Ok(())
  }

  #[test]
  fn wrong_arguments_get_the_usage() {
    let wrong: &[&[&str]] = &[
      &[],
      &["release"],
      &["sign-macos", "v0.1.4", "in"],
      &["sign-macos", "v0.1.4", "in", "out", "extra"],
      &["finalize-release"],
      &["finalize-release", "v0.1.4", "extra"],
      &["formula-checksums", "v0.1.4"],
    ];
    for words in wrong {
      assert_eq!(parse(&args(words)), Err(Refused::Usage), "{words:?}");
    }
    assert_eq!(Refused::Usage.to_string(), USAGE);
  }

  #[test]
  fn unstable_tags_are_refused_before_anything_runs() {
    let unstable: &[(&[&str], &str)] = &[
      (&["finalize-release", "0.1.4"], "0.1.4"),
      (&["finalize-release", "v0.1.4-rc.1"], "v0.1.4-rc.1"),
      (&["formula-checksums", "v01.1.4", "assets"], "v01.1.4"),
      (&["sign-macos", "v0.1", "in", "out"], "v0.1"),
    ];
    for (words, tag) in unstable {
      assert_eq!(
        parse(&args(words)),
        Err(Refused::Tag((*tag).into())),
        "{words:?}"
      );
    }
    let refused = Refused::Tag("0.1.4".into()).to_string();
    assert!(refused.starts_with("Expected a stable release tag such as v0.1.0, not 0.1.4\n"));
    assert!(refused.ends_with(USAGE));
  }
}
