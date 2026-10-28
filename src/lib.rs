pub mod auth;
mod auth_lock;
pub mod cli;
pub mod config;
pub mod display;
pub mod doctor;
pub mod storage;
pub mod usage;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

pub fn now() -> i64 {
  time::OffsetDateTime::now_utc().unix_timestamp()
}

/// Never allow configuration or server text to inject terminal control sequences.
pub fn clean(text: &str) -> String {
  text.chars().filter(|c| !c.is_control()).collect()
}
