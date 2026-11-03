//! The tests' stand-in Mac and GitHub. The fake codesign signs by appending to the binary,
//! the fake lipo answers as lipo does for each target's binary, and the release's assets
//! live in memory.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Result, anyhow, bail};
use flate2::{Compression, write::GzEncoder};
use tar::{Builder, EntryType, Header};

use crate::{
  checksums::digest,
  release::{BINARY, FILES, REPO, SUMS, TARGETS, Tag, Target},
  runner::{Captured, Invocation, Output, Runner},
  sign::{CODESIGN, DITTO, LIPO, XCRUN},
};

/// What `codesign -d --verbose=4 -r-` printed for the v0.1.5 release binary: the designated
/// requirement on stdout, the signature's details on stderr.
pub const DESIGNATED: &str = include_str!("../fixtures/codesign-v0.1.5.stdout");
const DETAILS: &str = include_str!("../fixtures/codesign-v0.1.5.stderr");

/// What the fake codesign appends to a binary it signs.
const SIGNATURE: &[u8] = b"\nsigned by the fake codesign";

pub fn signed_display() -> Captured {
  Captured {
    stdout: DESIGNATED.into(),
    stderr: DETAILS.into(),
  }
}

pub fn fixture_tag() -> Result<Tag> {
  Tag::parse("v0.1.4").ok_or_else(|| anyhow!("the fixture tag is not a stable tag"))
}

pub struct Fake {
  /// Each command run, as its words.
  pub calls: Vec<Vec<String>>,
  /// What each `codesign -d` prints, in turn; the last one repeats.
  pub displays: Vec<Captured>,
  /// What `lipo -archs` reports instead of the binary's own architecture.
  pub arch: Option<&'static str>,
  /// notarytool's answer.
  pub notary: &'static str,
  /// The first command whose line starts with this fails.
  pub failing: Option<&'static str>,
  /// What `gh release view --json isDraft` prints.
  pub draft: &'static str,
  /// The release's assets.
  pub assets: BTreeMap<String, Vec<u8>>,
  /// Applied to the assets after an upload: bytes that change on their way to GitHub.
  pub after_upload: fn(&mut BTreeMap<String, Vec<u8>>),
  /// How many `codesign -d` calls have been answered.
  pub displayed: usize,
}

impl Default for Fake {
  fn default() -> Fake {
    Fake {
      calls: Vec::new(),
      displays: vec![signed_display()],
      arch: None,
      notary: r#"{"id":"fixture","status":"Accepted"}"#,
      failing: None,
      draft: "true",
      assets: BTreeMap::new(),
      after_upload: |_| {},
      displayed: 0,
    }
  }
}

impl Fake {
  /// What the fake codesign makes of `binary`.
  pub fn signed(binary: &[u8]) -> Vec<u8> {
    [binary, SIGNATURE].concat()
  }

  /// How many of the calls began with `prefix`.
  pub fn ran(&self, prefix: &[&str]) -> usize {
    let begins = |call: &&Vec<String>| {
      call.len() >= prefix.len()
        && call
          .iter()
          .zip(prefix)
          .all(|(word, expected)| word == expected)
    };
    self.calls.iter().filter(begins).count()
  }

  fn lipo(&self, binary: &str) -> Result<String> {
    let bytes = fs::read(binary)?;
    // What lipo -archs prints for each Mac target's binary.
    let arch = if bytes.starts_with(b"aarch64-apple-darwin") {
      "arm64"
    } else if bytes.starts_with(b"x86_64-apple-darwin") {
      "x86_64"
    } else {
      bail!("lipo: not a Mac binary: {binary}");
    };
    Ok(format!("{}\n", self.arch.unwrap_or(arch)))
  }

  fn download(&self, directory: &str, options: &[&str]) -> Result<()> {
    let patterns = options
      .chunks(2)
      .map(|option| match option {
        ["--pattern", pattern] => Ok(*pattern),
        _ => Err(anyhow!("the fake gh does not download with {option:?}")),
      })
      .collect::<Result<Vec<&str>>>()?;
    fs::create_dir_all(directory)?;
    for (name, bytes) in &self.assets {
      if patterns.is_empty() || patterns.iter().any(|pattern| matches(pattern, name)) {
        fs::write(Path::new(directory).join(name), bytes)?;
      }
    }
    Ok(())
  }

  fn upload(&mut self, files: &[&str]) -> Result<()> {
    for file in files {
      let path = Path::new(file);
      let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("no file name: {file}"))?;
      self.assets.insert(name.to_owned(), fs::read(path)?);
    }
    (self.after_upload)(&mut self.assets);
    Ok(())
  }
}

