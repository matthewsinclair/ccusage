//! sign-macos: sign the two Mac archives with the Developer ID, whose private key stays in
//! the maintainer's Keychain, and bind all four archives to SHA256SUMS and
//! MACOS-SIGNING.json. Those two are written last, so a failed run leaves neither.

use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  path::Path,
};

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{
  archive,
  checksums::{self, Digests},
  manifest::{Manifest, Signed},
  release::{BINARY, IDENTIFIER, IDENTITY, MEMBER_LIMIT, REQUIREMENT, SUMS, TARGETS, TEAM, Tag},
  runner::{Captured, Invocation, Output, Runner},
};

pub const CODESIGN: &str = "/usr/bin/codesign";
pub const LIPO: &str = "/usr/bin/lipo";
pub const DITTO: &str = "/usr/bin/ditto";
pub const XCRUN: &str = "/usr/bin/xcrun";

pub const WRONG_ARCH: &str = "Release binary has the wrong architecture";
pub const NO_TIMESTAMP: &str = "Developer ID signing requires a secure timestamp";
pub const NO_RUNTIME: &str = "Developer ID signing requires the hardened runtime";
pub const UNSTABLE: &str = "Signature does not have a stable designated requirement";
pub const NOT_NOTARIZED: &str = "Notarization was not accepted";
pub const DIFFERENT_IDENTITIES: &str = "Mac architectures do not share the same signing identity";
pub const NOT_MAC: &str = "Run signing on the maintainer's Mac";

/// Signing runs only where the Developer ID's private key is: the maintainer's Mac.
pub fn require_mac(os: &str, refusal: &str) -> Result<()> {
  if os != "macos" {
    bail!("{refusal}");
  }
  Ok(())
}

/// The sign-macos command: sign the archives in `input` into the new directory `output`.
pub fn sign_macos(
  runner: &mut dyn Runner,
  os: &str,
  tag: &Tag,
  input: &Path,
  output: &Path,
  notary: Option<&str>,
) -> Result<()> {
  require_mac(os, NOT_MAC)?;
  prepare(runner, tag, input, output, notary, MEMBER_LIMIT)
}

/// Sign each Mac archive in `input`, copy each Linux one as it is, and write all four with
/// SHA256SUMS and MACOS-SIGNING.json into `output`, which must not exist yet. With a
/// notarytool profile, each signed binary must also be notarized.
pub fn prepare(
  runner: &mut dyn Runner,
  tag: &Tag,
  input: &Path,
  output: &Path,
  notary: Option<&str>,
  limit: u64,
) -> Result<()> {
  if let Some(parent) = output.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::create_dir(output).with_context(|| format!("creating {}", output.display()))?;
  let mut requirements = BTreeMap::new();
  for target in TARGETS {
    let name = tag.archive(target);
    let (source, destination) = (input.join(&name), output.join(&name));
    let Some(arch) = target.mac_arch else {
      fs::copy(&source, &destination).with_context(|| format!("copying {name}"))?;
      continue;
    };
    let work = tempfile::Builder::new().prefix("ccusage-sign-").tempdir()?;
    archive::extract(&source, work.path(), limit).with_context(|| format!("unpacking {name}"))?;
    let requirement = sign(runner, &work.path().join(BINARY), arch, notary)
      .with_context(|| format!("signing {name}"))?;
    archive::pack(work.path(), &destination).with_context(|| format!("packing {name}"))?;
    requirements.insert(name, requirement);
  }
  if requirements.values().collect::<BTreeSet<_>>().len() != 1 {
    bail!(DIFFERENT_IDENTITIES);
  }
  let digests = tag
    .archives()
    .map(|name| {
      let digest = checksums::file_digest(&output.join(&name))?;
      Ok((name, digest))
    })
    .collect::<Result<Digests>>()?;
  fs::write(output.join(SUMS), checksums::render(tag, &digests)?)?;
  let archives = requirements
    .into_iter()
    .map(|(name, requirement)| {
      let sha256 = digests
        .get(&name)
        .cloned()
        .with_context(|| format!("no checksum for {name}"))?;
      let signed = Signed {
        requirement,
        notarized: notary.is_some(),
        sha256,
      };
      Ok((name, signed))
    })
    .collect::<Result<_>>()?;
  let manifest = Manifest {
    tag: tag.to_string(),
    identifier: IDENTIFIER.into(),
    team_id: TEAM.into(),
    archives,
  };
  manifest.write(output)
}

