//! Release archives: every member checked before one is unpacked to sign or verify, and
//! the signed files repacked with deterministic headers.

use std::{
  collections::BTreeSet,
  fs::{self, File, OpenOptions, Permissions},
  io,
  os::unix::fs::PermissionsExt,
  path::Path,
};

use anyhow::{Context, Result, bail};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use tar::{Archive, Builder, EntryType, Header};

use crate::release::{BINARY, FILES};

pub const UNEXPECTED: &str = "Unexpected release archive member";
pub const INCOMPLETE: &str = "Release archive is missing expected files";

/// Unpack `archive` into the empty `directory`. The members must be exactly FILES, each
/// once, each a regular file of at most `limit` bytes, named bare or under `./`; the one
/// other member allowed is that `./` root directory. The binary comes out executable.
pub fn extract(archive: &Path, directory: &Path, limit: u64) -> Result<()> {
  let file = File::open(archive).with_context(|| format!("opening {}", archive.display()))?;
  let mut tar = Archive::new(GzDecoder::new(file));
  let mut seen = BTreeSet::new();
  for entry in tar.entries()? {
    let mut entry = entry?;
    let path = entry.path_bytes().into_owned();
    let name = path.strip_prefix(b"./").unwrap_or(&path);
    let kind = entry.header().entry_type();
    if kind.is_dir() && matches!(name, b"" | b".") {
      continue;
    }
    let Some(file) = FILES.iter().copied().find(|file| file.as_bytes() == name) else {
      bail!(UNEXPECTED);
    };
    if kind != EntryType::Regular || entry.size() > limit || !seen.insert(file) {
      bail!(UNEXPECTED);
    }
    let mut output = OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(directory.join(file))?;
    io::copy(&mut entry, &mut output)?;
  }
  if seen.len() != FILES.len() {
    bail!(INCOMPLETE);
  }
  fs::set_permissions(directory.join(BINARY), Permissions::from_mode(0o755))?;
  Ok(())
}

