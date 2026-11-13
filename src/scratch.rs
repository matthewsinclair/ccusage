//! The unit tests' scratch directories: each test's own, private, under the temporary
//! directory, and removed when it is dropped.

use std::{
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub struct Scratch(PathBuf);

impl Scratch {
  pub fn new() -> crate::Result<Self> {
    let dir = std::env::temp_dir().join(format!(
      "ccusage-scratch-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    crate::storage::private_dir(&dir)?;
    Ok(Self(dir))
  }

  pub fn path(&self) -> &Path {
    &self.0
  }
}

impl Drop for Scratch {
  fn drop(&mut self) {
    if let Err(error) = std::fs::remove_dir_all(&self.0) {
      eprintln!("could not remove scratch directory: {error}");
    }
  }
}
