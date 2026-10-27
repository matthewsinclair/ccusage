use std::{fs, io::Write, path::Path};

use crate::Result;

pub fn private_dir(path: &Path) -> Result<()> {
  let mut builder = fs::DirBuilder::new();
  builder.recursive(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::DirBuilderExt;
    builder.mode(0o700);
  }
  builder.create(path)?;
  Ok(())
}

pub fn create_config(path: &Path, bytes: &[u8]) -> Result<()> {
  private_dir(path.parent().ok_or("config path has no parent")?)?;
  let mut options = fs::OpenOptions::new();
  options.write(true).create_new(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o600);
  }
  let mut file = options.open(path)?;
  file.write_all(bytes)?;
  file.sync_all()?;
  Ok(())
}

/// Stage beside the destination so readers never see a partial state file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
  let parent = path.parent().ok_or("state path has no parent")?;
  private_dir(parent)?;
  let stamp = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)?
    .as_nanos();
  let temp = parent.join(format!(".ccusage-{}-{stamp}.tmp", std::process::id()));
  let result = (|| -> Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
      use std::os::unix::fs::OpenOptionsExt;
      options.mode(0o600);
    }
    let mut file = options.open(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(&temp, path)?;
    Ok(())
  })();
  if result.is_err()
    && let Err(error) = fs::remove_file(&temp)
    && error.kind() != std::io::ErrorKind::NotFound
  {
    eprintln!("ccusage: could not remove temporary state file: {error}");
  }
  result
}

pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
  match fs::read(path) {
    Ok(bytes) => Ok(Some(bytes)),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
    Err(error) => Err(error.into()),
  }
}
