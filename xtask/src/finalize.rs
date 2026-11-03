//! finalize-release: sign CI's draft on the maintainer's Mac, and publish it only once the
//! assets GitHub holds have been downloaded again and verified.

use std::{
  fs, io,
  path::{Path, PathBuf},
};

use anyhow::{Result, bail};

use crate::{
  archive,
  checksums::{self, Digests},
  manifest::Manifest,
  release::{BINARY, MANIFEST, MEMBER_LIMIT, REPO, SUMS, Tag},
  runner::{Invocation, Output, Runner},
  sign,
};

pub const NOT_MAC: &str = "Finalize releases on the maintainer's Mac; signing keys stay local";
pub const DRAFT_UNKNOWN: &str = "Could not determine draft status";
pub const REQUIREMENT_MISMATCH: &str = "Release designated requirement mismatch";

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
  /// The release was already public, and its Mac signatures verified.
  Verified,
  /// The draft was signed, uploaded, verified as GitHub holds it, and published.
  Published,
}

pub fn finalize(
  runner: &mut dyn Runner,
  os: &str,
  tag: &Tag,
  notary: Option<&str>,
) -> Result<Outcome> {
  sign::require_mac(os, NOT_MAC)?;
  let view = gh(tag, "view").args(["--json", "isDraft", "--jq", ".isDraft"]);
  let draft = match runner.run(&view, Output::Stdout)?.stdout.trim() {
    "true" => true,
    "false" => false,
    _ => bail!(DRAFT_UNKNOWN),
  };
  let root = tempfile::Builder::new()
    .prefix("ccusage-release-assets-")
    .tempdir()?;
  let incoming = root.path().join("incoming");
  fs::create_dir(&incoming)?;
  let archives = format!("ccusage-{tag}-*.tar.gz");
  let mut patterns = vec![archives.as_str(), SUMS];
  if !draft {
    patterns.push(MANIFEST);
  }
  let download = patterns.into_iter().fold(
    gh(tag, "download").arg("--dir").arg(&incoming),
    |download, pattern| download.args(["--pattern", pattern]),
  );
  runner.run(&download, Output::Shown)?;
  let digests = checksums::validate(&incoming, tag)?;
  if !draft {
    verify_signatures(runner, &incoming, tag, &digests)?;
    return Ok(Outcome::Verified);
  }
  let signed = root.path().join("signed");
  sign::prepare(runner, tag, &incoming, &signed, notary, MEMBER_LIMIT)?;
  let digests = checksums::validate(&signed, tag)?;
  verify_signatures(runner, &signed, tag, &digests)?;
  let mut files = fs::read_dir(&signed)?
    .map(|entry| entry.map(|entry| entry.path()))
    .collect::<io::Result<Vec<PathBuf>>>()?;
  files.sort();
  runner.run(
    &gh(tag, "upload").arg("--clobber").args(&files),
    Output::Shown,
  )?;
  // What GitHub now holds is verified before anything is published, on a resumed run too.
  let uploaded = root.path().join("uploaded");
  runner.run(
    &gh(tag, "download").arg("--dir").arg(&uploaded),
    Output::Shown,
  )?;
  let digests = checksums::validate(&uploaded, tag)?;
  verify_signatures(runner, &uploaded, tag, &digests)?;
  runner.run(&gh(tag, "edit").arg("--draft=false"), Output::Shown)?;
  Ok(Outcome::Published)
}

/// `gh release <verb> <tag> --repo <repo>`.
fn gh(tag: &Tag, verb: &str) -> Invocation {
  Invocation::new("gh").args(["release", verb, tag.as_str(), "--repo", REPO])
}