/// Sign one Mac binary once lipo has shown it is the target's architecture, and return its
/// designated requirement.
fn sign(
  runner: &mut dyn Runner,
  binary: &Path,
  arch: &str,
  notary: Option<&str>,
) -> Result<String> {
  let reported = runner.run(
    &Invocation::new(LIPO).arg("-archs").arg(binary),
    Output::Hidden,
  )?;
  if reported.stdout.trim() != arch {
    bail!(WRONG_ARCH);
  }
  let signing = Invocation::new(CODESIGN)
    .args([
      "--force",
      "--sign",
      IDENTITY,
      "--identifier",
      IDENTIFIER,
      "--options",
      "runtime",
      "--timestamp",
    ])
    .arg(binary);
  runner.run(&signing, Output::Hidden)?;
  let requirement = verify(runner, binary)?;
  if let Some(profile) = notary {
    notarize(runner, binary, profile)?;
  }
  Ok(requirement)
}

/// Submit a signed binary with a notarytool profile that already exists on this Mac; no
/// notarization credential is exported.
fn notarize(runner: &mut dyn Runner, binary: &Path, profile: &str) -> Result<()> {
  let zip = binary.with_file_name("notarize.zip");
  runner.run(
    &Invocation::new(DITTO)
      .args(["-c", "-k", "--keepParent"])
      .arg(binary)
      .arg(&zip),
    Output::Hidden,
  )?;
  let submission = Invocation::new(XCRUN)
    .args(["notarytool", "submit"])
    .arg(&zip)
    .args([
      "--keychain-profile",
      profile,
      "--wait",
      "--output-format",
      "json",
    ]);
  let answer = runner.run(&submission, Output::Hidden)?;
  let answer: Value =
    serde_json::from_str(&answer.stdout).context("reading notarytool's answer")?;
  if answer.get("status").and_then(Value::as_str) != Some("Accepted") {
    bail!(NOT_NOTARIZED);
  }
  Ok(())
}

/// Check a signed binary against REQUIREMENT, then read its signature, which must carry a
/// secure timestamp and the hardened runtime. Returns its designated requirement, the line
/// MACOS-SIGNING.json records.
pub fn verify(runner: &mut dyn Runner, binary: &Path) -> Result<String> {
  let check = Invocation::new(CODESIGN)
    .args(["--verify", "--strict", "-R"])
    .arg(format!("={REQUIREMENT}"))
    .arg(binary);
  runner.run(&check, Output::Hidden)?;
  let display = Invocation::new(CODESIGN)
    .args(["-d", "--verbose=4", "-r-"])
    .arg(binary);
  designated_requirement(&runner.run(&display, Output::Hidden)?)
}

/// Read what `codesign -d --verbose=4 -r-` printed: the designated requirement on stdout,
/// and the signature's details on stderr.
pub fn designated_requirement(display: &Captured) -> Result<String> {
  let lines = || display.stdout.lines().chain(display.stderr.lines());
  if !lines().any(|line| line.starts_with("Timestamp=")) {
    bail!(NO_TIMESTAMP);
  }
  if !lines().any(hardened) {
    bail!(NO_RUNTIME);
  }
  match lines().find(|line| line.starts_with("designated =>")) {
    Some(requirement) if !requirement.contains("cdhash") => Ok(requirement.to_owned()),
    _ => bail!(UNSTABLE),
  }
}

