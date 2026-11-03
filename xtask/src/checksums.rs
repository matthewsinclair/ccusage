//! SHA256SUMS: a `digest  name` line for each of a release's archives.

use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  path::Path,
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::release::{SUMS, TARGETS, Tag};

/// Each archive's name, and its SHA-256 as lowercase hex.
pub type Digests = BTreeMap<String, String>;

pub const INVALID: &str = "Invalid release checksum manifest";
pub const UNEXPECTED: &str = "Unexpected or missing release archives";
pub const MISMATCH: &str = "Release archive checksum mismatch";

/// Read SHA256SUMS's text: each line a digest and a name, no name twice, and the names
/// exactly the release's archives.
pub fn parse(text: &str, tag: &Tag) -> Result<Digests> {
  let mut digests = Digests::new();
  for line in text.lines() {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let [digest, name] = fields.as_slice() else {
      bail!(INVALID);
    };
    if !is_digest(digest)
      || digests
        .insert((*name).to_owned(), (*digest).to_owned())
        .is_some()
    {
      bail!(INVALID);
    }
  }
  let expected: BTreeSet<String> = tag.archives().collect();
  if !digests.keys().eq(expected.iter()) {
    bail!(UNEXPECTED);
  }
  Ok(digests)
}

/// Read SHA256SUMS in `directory` and hash each archive beside it against its line.
pub fn validate(directory: &Path, tag: &Tag) -> Result<Digests> {
  let path = directory.join(SUMS);
  let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
  let digests = parse(&text, tag)?;
  for (name, digest) in &digests {
    if file_digest(&directory.join(name))? != *digest {
      bail!(MISMATCH);
    }
  }
  Ok(digests)
}

/// SHA256SUMS's text for `digests`, in TARGETS order.
pub fn render(tag: &Tag, digests: &Digests) -> Result<String> {
  TARGETS
    .iter()
    .map(|target| {
      let name = tag.archive(target);
      let digest = digests
        .get(&name)
        .with_context(|| format!("no checksum for {name}"))?;
      Ok(format!("{digest}  {name}\n"))
    })
    .collect()
}

pub fn file_digest(path: &Path) -> Result<String> {
  let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
  Ok(digest(&bytes))
}

pub fn digest(bytes: &[u8]) -> String {
  Sha256::digest(bytes)
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect()
}

fn is_digest(text: &str) -> bool {
  text.len() == 2 * <Sha256 as Digest>::output_size()
    && text
      .bytes()
      .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::fake::{fixture_tag, write_assets};

  fn sums(tag: &Tag) -> String {
    tag
      .archives()
      .enumerate()
      .map(|(index, name)| {
        format!(
          "{}  {name}\n",
          digest(&[u8::try_from(index).unwrap_or_default()])
        )
      })
      .collect()
  }

  fn refusal(text: &str, tag: &Tag) -> String {
    match parse(text, tag) {
      Ok(_) => "parsed".to_owned(),
      Err(error) => error.to_string(),
    }
  }

  #[test]
  fn sums_parse_into_each_archive_digest() -> Result<()> {
    let tag = fixture_tag()?;
    let text = sums(&tag);
    let digests = parse(&text, &tag)?;
    assert_eq!(
      digests.keys().cloned().collect::<BTreeSet<_>>(),
      tag.archives().collect()
    );
    assert_eq!(render(&tag, &digests)?, text);
    Ok(())
  }

  #[test]
  fn lines_need_exactly_a_digest_and_a_name() -> Result<()> {
    let tag = fixture_tag()?;
    let text = sums(&tag);
    let first = text.lines().next().unwrap_or_default();
    for broken in [format!("{first} extra"), digest(b"alone"), String::new()] {
      let lines = text.replacen(first, &broken, 1);
      assert_eq!(refusal(&lines, &tag), INVALID, "{broken:?}");
    }
    Ok(())
  }

  #[test]
  fn digests_must_be_lowercase_sha256_hex() -> Result<()> {
    let tag = fixture_tag()?;
    let text = sums(&tag);
    let good = digest(&[0]);
    let mut short = good.clone();
    short.pop();
    for bad in [
      good.to_uppercase(),
      short,
      format!("{good}0"),
      good.replacen('0', "g", 1),
    ] {
      assert_eq!(
        refusal(&text.replacen(&good, &bad, 1), &tag),
        INVALID,
        "{bad}"
      );
    }
    Ok(())
  }

  #[test]
  fn names_must_not_repeat() -> Result<()> {
    let tag = fixture_tag()?;
    let text = sums(&tag);
    let first = text.lines().next().unwrap_or_default();
    assert_eq!(refusal(&format!("{text}{first}\n"), &tag), INVALID);
    Ok(())
  }

  #[test]
  fn names_must_be_exactly_the_release_archives() -> Result<()> {
    let tag = fixture_tag()?;
    let text = sums(&tag);
    let missing: String = text
      .lines()
      .skip(1)
      .map(|line| format!("{line}\n"))
      .collect();
    let extra = format!(
      "{text}{}  ccusage-{tag}-riscv64gc-unknown-linux-gnu.tar.gz\n",
      digest(b"extra")
    );
    let other = text.replace(tag.as_str(), "v9.9.9");
    for wrong in [missing, extra, other] {
      assert_eq!(refusal(&wrong, &tag), UNEXPECTED, "{wrong}");
    }
    Ok(())
  }

  #[test]
  fn archives_must_match_their_digests() -> Result<()> {
    let tag = fixture_tag()?;
    let work = tempfile::tempdir()?;
    let mut assets: BTreeMap<String, Vec<u8>> = tag
      .archives()
      .map(|name| (name.clone(), name.into_bytes()))
      .collect();
    let text: String = assets
      .iter()
      .map(|(name, bytes)| format!("{}  {name}\n", digest(bytes)))
      .collect();
    assets.insert(SUMS.into(), text.into_bytes());
    write_assets(work.path(), &assets)?;
    assert_eq!(validate(work.path(), &tag)?.len(), TARGETS.len());
    let tampered = tag.archive(&TARGETS[0]);
    fs::write(work.path().join(&tampered), b"tampered")?;
    let error = validate(work.path(), &tag)
      .err()
      .map(|error| error.to_string());
    assert_eq!(error.as_deref(), Some(MISMATCH));
    Ok(())
  }
}
