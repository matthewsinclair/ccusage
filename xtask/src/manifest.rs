//! MACOS-SIGNING.json: what binds each signed Mac archive to its checksum, and the release
//! to the Developer ID that signed it.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::{
  checksums::{self, Digests},
  release::{IDENTIFIER, MANIFEST, SUMS, TEAM, Tag, mac_targets},
};

pub const IDENTITY_MISMATCH: &str = "Release signing identity mismatch";
pub const CHECKSUM_MISMATCH: &str = "Signed archive checksum mismatch";

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
  pub tag: String,
  pub identifier: String,
  pub team_id: String,
  /// Each signed Mac archive, by name.
  pub archives: BTreeMap<String, Signed>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signed {
  /// The `designated => ...` line codesign printed for the signed binary.
  pub requirement: String,
  pub notarized: bool,
  pub sha256: String,
}

impl Manifest {
  pub fn read(directory: &Path) -> Result<Manifest> {
    let path = directory.join(MANIFEST);
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("reading {}", path.display()))
  }

  pub fn write(&self, directory: &Path) -> Result<()> {
    let mut text = serde_json::to_string_pretty(self)?;
    text.push('\n');
    fs::write(directory.join(MANIFEST), text)?;
    Ok(())
  }

  /// The Mac archives this manifest signs, in TARGETS order, once it has shown that it is
  /// this release's and the Geodica team's, and that each archive has the checksum
  /// SHA256SUMS gives it.
  pub fn mac_archives(&self, tag: &Tag, digests: &Digests) -> Result<Vec<(String, &Signed)>> {
    let identity = (
      self.tag.as_str(),
      self.identifier.as_str(),
      self.team_id.as_str(),
    );
    if identity != (tag.as_str(), IDENTIFIER, TEAM) {
      bail!(IDENTITY_MISMATCH);
    }
    mac_targets()
      .map(|target| {
        let name = tag.archive(target);
        let signed = self
          .archives
          .get(&name)
          .with_context(|| format!("{MANIFEST} does not sign {name}"))?;
        if digests.get(&name) != Some(&signed.sha256) {
          bail!(CHECKSUM_MISMATCH);
        }
        Ok((name, signed))
      })
      .collect()
  }
}

