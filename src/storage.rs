use std::{
  fs,
  io::{Read, Write},
  path::Path,
};

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

fn validate_private_metadata(metadata: &fs::Metadata, path: &Path) -> Result<()> {
  if !metadata.is_file() {
    return Err(format!("private path is not a regular file: {}", path.display()).into());
  }
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode() & 0o7777;
    if mode != 0o600 {
      return Err(
        format!(
          "private file {} has mode {mode:04o}; set it to 0600 with chmod 600",
          path.display()
        )
        .into(),
      );
    }
  }
  Ok(())
}

/// Inspect only metadata; doctor must not read credential contents.
pub fn check_private_file(path: &Path) -> Result<bool> {
  match fs::metadata(path) {
    Ok(metadata) => {
      validate_private_metadata(&metadata, path)?;
      Ok(true)
    }
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
    Err(error) => Err(error.into()),
  }
}

pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
  let metadata = match fs::metadata(path) {
    Ok(metadata) => metadata,
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
    Err(error) => return Err(error.into()),
  };
  if !metadata.is_file() {
    return Err(format!("private path is not a regular file: {}", path.display()).into());
  }
  let mut file = fs::File::open(path)?;
  // Inspect the opened file, not just the path, before reading any contents.
  let metadata = file.metadata()?;
  if !metadata.is_file() {
    return Err("private file changed type while opening".into());
  }
  validate_private_metadata(&metadata, path)?;
  let mut bytes = Vec::new();
  file.read_to_end(&mut bytes)?;
  Ok(Some(bytes))
}
