//! The one way the active account changes. The CLI's `use`, `run NAME` in shared mode,
//! the TUI's `a` key and `which --use` all call `to`, and `use --none` calls `none`, so
//! a switch means the same thing wherever it starts.

use crate::{
  Result, auth,
  config::Store,
  seat::{self, Move, Refusal, Step},
  shared,
  usage::{self, Standing},
};
use serde_json::Value;

/// What a switch did, which decides what the verb that asked for it says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Switched {
  /// Shared mode: the account's login is the one every Claude session reads.
  Shared,
  /// Shared mode, adopting 0.1.8's layout: the shared config's own login was replaced,
  /// and this says whose it was.
  Adopted(String),
  /// Shared mode off: future `ccusage run` launches use the account.
  Selected,
}

impl Switched {
  /// The one line a verb shows for the switch. Pure.
  pub fn message(&self, name: &str) -> String {
    match self {
      Self::Shared => format!(
        "Active: {name}. Every Claude session uses it from its next request; what a running session loaded when it started, such as connectors and Remote Control, stays the previous account's until it restarts."
      ),
      Self::Adopted(replaced) => format!(
        "{} It replaced the shared config's own login{replaced}.",
        Self::Shared.message(name)
      ),
      Self::Selected => format!("Active: {name}. Future ccusage run launches use this account."),
    }
  }
}

/// Whether the first switch on 0.1.8's layout may replace the shared config's own
/// login, and whose it was: it may when Claude can no longer renew that login, or when
/// another login of its account is parked in a profile (`parked` holds the account
/// records of the profiles holding their own). Otherwise it may be its account's only
/// login, and the switch is refused before anything moves. Pure.
fn adoption(standing: &Standing, parked: &[Value]) -> Result<String> {
  match standing {
    Standing::Dead => Ok(", which Claude could no longer renew".to_owned()),
    Standing::Live(account) => {
      let email = account
        .get("emailAddress")
        .and_then(Value::as_str)
        .unwrap_or("an unnamed account");
      if parked
        .iter()
        .any(|record| shared::same_account(account, record))
      {
        Ok(format!(
          " of {email}, whose account keeps another login in a profile"
        ))
      } else {
        Err(format!(
          "the shared config holds a login of {email}, and no profile holds another login of that account, so it may be that account's only one; nothing was moved. To keep it, add a profile for that account with ccusage add NAME --email {email} and ccusage login NAME, then switch again"
        )
        .into())
      }
    }
  }
}

/// The account records of the profiles that hold a login of their own: each one's
/// configured email, and the record its `.claude.json` keeps.
fn parked_accounts(store: &Store) -> Result<Vec<Value>> {
  let mut records = Vec::new();
  for name in store.config.profiles.keys() {
    if auth::read(&store.profile_dir(name)?).is_err() {
      continue;
    }
    if let Some(email) = &store.profile(name)?.email {
      records.push(serde_json::json!({ "emailAddress": email }));
    }
    records.extend(shared::profile_account(store, name)?);
  }
  Ok(records)
}

/// What `use --none` says once it is done.
pub const NONE: &str = "No account is active: every login is back in its own profile and the shared config holds none, as an older ccusage expects.";

/// Make `name` the active account. A profile with no login, or in shared mode with no
/// account record, is refused before anything changes. In shared mode an unfinished
/// move is finished first; then `name`'s login is seated in the shared store and the
/// previous account's goes back to its profile, the shared config records `name`'s
/// account, and on macOS launchd stops pointing GUI sessions at a profile's own login.
pub fn to(store: &Store, name: &str) -> Result<Switched> {
  let Some(shared) = store.shared_dir()? else {
    auth::read(&store.profile_dir(name)?)?;
    store.activate(name)?;
    return Ok(Switched::Selected);
  };
  finish(store)?;
  auth::read(&seat::home(store, name)?)?;
  shared::account_of(store, name)?;
  let seated = seat::seated(store)?;
  let mut switched = Switched::Shared;
  if seated.as_deref() != Some(name) {
    let mut request = Move {
      from: seated,
      to: Some(name.to_owned()),
      replace: false,
    };
    // 0.1.8's layout: no profile is seated, and the shared config holds its own login.
    if request.from.is_none() && auth::read(&shared).is_ok() {
      let replaced = adoption(&usage::standing(&shared)?, &parked_accounts(store)?)?;
      request.replace = true;
      switched = Switched::Adopted(replaced);
    }
    relocate(store, &request)?;
  }
  shared::record(store, &shared, name)?;
  store.activate(name)?;
  #[cfg(target_os = "macos")]
  shared::unset_launchd()?;
  Ok(switched)
}