/// A CodeDirectory line whose flags name the hardened runtime, as in
/// `CodeDirectory v=20500 size=7583 flags=0x10000(runtime) hashes=231+2 location=embedded`.
fn hardened(line: &str) -> bool {
  let Some(fields) = line.strip_prefix("CodeDirectory ") else {
    return false;
  };
  fields
    .split_whitespace()
    .filter_map(|field| field.strip_prefix("flags="))
    .filter_map(|flags| {
      flags
        .split_once('(')
        .and_then(|(_, names)| names.strip_suffix(')'))
    })
    .any(|names| names.split(',').any(|name| name == "runtime"))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    archive::UNEXPECTED,
    fake::{
      DESIGNATED, Fake, Member, ci_release, fixture_tag, signed_display, tarball, write_assets,
    },
    release::{FILES, MANIFEST, mac_targets},
  };

  /// A Developer ID display with `line` taken out of the details.
  fn without(line: &str) -> Captured {
    let display = signed_display();
    let stderr = display
      .stderr
      .lines()
      .filter(|kept| !kept.starts_with(line))
      .map(|kept| format!("{kept}\n"))
      .collect();
    Captured { stderr, ..display }
  }

  fn refusal(display: &Captured) -> String {
    match designated_requirement(display) {
      Ok(requirement) => requirement,
      Err(error) => error.to_string(),
    }
  }

  /// CI's four archives for the fixture tag in `<work>/in`, and where `prepare` signs them to.
  fn ci_input(work: &Path) -> Result<(Tag, std::path::PathBuf, std::path::PathBuf)> {
    let tag = fixture_tag()?;
    let input = work.join("in");
    fs::create_dir(&input)?;
    write_assets(&input, &ci_release(&tag)?)?;
    Ok((tag, input, work.join("out")))
  }

  fn root_cause(result: Result<()>) -> String {
    result
      .err()
      .map(|error| error.root_cause().to_string())
      .unwrap_or_default()
  }

  #[test]
  fn the_v0_1_5_signature_reads_as_stable() {
    assert_eq!(refusal(&signed_display()), DESIGNATED.trim_end());
  }

  #[test]
  fn the_requirement_is_read_from_stdout() {
    let display = signed_display();
    assert!(
      display.stdout.starts_with("designated =>") && !display.stderr.contains("designated =>")
    );
    assert!(designated_requirement(&display).is_ok());
  }

  #[test]
  fn a_signature_without_a_secure_timestamp_is_refused() {
    assert_eq!(refusal(&without("Timestamp=")), NO_TIMESTAMP);
  }

  #[test]
  fn a_signature_without_the_hardened_runtime_is_refused() {
    let display = signed_display();
    let plain = display
      .stderr
      .replace("flags=0x10000(runtime)", "flags=0x0(none)");
    assert_eq!(
      refusal(&Captured {
        stderr: plain,
        ..display.clone()
      }),
      NO_RUNTIME
    );
    // Runtime Version= is not the flag.
    let renamed = display
      .stderr
      .replace("flags=0x10000(runtime)", "flags=0x0(none)")
      .replace("Runtime Version", "runtime");
    assert_eq!(
      refusal(&Captured {
        stderr: renamed,
        ..display.clone()
      }),
      NO_RUNTIME
    );
    let combined = display
      .stderr
      .replace("flags=0x10000(runtime)", "flags=0x10002(adhoc,runtime)");
    assert!(
      designated_requirement(&Captured {
        stderr: combined,
        ..display
      })
      .is_ok()
    );
  }

  #[test]
  fn a_missing_or_cdhash_requirement_is_refused() {
    let display = signed_display();
    assert_eq!(
      refusal(&Captured {
        stdout: String::new(),
        ..display.clone()
      }),
      UNSTABLE
    );
    let pinned = Captured {
      stdout: "designated => cdhash H\"0509a96c0991831c182058c8c8f4f4ccd433a87c\"\n".into(),
      ..display
    };
    assert_eq!(refusal(&pinned), UNSTABLE);
  }

  #[test]
  fn verification_checks_the_geodica_requirement_first() -> Result<()> {
    let mut fake = Fake::default();
    verify(&mut fake, Path::new("ccusage"))?;
    let requirement = format!("={REQUIREMENT}");
    let expected = [
      vec![
        CODESIGN,
        "--verify",
        "--strict",
        "-R",
        &requirement,
        "ccusage",
      ],
      vec![CODESIGN, "-d", "--verbose=4", "-r-", "ccusage"],
    ];
    assert_eq!(fake.calls, expected);
    fake.failing = Some("/usr/bin/codesign --verify");
    assert!(verify(&mut fake, Path::new("ccusage")).is_err());
    Ok(())
  }

  /// macOS parses the requirement verification hands codesign; without the leading `=`,
  /// both codesign and csreq would read it as a file name.
  #[cfg(target_os = "macos")]
  #[test]
  fn macos_parses_the_verification_requirement() -> Result<()> {
    let mut fake = Fake::default();
    verify(&mut fake, Path::new("ccusage"))?;
    let check = fake.calls.first().cloned().unwrap_or_default();
    let requirement = check
      .iter()
      .skip_while(|word| *word != "-R")
      .nth(1)
      .cloned()
      .unwrap_or_default();
    let work = tempfile::tempdir()?;
    let compiled = std::process::Command::new("/usr/bin/csreq")
      .args(["-r", &requirement, "-b"])
      .arg(work.path().join("requirement"))
      .output()?;
    assert!(
      compiled.status.success(),
      "{}",
      String::from_utf8_lossy(&compiled.stderr)
    );
    Ok(())
  }

  #[test]
  fn signing_keeps_linux_archives_and_binds_every_checksum() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    let mut fake = Fake::default();
    prepare(&mut fake, &tag, &input, &output, None, MEMBER_LIMIT)?;
    let digests = checksums::validate(&output, &tag)?;
    let manifest = Manifest::read(&output)?;
    assert_eq!(
      manifest.mac_archives(&tag, &digests)?.len(),
      mac_targets().count()
    );
    for target in TARGETS {
      let name = tag.archive(target);
      let signed = manifest.archives.get(&name);
      match target.mac_arch {
        None => {
          assert_eq!(
            fs::read(input.join(&name))?,
            fs::read(output.join(&name))?,
            "{name}"
          );
          assert!(signed.is_none(), "{name}");
        }
        Some(_) => {
          let signed = signed.with_context(|| format!("{name} is not in the manifest"))?;
          assert_eq!(signed.requirement, DESIGNATED.trim_end());
          assert!(!signed.notarized);
          let unpacked = work.path().join(target.triple);
          fs::create_dir(&unpacked)?;
          archive::extract(&output.join(&name), &unpacked, MEMBER_LIMIT)?;
          let binary = fs::read(unpacked.join(BINARY))?;
          assert_eq!(binary, Fake::signed(target.triple.as_bytes()), "{name}");
        }
      }
    }
    let signing: Vec<&Vec<String>> = fake
      .calls
      .iter()
      .filter(|call| call.get(1).is_some_and(|word| word == "--force"))
      .collect();
    assert_eq!(signing.len(), mac_targets().count());
    for call in signing {
      let (words, binary) = call.split_at(call.len() - 1);
      assert_eq!(
        words,
        [
          CODESIGN,
          "--force",
          "--sign",
          IDENTITY,
          "--identifier",
          IDENTIFIER,
          "--options",
          "runtime",
          "--timestamp"
        ]
      );
      assert!(
        binary.iter().all(|path| path.ends_with("/ccusage")),
        "{binary:?}"
      );
    }
    Ok(())
  }

  #[test]
  fn notarization_must_be_accepted_and_is_recorded() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    let mut fake = Fake::default();
    prepare(
      &mut fake,
      &tag,
      &input,
      &output,
      Some("ccusage-notary"),
      MEMBER_LIMIT,
    )?;
    assert!(
      Manifest::read(&output)?
        .archives
        .values()
        .all(|signed| signed.notarized)
    );
    let submissions: Vec<&Vec<String>> = fake
      .calls
      .iter()
      .filter(|call| call.first().is_some_and(|program| program == XCRUN))
      .collect();
    assert_eq!(submissions.len(), mac_targets().count());
    for call in submissions {
      let profile = call
        .iter()
        .skip_while(|word| *word != "--keychain-profile")
        .nth(1);
      assert_eq!(profile.map(String::as_str), Some("ccusage-notary"));
      assert!(call.iter().any(|word| word == "--wait"));
    }
    for answer in [r#"{"status":"Invalid"}"#, "{}"] {
      let out = work.path().join(format!("rejected-{}", answer.len()));
      let mut fake = Fake {
        notary: answer,
        ..Fake::default()
      };
      let result = prepare(
        &mut fake,
        &tag,
        &input,
        &out,
        Some("ccusage-notary"),
        MEMBER_LIMIT,
      );
      assert_eq!(root_cause(result), NOT_NOTARIZED, "{answer}");
    }
    Ok(())
  }

  #[test]
  fn a_binary_of_the_wrong_architecture_is_refused() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    let mut fake = Fake {
      arch: Some("i386"),
      ..Fake::default()
    };
    assert_eq!(
      root_cause(prepare(
        &mut fake,
        &tag,
        &input,
        &output,
        None,
        MEMBER_LIMIT
      )),
      WRONG_ARCH
    );
    assert_eq!(fake.ran(&[CODESIGN]), 0);
    Ok(())
  }

  #[test]
  fn a_failed_signing_writes_neither_sums_nor_manifest() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    let mut fake = Fake {
      failing: Some("/usr/bin/codesign --force"),
      ..Fake::default()
    };
    assert!(prepare(&mut fake, &tag, &input, &output, None, MEMBER_LIMIT).is_err());
    assert!(!output.join(SUMS).exists());
    assert!(!output.join(MANIFEST).exists());
    Ok(())
  }

  #[test]
  fn architectures_signed_by_different_identities_are_refused() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    let other = Captured {
      stdout: "designated => identifier \"com.geodica.ccusage\" and anchor apple generic and certificate leaf[subject.OU] = \"0000000000\"\n".into(),
      ..signed_display()
    };
    let mut fake = Fake {
      displays: vec![signed_display(), other],
      ..Fake::default()
    };
    let result = prepare(&mut fake, &tag, &input, &output, None, MEMBER_LIMIT);
    assert_eq!(root_cause(result), DIFFERENT_IDENTITIES);
    assert!(!output.join(MANIFEST).exists());
    Ok(())
  }

  #[test]
  fn an_unexpected_archive_member_stops_signing() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    let mut members: Vec<Member> = FILES
      .iter()
      .map(|name| Member::file(name, name.as_bytes()))
      .collect();
    members.push(Member::file("../escape", b"extra"));
    fs::write(input.join(tag.archive(&TARGETS[0])), tarball(&members)?)?;
    let mut fake = Fake::default();
    assert_eq!(
      root_cause(prepare(
        &mut fake,
        &tag,
        &input,
        &output,
        None,
        MEMBER_LIMIT
      )),
      UNEXPECTED
    );
    assert!(fake.calls.is_empty());
    Ok(())
  }

  #[test]
  fn an_existing_output_is_never_reused() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    fs::create_dir(&output)?;
    let mut fake = Fake::default();
    assert!(prepare(&mut fake, &tag, &input, &output, None, MEMBER_LIMIT).is_err());
    assert!(fake.calls.is_empty());
    Ok(())
  }

  #[test]
  fn signing_refuses_anywhere_but_a_mac() -> Result<()> {
    let work = tempfile::tempdir()?;
    let (tag, input, output) = ci_input(work.path())?;
    let mut fake = Fake::default();
    let result = sign_macos(&mut fake, "linux", &tag, &input, &output, None);
    assert_eq!(
      result.err().map(|error| error.to_string()).as_deref(),
      Some(NOT_MAC)
    );
    assert!(fake.calls.is_empty() && !output.exists());
    Ok(())
  }
}
