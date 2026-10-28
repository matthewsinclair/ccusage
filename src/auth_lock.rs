//! Directory locks compatible with Claude Code's proper-lockfile protocol.
//! Heartbeats keep live holders from being reclaimed; abandoned locks expire.
use crate::{Result, storage};
use std::{
  fs::{self, File, FileTimes},
  path::{Path, PathBuf},
  sync::mpsc,
  thread::{self, JoinHandle},
  time::{Duration, Instant, SystemTime},
};

pub(crate) struct Lock {
  path: PathBuf,
  file: File,
  stop: mpsc::Sender<()>,
  heartbeat: Option<JoinHandle<()>>,
}

fn same_file(a: &fs::Metadata, b: &fs::Metadata) -> bool {
  #[cfg(unix)]
  {
    use std::os::unix::fs::MetadataExt;
    a.dev() == b.dev() && a.ino() == b.ino()
  }
  #[cfg(not(unix))]
  {
    a.created().ok() == b.created().ok()
  }
}

impl Lock {
  pub(crate) fn acquire(path: PathBuf, stale: Duration) -> Result<Self> {
    Self::acquire_for(path, stale, Duration::from_secs(20))
  }

  fn acquire_for(path: PathBuf, stale: Duration, timeout: Duration) -> Result<Self> {
    let start = Instant::now();
    loop {
      let mut builder = fs::DirBuilder::new();
      #[cfg(unix)]
      {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
      }
      match builder.create(&path) {
        Ok(()) => break,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
          let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
          };
          if !metadata.is_dir() || metadata.is_symlink() {
            return Err("credential lock is not a directory; refusing to refresh".into());
          }
          if metadata.modified()?.elapsed().is_ok_and(|age| age > stale) {
            // Recheck identity and heartbeat before removing an abandoned lock.
            if let Ok(current) = fs::symlink_metadata(&path)
              && same_file(&metadata, &current)
              && metadata.modified()? == current.modified()?
            {
              match fs::remove_dir(&path) {
                Ok(()) => continue,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => {}
              }
            }
          }
          if start.elapsed() >= timeout {
            return Err("another process is updating Claude credentials; try again shortly".into());
          }
          thread::sleep(Duration::from_millis(100));
        }
        Err(error) => return Err(error.into()),
      }
    }
    let file = File::open(&path)?;
    let pulse = file.try_clone()?;
    let (stop, receiver) = mpsc::channel();
    let heartbeat = thread::spawn(move || {
      while receiver.recv_timeout(Duration::from_secs(2)) == Err(mpsc::RecvTimeoutError::Timeout) {
        if pulse
          .set_times(FileTimes::new().set_modified(SystemTime::now()))
          .is_err()
        {
          break;
        }
      }
    });
    Ok(Self {
      path,
      file,
      stop,
      heartbeat: Some(heartbeat),
    })
  }

  pub(crate) fn verify(&self) -> Result<()> {
    let current = fs::symlink_metadata(&self.path)?;
    if !current.is_symlink() && same_file(&self.file.metadata()?, &current) {
      Ok(())
    } else {
      Err("credential lock changed during refresh; try again".into())
    }
  }
}

impl Drop for Lock {
  fn drop(&mut self) {
    let _ = self.stop.send(());
    if let Some(heartbeat) = self.heartbeat.take() {
      let _ = heartbeat.join();
    }
    if self.verify().is_ok() {
      let _ = fs::remove_dir(&self.path);
    }
  }
}

pub(crate) fn refresh(dir: &Path) -> Result<(Lock, Lock)> {
  storage::private_dir(dir)?;
  let current = Lock::acquire(dir.join(".oauth_refresh.lock"), Duration::from_secs(60))?;
  let mut legacy = dir.as_os_str().to_owned();
  legacy.push(".lock");
  let legacy = Lock::acquire(PathBuf::from(legacy), Duration::from_secs(60))?;
  Ok((current, legacy))
}

pub(crate) fn writer(dir: &Path) -> Result<Lock> {
  Lock::acquire(dir.join(".storage-write.lock"), Duration::from_secs(15))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lock_contention_stale_recovery_and_replacement() -> Result<()> {
    let dir = std::env::temp_dir().join(format!("ccusage-lock-{}", std::process::id()));
    storage::private_dir(&dir)?;
    let path = dir.join("lock");
    let lock = Lock::acquire(path.clone(), Duration::from_secs(60))?;
    assert!(Lock::acquire_for(path.clone(), Duration::from_secs(60), Duration::ZERO).is_err());
    fs::remove_dir(&path)?;
    fs::create_dir(&path)?;
    assert!(lock.verify().is_err());
    drop(lock);
    assert!(path.exists());
    File::open(&path)?.set_times(FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))?;
    let lock = Lock::acquire(path.clone(), Duration::from_secs(60))?;
    lock.verify()?;
    drop(lock);
    assert!(!path.exists());
    fs::remove_dir(&dir)?;
    Ok(())
  }

  #[test]
  fn independent_process_lock_and_heartbeat() -> Result<()> {
    use std::process::{Command, Stdio};
    if let Some(dir) = std::env::var_os("CCUSAGE_LOCK_TEST_CHILD") {
      let dir = PathBuf::from(dir);
      let _lock = Lock::acquire(dir.join("lock"), Duration::from_secs(60))?;
      storage::write_atomic(&dir.join("ready"), b"ready")?;
      let start = Instant::now();
      while !dir.join("release").exists() {
        if start.elapsed() > Duration::from_secs(10) {
          return Err("lock test parent timed out".into());
        }
        thread::sleep(Duration::from_millis(20));
      }
      return Ok(());
    }
    let dir = std::env::temp_dir().join(format!("ccusage-process-lock-{}", std::process::id()));
    storage::private_dir(&dir)?;
    struct Child(std::process::Child);
    impl Drop for Child {
      fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
      }
    }
    let mut child = Child(
      Command::new(std::env::current_exe()?)
        .args([
          "--exact",
          "auth_lock::tests::independent_process_lock_and_heartbeat",
        ])
        .env("CCUSAGE_LOCK_TEST_CHILD", &dir)
        .stdout(Stdio::null())
        .spawn()?,
    );
    let start = Instant::now();
    while !dir.join("ready").exists() {
      if start.elapsed() > Duration::from_secs(5) {
        return Err("lock test child timed out".into());
      }
      thread::sleep(Duration::from_millis(20));
    }
    let path = dir.join("lock");
    let before = fs::metadata(&path)?.modified()?;
    assert!(Lock::acquire_for(path.clone(), Duration::from_secs(60), Duration::ZERO).is_err());
    thread::sleep(Duration::from_millis(2200));
    assert!(fs::metadata(&path)?.modified()? > before);
    storage::write_atomic(&dir.join("release"), b"release")?;
    assert!(child.0.wait()?.success());
    assert!(!path.exists());
    fs::remove_dir_all(&dir)?;
    Ok(())
  }
}