impl Runner for Fake {
  fn run(&mut self, invocation: &Invocation, _output: Output) -> Result<Captured> {
    let words = invocation.words();
    self.calls.push(words.clone());
    let line = words.join(" ");
    if self.failing.is_some_and(|prefix| line.starts_with(prefix)) {
      bail!("{line} failed: exit status: 1");
    }
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    let stdout = match words.as_slice() {
      [LIPO, "-archs", binary] => self.lipo(binary)?,
      [CODESIGN, "--force", .., binary] => {
        let bytes = fs::read(binary)?;
        fs::write(binary, Fake::signed(&bytes))?;
        String::new()
      }
      [CODESIGN, "-d", ..] => {
        let display = self
          .displays
          .get(self.displayed)
          .or(self.displays.last())
          .cloned();
        self.displayed += 1;
        return Ok(display.unwrap_or_default());
      }
      [CODESIGN, "--verify", ..] | [DITTO, ..] | ["gh", "release", "edit", ..] => String::new(),
      [XCRUN, "notarytool", ..] => self.notary.to_owned(),
      ["gh", "release", "view", ..] => format!("{}\n", self.draft),
      [
        "gh",
        "release",
        "download",
        _,
        "--repo",
        REPO,
        "--dir",
        directory,
        options @ ..,
      ] => {
        self.download(directory, options)?;
        String::new()
      }
      [
        "gh",
        "release",
        "upload",
        _,
        "--repo",
        REPO,
        "--clobber",
        files @ ..,
      ] => {
        self.upload(files)?;
        String::new()
      }
      _ => bail!("the fake has no answer for {line}"),
    };
    Ok(Captured {
      stdout,
      stderr: String::new(),
    })
  }
}

/// gh's `--pattern`, here a glob with at most one `*`.
fn matches(pattern: &str, name: &str) -> bool {
  match pattern.split_once('*') {
    Some((prefix, suffix)) => {
      name.len() >= prefix.len() + suffix.len()
        && name.starts_with(prefix)
        && name.ends_with(suffix)
    }
    None => pattern == name,
  }
}

/// A member of a test archive. Its name is written as given, so a test can write what
/// the tar crate's own path checks refuse to.
pub struct Member {
  pub name: String,
  pub kind: EntryType,
  pub data: Vec<u8>,
}

impl Member {
  pub fn file(name: &str, data: &[u8]) -> Member {
    Member {
      name: name.into(),
      kind: EntryType::Regular,
      data: data.to_vec(),
    }
  }
}

/// A gzipped tar of `members`, in order.
pub fn tarball(members: &[Member]) -> Result<Vec<u8>> {
  let mut tar = Builder::new(GzEncoder::new(Vec::new(), Compression::default()));
  for member in members {
    let mut header = Header::new_gnu();
    let name = &mut header.as_old_mut().name;
    if member.name.len() > name.len() {
      bail!("fixture name too long: {}", member.name);
    }
    name
      .iter_mut()
      .zip(member.name.bytes())
      .for_each(|(slot, byte)| *slot = byte);
    // A well-formed sparse member has a real size, here octal 0 for one with no data, so it
    // reaches the extractor's own check instead of failing in the tar crate's parser.
    if member.kind == EntryType::GNUSparse
      && let Some(gnu) = header.as_gnu_mut()
    {
      gnu.realsize = *b"00000000000\0";
    }
    header.set_entry_type(member.kind);
    header.set_size(u64::try_from(member.data.len())?);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append(&header, member.data.as_slice())?;
  }
  Ok(tar.into_inner()?.finish()?)
}

/// What CI uploads for `target`: `tar -czf ... -C staging .` writes a `./` root directory
/// and each file under it. The binary holds the target's triple.
pub fn ci_archive(target: &Target) -> Result<Vec<u8>> {
  let root = Member {
    name: "./".into(),
    kind: EntryType::Directory,
    data: Vec::new(),
  };
  let files = FILES.iter().map(|name| {
    let data = if *name == BINARY {
      target.triple.as_bytes()
    } else {
      name.as_bytes()
    };
    Member::file(&format!("./{name}"), data)
  });
  tarball(&std::iter::once(root).chain(files).collect::<Vec<_>>())
}

/// CI's draft for `tag`: an archive for each target, and SHA256SUMS.
pub fn ci_release(tag: &Tag) -> Result<BTreeMap<String, Vec<u8>>> {
  let mut assets = BTreeMap::new();
  let mut sums = String::new();
  for target in TARGETS {
    let name = tag.archive(target);
    let bytes = ci_archive(target)?;
    sums.push_str(&format!("{}  {name}\n", digest(&bytes)));
    assets.insert(name, bytes);
  }
  assets.insert(SUMS.to_owned(), sums.into_bytes());
  Ok(assets)
}

pub fn write_assets(directory: &Path, assets: &BTreeMap<String, Vec<u8>>) -> Result<()> {
  for (name, bytes) in assets {
    fs::write(directory.join(name), bytes)?;
  }
  Ok(())
}
