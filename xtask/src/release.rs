//! What a Ccusage release is: a stable tag, an archive for each of four targets, and the
//! Geodica Developer ID that signs the two Mac ones.

use std::fmt;

// The team ID and the signing identifier are each written once: the identity and the
// requirement below are built from them.
macro_rules! team {
  () => {
    "76BQL8L47U"
  };
}
macro_rules! identifier {
  () => {
    "com.geodica.ccusage"
  };
}

pub const REPO: &str = "matthewsinclair/ccusage";
pub const TEAM: &str = team!();
pub const IDENTIFIER: &str = identifier!();
/// The Keychain identity codesign signs with. Its private key never leaves the maintainer's Mac.
pub const IDENTITY: &str = concat!("Developer ID Application: Geodica Pty Ltd (", team!(), ")");
/// What a signed binary must satisfy: Apple-anchored, this identifier and team, and a
/// Developer ID leaf certificate. packaging/homebrew/ccusage.rb checks the same at install.
pub const REQUIREMENT: &str = concat!(
  "identifier \"",
  identifier!(),
  "\" and anchor apple generic and certificate leaf[subject.OU] = \"",
  team!(),
  "\" and certificate leaf[field.1.2.840.113635.100.6.1.13] exists"
);

/// A `digest  name` line for each archive.
pub const SUMS: &str = "SHA256SUMS";
/// Binds the signed Mac archives to their checksums and to the identity that signed them.
pub const MANIFEST: &str = "MACOS-SIGNING.json";
/// The executable every archive holds.
pub const BINARY: &str = "ccusage";
/// Every archive holds these files and nothing else; a repack writes them in this order.
pub const FILES: &[&str] = &["LICENSE", "README.md", BINARY, "ccusage-logo.svg"];
/// The largest archive member signing or verification will unpack.
pub const MEMBER_LIMIT: u64 = 128 * 1024 * 1024;

pub struct Target {
  pub triple: &'static str,
  /// What `lipo -archs` reports for a Mac target's binary; a Linux target has none.
  pub mac_arch: Option<&'static str>,
}

/// The release's targets in SHA256SUMS order: the Macs first, Apple silicon before Intel,
/// which is also the order tools/formula reads the Mac checksums in.
pub const TARGETS: &[Target] = &[
  Target {
    triple: "aarch64-apple-darwin",
    mac_arch: Some("arm64"),
  },
  Target {
    triple: "x86_64-apple-darwin",
    mac_arch: Some("x86_64"),
  },
  Target {
    triple: "aarch64-unknown-linux-gnu",
    mac_arch: None,
  },
  Target {
    triple: "x86_64-unknown-linux-gnu",
    mac_arch: None,
  },
];

/// The Mac targets, in TARGETS order.
pub fn mac_targets() -> impl Iterator<Item = &'static Target> {
  TARGETS.iter().filter(|target| target.mac_arch.is_some())
}

/// A stable release tag: `v`, then three dot-separated numbers without leading zeros.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag(String);

impl Tag {
  pub fn parse(text: &str) -> Option<Tag> {
    let numbers: Vec<&str> = text.strip_prefix('v')?.split('.').collect();
    let stable = numbers.len() == 3 && numbers.iter().all(|number| is_number(number));
    stable.then(|| Tag(text.to_owned()))
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// The archive CI builds for `target`.
  pub fn archive(&self, target: &Target) -> String {
    format!("ccusage-{}-{}.tar.gz", self.0, target.triple)
  }

  /// Every archive of the release, in TARGETS order.
  pub fn archives(&self) -> impl Iterator<Item = String> + '_ {
    TARGETS.iter().map(|target| self.archive(target))
  }
}

impl fmt::Display for Tag {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.0)
  }
}

fn is_number(text: &str) -> bool {
  !text.is_empty()
    && text.bytes().all(|byte| byte.is_ascii_digit())
    && (text == "0" || !text.starts_with('0'))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn stable_tags_parse() {
    for text in ["v0.1.4", "v0.0.0", "v10.20.30"] {
      assert_eq!(Tag::parse(text).as_ref().map(Tag::as_str), Some(text));
    }
  }

  #[test]
  fn other_tags_are_refused() {
    let refused = [
      "",
      "0.1.4",
      "V0.1.4",
      "v",
      "v0.1",
      "v0.1.4.5",
      "v01.1.4",
      "v0.01.4",
      "v0.1.04",
      "v0.1.4-rc.1",
      "v0..4",
      "v.1.4",
      "v0.1.x",
      "v0.1.4\n",
      " v0.1.4",
      "v0.1.4/../x",
    ];
    for text in refused {
      assert_eq!(Tag::parse(text), None, "{text:?}");
    }
  }

  #[test]
  fn archives_follow_ci_naming() -> anyhow::Result<()> {
    let tag = Tag::parse("v0.1.4").ok_or_else(|| anyhow::anyhow!("fixture tag"))?;
    let archives: Vec<String> = tag.archives().collect();
    let expected: Vec<String> = TARGETS
      .iter()
      .map(|target| format!("ccusage-v0.1.4-{}.tar.gz", target.triple))
      .collect();
    assert_eq!(archives, expected);
    let macs: Vec<&str> = mac_targets().map(|target| target.triple).collect();
    assert_eq!(macs, ["aarch64-apple-darwin", "x86_64-apple-darwin"]);
    Ok(())
  }

  #[test]
  fn identity_and_requirement_name_the_geodica_team() {
    assert_eq!(
      IDENTITY,
      "Developer ID Application: Geodica Pty Ltd (76BQL8L47U)"
    );
    assert_eq!(
      REQUIREMENT,
      "identifier \"com.geodica.ccusage\" and anchor apple generic and certificate leaf[subject.OU] = \"76BQL8L47U\" and certificate leaf[field.1.2.840.113635.100.6.1.13] exists"
    );
  }
}