/// formula-checksums: the Mac archives' checksums, in TARGETS order, read from a published
/// release's SHA256SUMS once its signing manifest has shown they are the signed ones.
pub fn mac_checksums(tag: &Tag, directory: &Path) -> Result<Vec<String>> {
  let path = directory.join(SUMS);
  let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
  let digests = checksums::parse(&text, tag)?;
  let manifest = Manifest::read(directory)?;
  let archives = manifest.mac_archives(tag, &digests)?;
  Ok(
    archives
      .into_iter()
      .map(|(_, signed)| signed.sha256.clone())
      .collect(),
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    checksums::digest,
    fake::{fixture_tag, write_assets},
  };

  /// A release's SHA256SUMS and a manifest that agrees with it.
  fn release(tag: &Tag) -> (Digests, Manifest) {
    let digests: Digests = tag
      .archives()
      .map(|name| (name.clone(), digest(name.as_bytes())))
      .collect();
    let archives = mac_targets()
      .map(|target| {
        let name = tag.archive(target);
        let signed = Signed {
          requirement: format!("designated => fixture {}", target.triple),
          notarized: false,
          sha256: digests.get(&name).cloned().unwrap_or_default(),
        };
        (name, signed)
      })
      .collect();
    let manifest = Manifest {
      tag: tag.to_string(),
      identifier: IDENTIFIER.into(),
      team_id: TEAM.into(),
      archives,
    };
    (digests, manifest)
  }

  fn refusal(manifest: &Manifest, tag: &Tag, digests: &Digests) -> String {
    match manifest.mac_archives(tag, digests) {
      Ok(_) => "accepted".to_owned(),
      Err(error) => error.to_string(),
    }
  }

  #[test]
  fn a_matching_manifest_yields_the_mac_archives_in_target_order() -> Result<()> {
    let tag = fixture_tag()?;
    let (digests, manifest) = release(&tag);
    let names: Vec<String> = manifest
      .mac_archives(&tag, &digests)?
      .into_iter()
      .map(|(name, _)| name)
      .collect();
    assert_eq!(
      names,
      mac_targets()
        .map(|target| tag.archive(target))
        .collect::<Vec<_>>()
    );
    Ok(())
  }

  #[test]
  fn another_release_or_identity_is_refused() -> Result<()> {
    let tag = fixture_tag()?;
    let (digests, _) = release(&tag);
    let wrongs: &[fn(&mut Manifest)] = &[
      |manifest| manifest.tag = "v9.9.9".into(),
      |manifest| manifest.identifier = "com.example.ccusage".into(),
      |manifest| manifest.team_id = "0000000000".into(),
    ];
    for wrong in wrongs {
      let (_, mut manifest) = release(&tag);
      wrong(&mut manifest);
      assert_eq!(
        refusal(&manifest, &tag, &digests),
        IDENTITY_MISMATCH,
        "{manifest:?}"
      );
    }
    Ok(())
  }

  #[test]
  fn a_signed_checksum_must_match_sha256sums() -> Result<()> {
    let tag = fixture_tag()?;
    for target in mac_targets() {
      let (mut digests, manifest) = release(&tag);
      digests.insert(tag.archive(target), digest(b"other bytes"));
      assert_eq!(
        refusal(&manifest, &tag, &digests),
        CHECKSUM_MISMATCH,
        "{}",
        target.triple
      );
    }
    Ok(())
  }

  #[test]
  fn every_mac_archive_must_be_signed() -> Result<()> {
    let tag = fixture_tag()?;
    for target in mac_targets() {
      let (digests, mut manifest) = release(&tag);
      manifest.archives.remove(&tag.archive(target));
      assert!(
        refusal(&manifest, &tag, &digests).contains("does not sign"),
        "{}",
        target.triple
      );
    }
    Ok(())
  }

  #[test]
  fn manifests_read_as_the_python_tooling_wrote_them() -> Result<()> {
    let tag = fixture_tag()?;
    let (_, manifest) = release(&tag);
    let work = tempfile::tempdir()?;
    manifest.write(work.path())?;
    assert_eq!(Manifest::read(work.path())?, manifest);
    // json.dumps(manifest, indent=2) plus a newline, as tools/sign-macos wrote it.
    let arm = format!("ccusage-{tag}-aarch64-apple-darwin.tar.gz");
    let intel = format!("ccusage-{tag}-x86_64-apple-darwin.tar.gz");
    let sha = |name: &str| {
      manifest
        .archives
        .get(name)
        .map(|signed| signed.sha256.clone())
        .unwrap_or_default()
    };
    let expected = format!(
      "{{\n  \"tag\": \"{tag}\",\n  \"identifier\": \"{IDENTIFIER}\",\n  \"team_id\": \"{TEAM}\",\n  \"archives\": {{\n    \"{arm}\": {{\n      \"requirement\": \"designated => fixture aarch64-apple-darwin\",\n      \"notarized\": false,\n      \"sha256\": \"{}\"\n    }},\n    \"{intel}\": {{\n      \"requirement\": \"designated => fixture x86_64-apple-darwin\",\n      \"notarized\": false,\n      \"sha256\": \"{}\"\n    }}\n  }}\n}}\n",
      sha(&arm),
      sha(&intel)
    );
    assert_eq!(fs::read_to_string(work.path().join(MANIFEST))?, expected);
    Ok(())
  }

  #[test]
  fn formula_checksums_are_the_mac_digests_apple_silicon_first() -> Result<()> {
    let tag = fixture_tag()?;
    let (digests, manifest) = release(&tag);
    let work = tempfile::tempdir()?;
    let sums = checksums::render(&tag, &digests)?;
    write_assets(
      work.path(),
      &BTreeMap::from([(SUMS.to_owned(), sums.into_bytes())]),
    )?;
    manifest.write(work.path())?;
    // tools/formula pins the first line to Apple silicon and the second to Intel.
    let expected: Vec<String> = ["aarch64-apple-darwin", "x86_64-apple-darwin"]
      .iter()
      .map(|triple| {
        digests
          .get(&format!("ccusage-{tag}-{triple}.tar.gz"))
          .cloned()
          .unwrap_or_default()
      })
      .collect();
    assert_eq!(mac_checksums(&tag, work.path())?, expected);
    Ok(())
  }

  #[test]
  fn formula_checksums_refuse_a_manifest_that_disagrees() -> Result<()> {
    let tag = fixture_tag()?;
    let (digests, mut manifest) = release(&tag);
    manifest.team_id = "0000000000".into();
    let work = tempfile::tempdir()?;
    let sums = checksums::render(&tag, &digests)?;
    write_assets(
      work.path(),
      &BTreeMap::from([(SUMS.to_owned(), sums.into_bytes())]),
    )?;
    manifest.write(work.path())?;
    let error = mac_checksums(&tag, work.path())
      .err()
      .map(|error| error.to_string());
    assert_eq!(error.as_deref(), Some(IDENTITY_MISMATCH));
    Ok(())
  }
}