/// Pack FILES from `directory` into the new archive `archive`, in FILES order, each header
/// deterministic: no owner, no timestamp, and only the binary executable.
pub fn pack(directory: &Path, archive: &Path) -> Result<()> {
  let file = OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(archive)
    .with_context(|| format!("creating {}", archive.display()))?;
  let mut tar = Builder::new(GzEncoder::new(file, Compression::default()));
  for name in FILES {
    let data = fs::read(directory.join(name))?;
    let mut header = Header::new_ustar();
    header.set_entry_type(EntryType::Regular);
    header.set_size(u64::try_from(data.len())?);
    header.set_mode(if *name == BINARY { 0o755 } else { 0o644 });
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    tar.append_data(&mut header, name, data.as_slice())?;
  }
  tar.into_inner()?.finish()?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    fake::{Member, ci_archive, tarball},
    release::{MEMBER_LIMIT, TARGETS},
  };

  fn unpack(members: &[Member], limit: u64) -> Result<tempfile::TempDir> {
    let work = tempfile::tempdir()?;
    let archive = work.path().join("fixture.tar.gz");
    fs::write(&archive, tarball(members)?)?;
    let directory = work.path().join("out");
    fs::create_dir(&directory)?;
    extract(&archive, &directory, limit)?;
    Ok(work)
  }

  fn refusal(members: &[Member], limit: u64) -> String {
    match unpack(members, limit) {
      Ok(_) => "unpacked".to_owned(),
      Err(error) => error.to_string(),
    }
  }

  fn files() -> Vec<Member> {
    FILES
      .iter()
      .map(|name| Member::file(name, name.as_bytes()))
      .collect()
  }

  #[test]
  fn ci_archives_unpack_with_an_executable_binary() -> Result<()> {
    let work = tempfile::tempdir()?;
    let archive = work.path().join("ci.tar.gz");
    fs::write(&archive, ci_archive(&TARGETS[0])?)?;
    let directory = work.path().join("out");
    fs::create_dir(&directory)?;
    extract(&archive, &directory, MEMBER_LIMIT)?;
    let mut unpacked: Vec<String> = fs::read_dir(&directory)?
      .map(|entry| Ok(entry?.file_name().to_string_lossy().into_owned()))
      .collect::<Result<_>>()?;
    unpacked.sort();
    assert_eq!(unpacked, FILES);
    assert_eq!(
      fs::read(directory.join(BINARY))?,
      TARGETS[0].triple.as_bytes()
    );
    assert_eq!(
      fs::metadata(directory.join(BINARY))?.permissions().mode() & 0o777,
      0o755
    );
    Ok(())
  }

  #[test]
  fn bare_names_unpack_too() -> Result<()> {
    let work = unpack(&files(), MEMBER_LIMIT)?;
    assert_eq!(
      fs::read(work.path().join("out").join("LICENSE"))?,
      b"LICENSE"
    );
    Ok(())
  }

  #[test]
  fn traversal_and_unknown_names_are_refused() {
    // Each name stands in for LICENSE, so a check that read only the last component,
    // or matched case-insensitively, would find a complete archive.
    for name in [
      "../escape",
      "../LICENSE",
      "./../LICENSE",
      "/LICENSE",
      "sub/LICENSE",
      "./sub/",
      "././LICENSE",
      "license",
    ] {
      let mut members: Vec<Member> = files()
        .into_iter()
        .filter(|member| member.name != "LICENSE")
        .collect();
      members.push(Member::file(name, b"LICENSE"));
      assert_eq!(refusal(&members, MEMBER_LIMIT), UNEXPECTED, "{name}");
    }
  }

  #[test]
  fn members_other_than_regular_files_are_refused() {
    for kind in [
      EntryType::Symlink,
      EntryType::Link,
      EntryType::Directory,
      EntryType::Fifo,
      EntryType::GNUSparse,
    ] {
      let mut members: Vec<Member> = files()
        .into_iter()
        .filter(|member| member.name != BINARY)
        .collect();
      members.push(Member {
        name: BINARY.into(),
        kind,
        data: Vec::new(),
      });
      assert_eq!(refusal(&members, MEMBER_LIMIT), UNEXPECTED, "{kind:?}");
    }
  }

  #[test]
  fn pax_global_headers_are_refused() {
    let mut members = files();
    members.insert(
      0,
      Member {
        name: "pax_global_header".into(),
        kind: EntryType::XGlobalHeader,
        data: b"26 comment=ccusage fixture\n".to_vec(),
      },
    );
    assert_eq!(refusal(&members, MEMBER_LIMIT), UNEXPECTED);
  }

  #[test]
  fn duplicate_members_are_refused() {
    let mut members = files();
    members.push(Member::file("./ccusage", b"second"));
    assert_eq!(refusal(&members, MEMBER_LIMIT), UNEXPECTED);
  }

  #[test]
  fn members_over_the_limit_are_refused() -> Result<()> {
    let largest = FILES
      .iter()
      .map(|name| name.len())
      .max()
      .unwrap_or_default();
    let limit = u64::try_from(largest)?;
    assert!(unpack(&files(), limit).is_ok());
    assert_eq!(refusal(&files(), limit - 1), UNEXPECTED);
    Ok(())
  }

  #[test]
  fn missing_files_are_refused() {
    for missing in FILES {
      let members: Vec<Member> = files()
        .into_iter()
        .filter(|member| member.name != *missing)
        .collect();
      assert_eq!(refusal(&members, MEMBER_LIMIT), INCOMPLETE, "{missing}");
    }
  }

  #[test]
  fn packs_are_deterministic_and_only_the_binary_is_executable() -> Result<()> {
    let work = unpack(&files(), MEMBER_LIMIT)?;
    let directory = work.path().join("out");
    let (first, second) = (
      work.path().join("first.tar.gz"),
      work.path().join("second.tar.gz"),
    );
    pack(&directory, &first)?;
    pack(&directory, &second)?;
    assert_eq!(fs::read(&first)?, fs::read(&second)?);
    let mut tar = Archive::new(GzDecoder::new(File::open(&first)?));
    let mut names = Vec::new();
    for entry in tar.entries()? {
      let entry = entry?;
      let header = entry.header();
      let name = String::from_utf8(entry.path_bytes().into_owned())?;
      let mode = if name == BINARY { 0o755 } else { 0o644 };
      assert_eq!(header.entry_type(), EntryType::Regular, "{name}");
      assert_eq!(header.mode()?, mode, "{name}");
      assert_eq!(
        (header.mtime()?, header.uid()?, header.gid()?),
        (0, 0, 0),
        "{name}"
      );
      assert_eq!(header.username()?, Some(""), "{name}");
      names.push(name);
    }
    assert_eq!(names, FILES);
    let unpacked = work.path().join("again");
    fs::create_dir(&unpacked)?;
    extract(&first, &unpacked, MEMBER_LIMIT)?;
    for name in FILES {
      assert_eq!(
        fs::read(unpacked.join(name))?,
        fs::read(directory.join(name))?,
        "{name}"
      );
    }
    Ok(())
  }
}