/// Put the active account's login back in its profile and leave the shared store
/// without one, as ccusage 0.1.8 kept them: shared mode's way off. Refused when shared
/// mode is off, where no login is shared.
pub fn none(store: &Store) -> Result<()> {
  if store.shared_dir()?.is_none() {
    return Err(
      "shared mode is off, so no login is shared and use --none has nothing to do".into(),
    );
  }
  finish(store)?;
  if let Some(seated) = seat::seated(store)? {
    relocate(
      store,
      &Move {
        from: Some(seated),
        ..Move::default()
      },
    )?;
  }
  #[cfg(target_os = "macos")]
  shared::unset_launchd()?;
  Ok(())
}

/// Seat `name`'s fresh login, which `ccusage login` has just put in its profile's own
/// store, in place of the one the shared store holds for it. Only the seated profile's
/// own login is replaced, so no account is left without one.
pub fn reseat(store: &Store, name: &str) -> Result<()> {
  let Some(shared) = store.shared_dir()? else {
    return Ok(());
  };
  if seat::seated(store)?.as_deref() != Some(name) {
    return Ok(());
  }
  let request = Move {
    to: Some(name.to_owned()),
    replace: true,
    ..Move::default()
  };
  seat::run(store, &request, &seat::places(store, &request)?, None)?;
  shared::record(store, &shared, name)
}

/// Finish the move an interrupted run left, verifying the outgoing login again when the
/// journal stopped before its park was confirmed.
fn finish(store: &Store) -> Result<()> {
  let Some(journal) = seat::pending(store)? else {
    return Ok(());
  };
  let places = seat::places(store, &journal.request)?;
  let verified = match (&journal.request.from, journal.step) {
    (Some(from), Step::Begun) => Some(usage::verified_token(store, from, &places.shared.dir)?),
    _ => None,
  };
  seat::run(store, &journal.request, &places, verified.as_deref())
}

/// Make `request`, with its outgoing login verified first. Claude Code may renew that
/// login between the check and the move's locks, which the move refuses as unverified,
/// so the login is verified once more and the move tried again.
fn relocate(store: &Store, request: &Move) -> Result<()> {
  let places = seat::places(store, request)?;
  let verify = || {
    request
      .from
      .as_deref()
      .map(|from| usage::verified_token(store, from, &places.shared.dir))
      .transpose()
  };
  match seat::run(store, request, &places, verify()?.as_deref()) {
    Err(error)
      if request.from.is_some()
        && matches!(
          error.downcast_ref::<Refusal>(),
          Some(Refusal::Unverified(_))
        ) =>
    {
      seat::run(store, request, &places, verify()?.as_deref())
    }
    moved => moved,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  // ST0003 AT-03.9: adoption replaces the shared config's own login only when Claude can
  // no longer renew it, or when its account keeps another login in a profile, and says
  // whose it was; otherwise it refuses, saying how to keep that account's login.
  #[test]
  fn adoption_replaces_only_a_dead_or_duplicated_shared_login() -> Result<()> {
    let account = json!({"accountUuid": "u-01", "emailAddress": "e01@example.com"});
    let parked = [
      json!({"accountUuid": "u-02", "emailAddress": "e02@example.com"}),
      json!({"accountUuid": "u-01", "emailAddress": "renamed@example.com"}),
    ];
    let replaced = adoption(&Standing::Live(account.clone()), &parked)?;
    assert!(replaced.contains("e01@example.com"), "{replaced}");
    let refused = adoption(&Standing::Live(account), &parked[..1])
      .err()
      .ok_or("expected adoption to be refused")?;
    assert!(
      refused
        .to_string()
        .contains("ccusage add NAME --email e01@example.com"),
      "{refused}"
    );
    assert!(adoption(&Standing::Dead, &[])?.contains("could no longer renew"));
    Ok(())
  }
}
