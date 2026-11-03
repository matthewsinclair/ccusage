//! Every external command the xtask runs goes through a Runner, so the tests can stand in
//! for codesign, lipo, notarytool and gh.

use std::{
  ffi::{OsStr, OsString},
  process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

/// A program and its arguments.
pub struct Invocation {
  program: &'static str,
  args: Vec<OsString>,
}

impl Invocation {
  pub fn new(program: &'static str) -> Invocation {
    Invocation {
      program,
      args: Vec::new(),
    }
  }

  pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Invocation {
    self.args.push(arg.as_ref().to_owned());
    self
  }

  pub fn args<I>(self, args: I) -> Invocation
  where
    I: IntoIterator,
    I::Item: AsRef<OsStr>,
  {
    args.into_iter().fold(self, Invocation::arg)
  }

  /// The program and its arguments as text, for an error message or a test's assertion.
  pub fn words(&self) -> Vec<String> {
    let args = self
      .args
      .iter()
      .map(|arg| arg.to_string_lossy().into_owned());
    std::iter::once(self.program.to_owned())
      .chain(args)
      .collect()
  }
}

/// What happens to a command's output while it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
  /// Captured and returned, and never shown: signing tools can print Keychain diagnostics.
  Hidden,
  /// stdout captured and returned; stderr shown, so gh's errors reach the maintainer.
  Stdout,
  /// Both shown, as gh reports its downloads, uploads and edits.
  Shown,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Captured {
  pub stdout: String,
  pub stderr: String,
}

pub trait Runner {
  /// Run `invocation` to its end. A non-zero exit is an error, and the error never carries
  /// what the command printed.
  fn run(&mut self, invocation: &Invocation, output: Output) -> Result<Captured>;
}

/// Runs real processes.
pub struct System;

impl Runner for System {
  fn run(&mut self, invocation: &Invocation, output: Output) -> Result<Captured> {
    let line = invocation.words().join(" ");
    let mut command = Command::new(invocation.program);
    command.args(&invocation.args);
    if output == Output::Shown {
      let status = command
        .status()
        .with_context(|| format!("starting {line}"))?;
      if !status.success() {
        bail!("{line} failed: {status}");
      }
      return Ok(Captured::default());
    }
    if output == Output::Stdout {
      command.stderr(Stdio::inherit());
    }
    let result = command
      .output()
      .with_context(|| format!("starting {line}"))?;
    if !result.status.success() {
      bail!("{line} failed: {}", result.status);
    }
    Ok(Captured {
      stdout: String::from_utf8(result.stdout)
        .with_context(|| format!("reading what {line} printed"))?,
      stderr: String::from_utf8(result.stderr)
        .with_context(|| format!("reading what {line} printed"))?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hidden_output_is_returned_and_a_failure_never_carries_it() -> Result<()> {
    let printed = System.run(
      &Invocation::new("/bin/sh").args(["-c", "echo out; echo err >&2"]),
      Output::Hidden,
    )?;
    assert_eq!(
      printed,
      Captured {
        stdout: "out\n".into(),
        stderr: "err\n".into()
      }
    );
    // The script prints a word its own command line never spells out.
    let script = "printf 'pri%s\\n' vate; printf 'pri%s\\n' vate >&2; exit 3";
    let Err(error) = System.run(
      &Invocation::new("/bin/sh").args(["-c", script]),
      Output::Hidden,
    ) else {
      bail!("a command exiting 3 succeeded");
    };
    let failure = format!("{error:#}");
    assert!(failure.contains("exit status: 3"), "{failure}");
    assert!(!failure.contains("private"), "{failure}");
    Ok(())
  }

  #[test]
  fn shown_output_is_not_captured() -> Result<()> {
    let printed = System.run(
      &Invocation::new("/bin/sh").args(["-c", "echo shown"]),
      Output::Shown,
    )?;
    assert_eq!(printed, Captured::default());
    assert!(
      System
        .run(
          &Invocation::new("/bin/sh").args(["-c", "exit 1"]),
          Output::Shown
        )
        .is_err()
    );
    Ok(())
  }
}