/// Each Mac archive in `directory` is the one MACOS-SIGNING.json signs, and its binary
/// still verifies with the designated requirement the manifest recorded for it.
fn verify_signatures(
  runner: &mut dyn Runner,
  directory: &Path,
  tag: &Tag,
  digests: &Digests,
) -> Result<()> {
  let manifest = Manifest::read(directory)?;
  for (name, signed) in manifest.mac_archives(tag, digests)? {
    let work = tempfile::Builder::new()
      .prefix("ccusage-verify-")
      .tempdir()?;
    archive::extract(&directory.join(&name), work.path(), MEMBER_LIMIT)?;
    if sign::verify(runner, &work.path().join(BINARY))? != signed.requirement {
      bail!(REQUIREMENT_MISMATCH);
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    checksums::MISMATCH,
    fake::{Fake, ci_release, fixture_tag, signed_display, write_assets},
    manifest::IDENTITY_MISMATCH,
    runner::Captured,
    sign::CODESIGN,
  };

  /// A Mac whose GitHub holds CI's draft for the fixture tag.
  fn draft() -> Result<(Tag, Fake)> {
    let tag = fixture_tag()?;
    let fake = Fake {
      assets: ci_release(&tag)?,
      ..Fake::default()
    };
    Ok((tag, fake))
  }

  /// A Mac whose GitHub holds the fixture tag already signed and published.
  fn published() -> Result<(Tag, Fake)> {
    let (tag, mut fake) = draft()?;
    finalize(&mut fake, "macos", &tag, None)?;
    fake.draft = "false";
    fake.calls.clear();
    Ok((tag, fake))
  }

  fn position(fake: &Fake, prefix: &[&str]) -> Option<usize> {
    fake.calls.iter().position(|call| {
      call
        .iter()
        .zip(prefix)
        .all(|(word, expected)| word == expected)
        && call.len() >= prefix.len()
    })
  }

  fn root_cause(result: Result<Outcome>) -> String {
    result
      .err()
      .map(|error| error.root_cause().to_string())
      .unwrap_or_default()
  }

  #[test]
  fn a_draft_is_signed_uploaded_reverified_then_published() -> Result<()> {
    let (tag, mut fake) = draft()?;
    assert_eq!(
      finalize(&mut fake, "macos", &tag, None)?,
      Outcome::Published
    );
    let upload = position(&fake, &["gh", "release", "upload"]);
    let edit = position(&fake, &["gh", "release", "edit"]);
    let downloads: Vec<usize> = fake
      .calls
      .iter()
      .enumerate()
      .filter(|(_, call)| {
        call.starts_with(&["gh".to_owned(), "release".to_owned(), "download".to_owned()])
      })
      .map(|(index, _)| index)
      .collect();
    assert!(
      upload.is_some() && downloads.last().copied() > upload,
      "{:?}",
      fake.calls
    );
    assert_eq!(edit, Some(fake.calls.len() - 1), "{:?}", fake.calls);
    let last_download = downloads.last().copied().unwrap_or_default();
    let verified_after = fake
      .calls
      .iter()
      .skip(last_download)
      .filter(|call| call.get(1).is_some_and(|word| word == "--verify"));
    assert!(
      verified_after.count() > 0,
      "the uploaded assets were not verified"
    );
    let work = tempfile::tempdir()?;
    write_assets(work.path(), &fake.assets)?;
    let digests = checksums::validate(work.path(), &tag)?;
    Manifest::read(work.path())?.mac_archives(&tag, &digests)?;
    Ok(())
  }

  #[test]
  fn a_published_release_is_verified_and_left_alone() -> Result<()> {
    let (tag, mut fake) = published()?;
    let before = fake.assets.clone();
    assert_eq!(finalize(&mut fake, "macos", &tag, None)?, Outcome::Verified);
    let download =
      position(&fake, &["gh", "release", "download"]).and_then(|index| fake.calls.get(index));
    assert!(
      download.is_some_and(|call| call.iter().any(|word| word == MANIFEST)),
      "{download:?}"
    );
    assert_eq!(position(&fake, &["gh", "release", "upload"]), None);
    assert_eq!(position(&fake, &["gh", "release", "edit"]), None);
    assert_eq!(fake.ran(&[CODESIGN, "--force"]), 0);
    assert_eq!(fake.assets, before);
    Ok(())
  }

  #[test]
  fn a_published_release_with_another_identity_is_refused() -> Result<()> {
    let (tag, mut fake) = published()?;
    let manifest = fake.assets.get(MANIFEST).cloned().unwrap_or_default();
    let changed = String::from_utf8(manifest)?.replace("76BQL8L47U", "0000000000");
    fake.assets.insert(MANIFEST.into(), changed.into_bytes());
    assert_eq!(
      root_cause(finalize(&mut fake, "macos", &tag, None)),
      IDENTITY_MISMATCH
    );
    Ok(())
  }

  #[test]
  fn a_signature_that_no_longer_matches_the_manifest_is_refused() -> Result<()> {
    let (tag, mut fake) = published()?;
    let display = signed_display();
    let other = display.stdout.replace("76BQL8L47U", "0000000000");
    fake.displays = vec![Captured {
      stdout: other,
      ..display
    }];
    assert_eq!(
      root_cause(finalize(&mut fake, "macos", &tag, None)),
      REQUIREMENT_MISMATCH
    );
    Ok(())
  }

  #[test]
  fn a_draft_is_not_published_when_signing_fails() -> Result<()> {
    let (tag, mut fake) = draft()?;
    fake.failing = Some("/usr/bin/codesign --force");
    assert!(finalize(&mut fake, "macos", &tag, None).is_err());
    assert_eq!(position(&fake, &["gh", "release", "upload"]), None);
    assert_eq!(position(&fake, &["gh", "release", "edit"]), None);
    Ok(())
  }

  #[test]
  fn a_draft_whose_uploaded_bytes_changed_is_not_published() -> Result<()> {
    let (tag, mut fake) = draft()?;
    fake.after_upload = |assets| {
      if let Some(bytes) = assets
        .iter_mut()
        .find(|(name, _)| name.ends_with(".tar.gz"))
        .map(|(_, bytes)| bytes)
      {
        bytes.push(0);
      }
    };
    assert_eq!(
      root_cause(finalize(&mut fake, "macos", &tag, None)),
      MISMATCH
    );
    assert!(position(&fake, &["gh", "release", "upload"]).is_some());
    assert_eq!(position(&fake, &["gh", "release", "edit"]), None);
    Ok(())
  }

  #[test]
  fn a_draft_that_does_not_match_its_checksums_is_never_signed() -> Result<()> {
    let (tag, mut fake) = draft()?;
    let archive = fake
      .assets
      .iter_mut()
      .find(|(name, _)| name.ends_with(".tar.gz"));
    if let Some((_, bytes)) = archive {
      bytes.push(0);
    }
    assert_eq!(
      root_cause(finalize(&mut fake, "macos", &tag, None)),
      MISMATCH
    );
    assert_eq!(fake.ran(&[CODESIGN]), 0);
    Ok(())
  }

  #[test]
  fn an_unreadable_draft_status_stops_before_any_download() -> Result<()> {
    let (tag, mut fake) = draft()?;
    fake.draft = "maybe";
    assert_eq!(
      root_cause(finalize(&mut fake, "macos", &tag, None)),
      DRAFT_UNKNOWN
    );
    assert_eq!(position(&fake, &["gh", "release", "download"]), None);
    Ok(())
  }

  #[test]
  fn finalizing_refuses_anywhere_but_a_mac() -> Result<()> {
    let (tag, mut fake) = draft()?;
    assert_eq!(
      root_cause(finalize(&mut fake, "linux", &tag, None)),
      NOT_MAC
    );
    assert!(fake.calls.is_empty());
    Ok(())
  }
}
