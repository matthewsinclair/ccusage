//! Shared mode's one live login. The seated profile's login is in the shared config's
//! own store, which every session reads; every other profile's login is parked in that
//! profile's own store. A move takes the seated login back to its profile and seats
//! another in its place, and at no point is an account left without a login.
//!
//! A pure plan turns the stores' documents and the request into the ordered writes, or
//! a refusal before any write. The executor holds the locks for the whole move, makes
//! each write, reads it back and journals the step it reached, so the next run
//! finishes a move that was interrupted. `claudeAiOauth` is the only key that moves.

use crate::{
  Result,
  auth::Location,
  auth_lock::{self, Lock},
  config::Store,
  storage,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{path::PathBuf, time::Duration};

/// The key a credential document keeps its login under.
const LOGIN: &str = "claudeAiOauth";
/// The move in progress, in the state directory.
const JOURNAL: &str = "move.json";
/// The profile whose login the shared store holds, in the state directory.
const SEAT: &str = "seat.json";
/// ccusage's own lock, so that two moves never interleave.
const SWITCH_LOCK: &str = "switch.lock";
/// When the switch lock is taken as abandoned; a live holder's heartbeat keeps it fresh.
const SWITCH_STALE: Duration = Duration::from_secs(60);

/// A credential document: the JSON object Claude Code keeps a login in.
pub(crate) type Document = Map<String, Value>;

/// The login a document holds, if any. Pure.
pub(crate) fn login(document: &Document) -> Option<&Value> {
  document.get(LOGIN).filter(|value| !value.is_null())
}

/// Whether a stored credential document holds a login. Pure.
pub(crate) fn holds_login(bytes: &[u8]) -> Result<bool> {
  let document: Document = serde_json::from_slice(bytes)
    .map_err(|_| "a Claude credential document is not a JSON object (contents redacted)")?;
  Ok(login(&document).is_some())
}

/// `document` holding `login`, with every other key and their order as they were. Pure.
fn with_login(document: &Document, login: &Value) -> Document {
  let mut document = document.clone();
  document.insert(LOGIN.to_owned(), login.clone());
  document
}

/// `document` holding no login, with every other key and their order as they were.
/// Pure.
fn without_login(document: &Document) -> Document {
  let mut document = document.clone();
  // remove() would swap the last key into its place.
  document.shift_remove(LOGIN);
  document
}

/// A login's access token. Pure.
fn access_token(login: &Value) -> Option<&str> {
  login.get("accessToken").and_then(Value::as_str)
}

/// How far a move has gone. Each step is journaled once its write has been read back.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Step {
  /// The journal is written, and no write is confirmed.
  Begun,
  /// The outgoing login is in its profile's store, or there was none to park.
  Parked,
  /// The shared store holds the incoming login, or none when nothing comes in.
  Seated,
  /// The incoming profile's store holds no login, or nothing came in.
  Cleared,
}

/// A move: the seated profile, whose login leaves the shared store for its own, and
/// the profile whose parked login takes its place. Either may be absent: with no
/// incoming profile the shared store is left without a login, and with no outgoing
/// one a login is seated in a shared store that holds none.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Move {
  pub from: Option<String>,
  pub to: Option<String>,
  /// The shared store's login may be overwritten although no profile is seated with
  /// it, which the first switch on 0.1.8's layout decides (ST0003/WP03).
  #[serde(default)]
  pub replace: bool,
}

/// A move that has begun, and the step it reached. It names the profiles and never a
/// token.
#[derive(Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Journal {
  pub request: Move,
  pub step: Step,
}

/// Which profile's login the shared store holds: none on 0.1.8's layout, where each
/// profile keeps its own, and none once every login is parked.
#[derive(Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Seat {
  pub seated: Option<String>,
}

impl Seat {
  pub(crate) fn load(store: &Store) -> Result<Self> {
    storage::read_optional(&store.state.join(SEAT))?
      .map(|bytes| {
        serde_json::from_slice(&bytes).map_err(|_| "ccusage's seat record is not valid".into())
      })
      .transpose()
      .map(Option::unwrap_or_default)
  }

  fn save(&self, store: &Store) -> Result<()> {
    storage::write_atomic(&store.state.join(SEAT), &serde_json::to_vec(self)?)
  }
}

/// Why a move does not go ahead. Every refusal comes before any write.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Refusal {
  /// The incoming profile's store holds no login.
  NothingToSeat(String),
  /// The seated profile's login should be in the shared store, which holds none.
  SharedEmpty(String),
  /// The shared login was not verified as the outgoing profile's.
  Unverified(String),
  /// The outgoing profile's own store holds a login, which parking would overwrite.
  Occupied(String),
  /// The shared store holds a login no profile is seated with, which may be its
  /// account's only one.
  OnlyLogin,
  /// An interrupted move found a login where it had not left one.
  Changed,
}

impl std::fmt::Display for Refusal {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NothingToSeat(name) => write!(
        formatter,
        "{name} holds no login to switch to; run ccusage login {name}"
      ),
      Self::SharedEmpty(name) => write!(
        formatter,
        "the shared config holds no login, though {name}'s should be there; nothing was moved; see ccusage doctor"
      ),
      Self::Unverified(name) => write!(
        formatter,
        "the shared config's login could not be verified as {name}'s, so nothing was moved; try again, and see ccusage doctor if it persists"
      ),
      Self::Occupied(name) => write!(
        formatter,
        "{name}'s own store already holds a login, which moving the shared login back would overwrite; nothing was moved; see ccusage doctor"
      ),
      Self::OnlyLogin => formatter.write_str(
        "the shared config holds a login that belongs to no profile, and it may be its account's only one; nothing was moved; see ccusage doctor",
      ),
      Self::Changed => formatter.write_str(
        "an interrupted switch cannot be finished, because a login is no longer where the switch left it; nothing more was written; see ccusage doctor",
      ),
    }
  }
}

impl std::error::Error for Refusal {}

/// The store a write goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
  Shared,
  Outgoing,
  Incoming,
}

/// One write of a move: a store's whole new document, and the step it reaches once
/// read back.
#[derive(Debug, PartialEq)]
pub(crate) struct Write {
  pub role: Role,
  pub document: Document,
  pub reached: Step,
}

/// A profile's side of a move: its name and its store's document now.
#[derive(Clone, Copy)]
pub(crate) struct Side<'a> {
  pub name: &'a str,
  pub document: &'a Document,
}

/// The writes that finish `request` from `step`, in order, given each store's
/// document now; or the refusal that stops it before any write. `verified` is the
/// shared login's access token, verified as the outgoing profile's before the move
/// began. A write the journal had not yet recorded is recognised from the documents
/// and not made again. Pure.
pub(crate) fn plan(
  step: Step,
  request: &Move,
  verified: Option<&str>,
  shared: &Document,
  from: Option<Side>,
  to: Option<Side>,
) -> std::result::Result<Vec<Write>, Refusal> {
  let mut writes = Vec::new();
  let held = login(shared);
  // Park: the outgoing login, once it is in its profile's own store.
  let parked = match from {
    None => None,
    Some(side) => match step {
      Step::Begun => {
        let held = held.ok_or_else(|| Refusal::SharedEmpty(side.name.to_owned()))?;
        match login(side.document) {
          None => {
            if verified.is_none() || verified != access_token(held) {
              return Err(Refusal::Unverified(side.name.to_owned()));
            }
            writes.push(Write {
              role: Role::Outgoing,
              document: with_login(side.document, held),
              reached: Step::Parked,
            });
          }
          // Written before the journal recorded it.
          Some(own) if own == held => {}
          Some(_) => return Err(Refusal::Occupied(side.name.to_owned())),
        }
        Some(held)
      }
      Step::Parked => Some(login(side.document).ok_or(Refusal::Changed)?),
      Step::Seated | Step::Cleared => None,
    },
  };
  // Seat: the shared store takes the incoming login, or gives up its own.
  if step <= Step::Parked {
    let incoming = match to {
      Some(side) => Some(login(side.document).ok_or_else(|| {
        if step == Step::Begun {
          Refusal::NothingToSeat(side.name.to_owned())
        } else {
          Refusal::Changed
        }
      })?),
      None => None,
    };
    // Once a login was parked, the seat can have been written only from Parked.
    let landed = (from.is_none() || step == Step::Parked) && held == incoming;
    if !landed {
      // The shared login is overwritten only once it is parked, or when the move says
      // replace.
      let free = match parked {
        Some(parked) => held == Some(parked),
        None => held.is_none() || request.replace,
      };
      if !free {
        return Err(if parked.is_some() {
          Refusal::Changed
        } else {
          Refusal::OnlyLogin
        });
      }
      writes.push(Write {
        role: Role::Shared,
        document: match incoming {
          Some(entering) => with_login(shared, entering),
          None => without_login(shared),
        },
        reached: Step::Seated,
      });
    }
  }
  // Clear: the incoming profile gives up the login the shared store now holds.
  if step <= Step::Seated
    && let Some(side) = to
    && login(side.document).is_some()
  {
    writes.push(Write {
      role: Role::Incoming,
      document: without_login(side.document),
      reached: Step::Cleared,
    });
  }
  Ok(writes)
}

/// A store a move touches: the config directory whose locks guard it, and where its
/// document lives.
pub(crate) struct Place {
  pub dir: PathBuf,
  pub location: Location,
}

/// The shared config's store, and the stores of a move's two profiles.
pub(crate) struct Places {
  pub shared: Place,
  pub from: Option<Place>,
  pub to: Option<Place>,
}

/// The stores `request` touches, found as Claude Code finds them; a store that does
/// not exist is an error that says how to make it.
pub(crate) fn places(store: &Store, request: &Move) -> Result<Places> {
  let shared = store
    .shared_dir()?
    .ok_or("shared mode is off; set claude_config_dir")?;
  let missing = format!(
    "the shared Claude config {} has no credential store; start Claude once with CLAUDE_CONFIG_DIR={} and sign in",
    shared.display(),
    shared.display()
  );
  let profile = |name: &Option<String>| -> Result<Option<Place>> {
    name
      .as_deref()
      .map(|name| {
        found(
          store.profile_dir(name)?,
          format!("profile {name} has no Claude credential store; run ccusage login {name}"),
        )
      })
      .transpose()
  };
  Ok(Places {
    shared: found(shared, missing)?,
    from: profile(&request.from)?,
    to: profile(&request.to)?,
  })
}

fn found(dir: PathBuf, missing: String) -> Result<Place> {
  match crate::auth::load(&dir)? {
    Some((location, _)) => Ok(Place { dir, location }),
    None => Err(missing.into()),
  }
}

/// The move an interrupted run left unfinished, if any, and the step it reached. A run
/// finishes it before it does anything else.
pub(crate) fn pending(store: &Store) -> Result<Option<Journal>> {
  read_journal(store)
}

/// The profile whose login the shared store holds, if any.
pub(crate) fn seated(store: &Store) -> Result<Option<String>> {
  Ok(Seat::load(store)?.seated)
}

/// The config directory whose store holds `name`'s login: the shared config's for the
/// profile seated there, the profile's own otherwise and whenever shared mode is off.
/// While a move is unfinished, where its two profiles' logins are is not settled, so
/// they are refused until `use` finishes it.
pub(crate) fn home(store: &Store, name: &str) -> Result<PathBuf> {
  let Some(shared) = store.shared_dir()? else {
    return store.profile_dir(name);
  };
  if let Some(Journal { request, .. }) = pending(store)?
    && [&request.from, &request.to]
      .into_iter()
      .any(|moving| moving.as_deref() == Some(name))
  {
    return Err(unfinished(&request).into());
  }
  if seated(store)?.as_deref() == Some(name) {
    Ok(shared)
  } else {
    store.profile_dir(name)
  }
}

/// What a verb says when an interrupted move stands in its way: the `use` that
/// finishes it.
pub(crate) fn unfinished(request: &Move) -> String {
  format!(
    "an interrupted switch is unfinished; run ccusage use {} to finish it",
    request.to.as_deref().unwrap_or("--none")
  )
}

fn read_journal(store: &Store) -> Result<Option<Journal>> {
  storage::read_optional(&store.state.join(JOURNAL))?
    .map(|bytes| {
      serde_json::from_slice(&bytes)
        .map_err(|_| "ccusage's switch journal is not valid; see ccusage doctor".into())
    })
    .transpose()
}

fn save_journal(store: &Store, request: &Move, step: Step) -> Result<()> {
  let journal = Journal {
    request: request.clone(),
    step,
  };
  storage::write_atomic(&store.state.join(JOURNAL), &serde_json::to_vec(&journal)?)
}

fn remove_journal(store: &Store) -> Result<()> {
  match std::fs::remove_file(store.state.join(JOURNAL)) {
    Ok(()) => Ok(()),
    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
    Err(error) => Err(error.into()),
  }
}

/// Locks held for a whole move, released last first.
struct Held(Vec<Lock>);

impl Drop for Held {
  fn drop(&mut self) {
    while self.0.pop().is_some() {}
  }
}

/// ccusage's switch lock, then Claude Code's refresh locks and write lock for the
/// shared store, then for each profile's store in name order. A session's refresh
/// waits for them, then adopts whatever login it finds.
fn lock(store: &Store, request: &Move, places: &Places) -> Result<Held> {
  storage::private_dir(&store.state)?;
  let mut held = Held(vec![Lock::acquire(
    store.state.join(SWITCH_LOCK),
    SWITCH_STALE,
  )?]);
  let mut profiles: Vec<(&str, &Place)> = [
    (request.from.as_deref(), places.from.as_ref()),
    (request.to.as_deref(), places.to.as_ref()),
  ]
  .into_iter()
  .filter_map(|(name, place)| name.zip(place))
  .collect();
  profiles.sort_by_key(|(name, _)| *name);
  profiles.dedup_by_key(|(name, _)| *name);
  for place in std::iter::once(&places.shared).chain(profiles.into_iter().map(|(_, place)| place)) {
    let (current, legacy) = auth_lock::refresh(&place.dir)?;
    held.0.extend([current, legacy]);
    held.0.push(auth_lock::writer(&place.dir)?);
  }
  Ok(held)
}

/// A store's document, read whole.
fn document(place: &Place) -> Result<Document> {
  let bytes = place.location.read()?.ok_or_else(|| {
    format!(
      "the credential store of {} is gone; nothing was moved",
      place.dir.display()
    )
  })?;
  serde_json::from_slice(&bytes)
    .map_err(|_| "a Claude credential document is not a JSON object (contents redacted)".into())
}

/// A profile named by the move, with its store.
fn paired<'a>(
  name: &'a Option<String>,
  place: &'a Option<Place>,
) -> Result<Option<(&'a str, &'a Place)>> {
  match (name.as_deref(), place.as_ref()) {
    (Some(name), Some(place)) => Ok(Some((name, place))),
    (None, None) => Ok(None),
    _ => Err("a move must name each profile together with its store".into()),
  }
}

/// A profile's side of the plan: its name and the document its store holds.
fn side<'a>(pair: Option<(&'a str, &Place)>, document: &'a Option<Document>) -> Option<Side<'a>> {
  pair
    .zip(document.as_ref())
    .map(|((name, _), document)| Side { name, document })
}

/// Make `request`, or finish it when the journal holds it. `verified` is the shared
/// login's access token, verified as `request.from`'s; it is needed only while that
/// login is still to be parked.
pub(crate) fn run(
  store: &Store,
  request: &Move,
  places: &Places,
  verified: Option<&str>,
) -> Result<()> {
  run_until(store, request, places, verified, &mut |_| Ok(()))
}

/// The points a move passes, where a test stops it as an interruption would.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Point {
  Journaled(Step),
  Wrote(Step),
  Selected,
}

fn run_until(
  store: &Store,
  request: &Move,
  places: &Places,
  verified: Option<&str>,
  stop: &mut dyn FnMut(Point) -> Result<()>,
) -> Result<()> {
  let from = paired(&request.from, &places.from)?;
  let to = paired(&request.to, &places.to)?;
  let _held = lock(store, request, places)?;
  let step = match read_journal(store)? {
    None => Step::Begun,
    Some(journal) if journal.request == *request => journal.step,
    Some(journal) => return Err(unfinished(&journal.request).into()),
  };
  let shared = document(&places.shared)?;
  let outgoing = from.map(|(_, place)| document(place)).transpose()?;
  let incoming = to.map(|(_, place)| document(place)).transpose()?;
  let writes = plan(
    step,
    request,
    verified,
    &shared,
    side(from, &outgoing),
    side(to, &incoming),
  )?;
  // Every write is encoded and checked before the first is made.
  let mut ready = Vec::new();
  for write in writes {
    let place = match write.role {
      Role::Shared => &places.shared,
      Role::Outgoing => from.ok_or("a park needs the outgoing profile's store")?.1,
      Role::Incoming => to.ok_or("a clear needs the incoming profile's store")?.1,
    };
    let bytes =
      serde_json::to_vec(&write.document).map_err(|_| "could not encode Claude credentials")?;
    place.location.check(&bytes)?;
    ready.push((place, bytes, write.reached));
  }
  if step == Step::Begun {
    save_journal(store, request, step)?;
    stop(Point::Journaled(step))?;
  }
  for (place, bytes, reached) in ready {
    place.location.write(&bytes)?;
    stop(Point::Wrote(reached))?;
    save_journal(store, request, reached)?;
    stop(Point::Journaled(reached))?;
  }
  // The incoming profile becomes the selected one with the seat, inside the journal,
  // so a run never finds a login seated for a profile it does not take as active.
  if let Some(name) = request.to.as_deref() {
    store.activate(name)?;
  }
  Seat {
    seated: request.to.clone(),
  }
  .save(store)?;
  stop(Point::Selected)?;
  remove_journal(store)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{Config, Profile};
  use serde_json::json;
  use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
  };

  const SHARED: &str = "shared";
  const A: &str = "a";
  const B: &str = "b";

  fn login_a() -> Value {
    json!({"accessToken": "ACCESS_A", "refreshToken": "REFRESH_A", "expiresAt": 9_999_999_999_999_i64,
      "scopes": ["user:inference"], "subscriptionType": "max"})
  }

  fn login_b() -> Value {
    json!({"accessToken": "ACCESS_B", "refreshToken": "REFRESH_B", "expiresAt": 9_999_999_999_999_i64,
      "scopes": ["user:inference"], "subscriptionType": "max", "rateLimitTier": "b's own"})
  }

  fn object(value: Value) -> Result<Document> {
    serde_json::from_value(value).map_err(Into::into)
  }

  /// a seated: its login in the shared store, whose MCP logins stay, and b parked,
  /// with a key of its own before its login and two after it, so a removal that
  /// swapped the last key into the login's place would show in their order.
  fn seated_a() -> Result<[Document; 3]> {
    Ok([
      object(json!({LOGIN: login_a(), "mcpOAuth": {"server": {"accessToken": "MCP"}}}))?,
      object(json!({"note": "a's own"}))?,
      object(json!({"other": [1, 2], LOGIN: login_b(), "middle": "kept", "last": true}))?,
    ])
  }

  fn a_to_b() -> Move {
    Move {
      from: Some(A.into()),
      to: Some(B.into()),
      replace: false,
    }
  }

  /// The shared config's store and two profiles' stores as credentials files, with a
  /// state directory and a config naming the profiles.
  struct Fixture {
    root: PathBuf,
    store: Store,
  }

  impl Fixture {
    fn new([shared, a, b]: [Document; 3]) -> Result<Self> {
      static NEXT: AtomicU64 = AtomicU64::new(0);
      let root = std::env::temp_dir().join(format!(
        "ccusage-seat-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
      ));
      storage::private_dir(&root)?;
      let root = root.canonicalize()?;
      let mut config = Config {
        claude_config_dir: Some(root.join(SHARED)),
        ..Config::default()
      };
      for name in [A, B] {
        config.profiles.insert(
          name.into(),
          Profile {
            email: None,
            config_dir: Some(root.join(name)),
          },
        );
      }
      let store = Store {
        path: root.join("config.toml"),
        root: root.clone(),
        data: root.join("data"),
        state: root.join("state"),
        cache: root.join("cache"),
        config,
      };
      let fixture = Self { root, store };
      for (name, document) in [(SHARED, shared), (A, a), (B, b)] {
        storage::private_dir(&fixture.root.join(name))?;
        storage::write_atomic(&fixture.file(name), &serde_json::to_vec(&document)?)?;
      }
      Ok(fixture)
    }

    fn file(&self, name: &str) -> PathBuf {
      self.root.join(name).join(".credentials.json")
    }

    fn place(&self, name: &str) -> Place {
      Place {
        dir: self.root.join(name),
        location: Location::File(self.file(name)),
      }
    }

    fn places(&self, request: &Move) -> Places {
      Places {
        shared: self.place(SHARED),
        from: request.from.as_deref().map(|name| self.place(name)),
        to: request.to.as_deref().map(|name| self.place(name)),
      }
    }

    fn run(&self, request: &Move, verified: Option<&str>) -> Result<()> {
      run(&self.store, request, &self.places(request), verified)
    }

    fn until(
      &self,
      request: &Move,
      verified: Option<&str>,
      stop: &mut dyn FnMut(Point) -> Result<()>,
    ) -> Result<()> {
      run_until(&self.store, request, &self.places(request), verified, stop)
    }

    fn document(&self, name: &str) -> Result<Document> {
      Ok(serde_json::from_slice(&fs::read(self.file(name))?)?)
    }

    /// Every store's bytes, as a refusal must leave them.
    fn bytes(&self) -> Result<Vec<Vec<u8>>> {
      [SHARED, A, B]
        .into_iter()
        .map(|name| Ok(fs::read(self.file(name))?))
        .collect()
    }

    /// The stores holding `login`.
    fn holders(&self, login: &Value) -> Result<Vec<&'static str>> {
      let mut holders = Vec::new();
      for name in [SHARED, A, B] {
        if super::login(&self.document(name)?) == Some(login) {
          holders.push(name);
        }
      }
      Ok(holders)
    }

    fn journal(&self) -> bool {
      self.store.state.join(JOURNAL).exists()
    }
  }

  impl Drop for Fixture {
    fn drop(&mut self) {
      if let Err(error) = fs::remove_dir_all(&self.root) {
        eprintln!("could not clean seat fixture: {error}");
      }
    }
  }

  /// A document's keys and values apart from its login, in order. Read without
  /// `without_login`, so a fault there cannot hide in the expectation too.
  fn rest(document: &Document) -> Vec<(&String, &Value)> {
    document
      .iter()
      .filter(|(key, _)| key.as_str() != LOGIN)
      .collect()
  }

  // ST0003 AT-02.1: a move leaves each account one login, and every other key of each
  // document, in its order, as it was.
  #[test]
  fn a_move_leaves_each_account_one_login_and_every_other_key_as_it_was() -> Result<()> {
    let before = seated_a()?;
    let fixture = Fixture::new(before.clone())?;
    fixture.run(&a_to_b(), Some("ACCESS_A"))?;
    assert_eq!(fixture.holders(&login_a())?, [A]);
    assert_eq!(fixture.holders(&login_b())?, [SHARED]);
    for (name, before) in [SHARED, A, B].into_iter().zip(&before) {
      assert_eq!(rest(&fixture.document(name)?), rest(before), "{name}");
    }
    // The shared login keeps its place among the shared document's keys.
    assert_eq!(
      fixture.document(SHARED)?.keys().collect::<Vec<_>>(),
      before[0].keys().collect::<Vec<_>>()
    );
    assert_eq!(fixture.store.active()?.as_deref(), Some(B));
    assert_eq!(Seat::load(&fixture.store)?.seated.as_deref(), Some(B));
    assert!(!fixture.journal());
    assert!(!fixture.store.state.join(SWITCH_LOCK).exists());
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      for file in [fixture.file(A), fixture.file(SHARED), fixture.file(B)] {
        assert_eq!(fs::metadata(&file)?.permissions().mode() & 0o7777, 0o600);
      }
    }
    Ok(())
  }

  // ST0003 AT-02.2: parking the seated login leaves the shared store holding none.
  #[test]
  fn parking_everything_leaves_the_shared_store_without_a_login() -> Result<()> {
    let before = seated_a()?;
    let fixture = Fixture::new(before.clone())?;
    let request = Move {
      from: Some(A.into()),
      ..Move::default()
    };
    fixture.run(&request, Some("ACCESS_A"))?;
    assert_eq!(fixture.holders(&login_a())?, [A]);
    assert_eq!(fixture.holders(&login_b())?, [B]);
    assert_eq!(rest(&fixture.document(SHARED)?), rest(&before[0]));
    assert_eq!(super::login(&fixture.document(SHARED)?), None);
    assert_eq!(Seat::load(&fixture.store)?, Seat::default());
    Ok(())
  }

  // ST0003 AT-02.3: a parked login is seated in a shared store that holds none.
  #[test]
  fn a_login_is_seated_in_a_shared_store_that_holds_none() -> Result<()> {
    let [shared, _, b] = seated_a()?;
    let fixture = Fixture::new([
      without_login(&shared),
      object(json!({"note": "a's own"}))?,
      b,
    ])?;
    let request = Move {
      to: Some(B.into()),
      ..Move::default()
    };
    fixture.run(&request, None)?;
    assert_eq!(fixture.holders(&login_b())?, [SHARED]);
    assert_eq!(fixture.store.active()?.as_deref(), Some(B));
    Ok(())
  }

  // ST0003 AT-02.4: a move interrupted at any point the clean run passes keeps every
  // login somewhere, and the next run finishes it as the clean run did.
  #[test]
  fn a_move_interrupted_at_any_point_is_finished_by_the_next_run() -> Result<()> {
    // The points an uninterrupted move passes, and where it leaves every store.
    let clean = Fixture::new(seated_a()?)?;
    let mut points = Vec::new();
    clean.until(&a_to_b(), Some("ACCESS_A"), &mut |point| {
      points.push(point);
      Ok(())
    })?;
    assert!(!points.is_empty());
    for point in points {
      let fixture = Fixture::new(seated_a()?)?;
      let stopped = fixture.until(&a_to_b(), Some("ACCESS_A"), &mut |reached| {
        if reached == point {
          Err(crate::Error::from("interrupted"))
        } else {
          Ok(())
        }
      });
      assert!(stopped.is_err(), "{point:?}");
      for login in [login_a(), login_b()] {
        assert!(
          !fixture.holders(&login)?.is_empty(),
          "{point:?}: a login was lost"
        );
      }
      assert!(fixture.journal(), "{point:?}");
      let request = pending(&fixture.store)?
        .ok_or("the journal names no move")?
        .request;
      assert_eq!(request, a_to_b());
      fixture.run(&request, Some("ACCESS_A"))?;
      for name in [SHARED, A, B] {
        assert_eq!(
          fixture.document(name)?,
          clean.document(name)?,
          "{point:?}: {name}"
        );
      }
      assert_eq!(fixture.store.active()?, clean.store.active()?);
      assert_eq!(Seat::load(&fixture.store)?, Seat::load(&clean.store)?);
      assert!(!fixture.journal(), "{point:?}");
    }
    Ok(())
  }

  // ST0003 AT-02.5: another move is refused, before any write, while an interrupted one
  // is unfinished.
  #[test]
  fn another_move_waits_for_an_interrupted_one() -> Result<()> {
    let fixture = Fixture::new(seated_a()?)?;
    let stopped = fixture.until(&a_to_b(), Some("ACCESS_A"), &mut |point| match point {
      Point::Wrote(Step::Parked) => Err(crate::Error::from("interrupted")),
      _ => Ok(()),
    });
    assert!(stopped.is_err());
    let before = fixture.bytes()?;
    let other = Move {
      from: Some(A.into()),
      ..Move::default()
    };
    let refused = fixture.run(&other, Some("ACCESS_A"));
    assert!(
      refused.is_err_and(|error| error.to_string().contains("run ccusage use b to finish it"))
    );
    assert_eq!(fixture.bytes()?, before);
    Ok(())
  }

  // ST0003 AT-02.10: an interrupted move whose parked login has gone from its profile's
  // store is refused as changed, and writes nothing more.
  #[test]
  fn an_interrupted_move_whose_login_moved_is_refused_as_changed() -> Result<()> {
    let fixture = Fixture::new(seated_a()?)?;
    let stopped = fixture.until(&a_to_b(), Some("ACCESS_A"), &mut |point| match point {
      Point::Journaled(Step::Parked) => Err(crate::Error::from("interrupted")),
      _ => Ok(()),
    });
    assert!(stopped.is_err());
    storage::write_atomic(
      &fixture.file(A),
      &serde_json::to_vec(&without_login(&fixture.document(A)?))?,
    )?;
    let before = fixture.bytes()?;
    let refused = fixture
      .run(&a_to_b(), Some("ACCESS_A"))
      .err()
      .ok_or("expected the move to be refused as changed")?;
    assert_eq!(refused.downcast_ref::<Refusal>(), Some(&Refusal::Changed));
    assert_eq!(fixture.bytes()?, before);
    assert!(fixture.journal());
    Ok(())
  }

  // ST0003 AT-02.6: each refusal on file stores comes before any write, journal or
  // selection, and its message carries no token.
  #[test]
  fn each_refusal_comes_before_any_write() -> Result<()> {
    let [shared, a, b] = seated_a()?;
    let only_b = Move {
      to: Some(B.into()),
      ..Move::default()
    };
    let cases = [
      (
        [shared.clone(), a.clone(), without_login(&b)],
        a_to_b(),
        Some("ACCESS_A"),
        Refusal::NothingToSeat(B.into()),
      ),
      // A copy of the shared login in a's store is a park that landed, not this.
      (
        [
          shared.clone(),
          with_login(&a, &json!({"accessToken": "ACCESS_OLD"})),
          b.clone(),
        ],
        a_to_b(),
        Some("ACCESS_A"),
        Refusal::Occupied(A.into()),
      ),
      (
        [shared.clone(), a.clone(), b.clone()],
        a_to_b(),
        Some("ACCESS_B"),
        Refusal::Unverified(A.into()),
      ),
      (
        [shared.clone(), a.clone(), b.clone()],
        a_to_b(),
        None,
        Refusal::Unverified(A.into()),
      ),
      (
        [without_login(&shared), a.clone(), b.clone()],
        a_to_b(),
        Some("ACCESS_A"),
        Refusal::SharedEmpty(A.into()),
      ),
      (
        [shared.clone(), a.clone(), b.clone()],
        only_b,
        None,
        Refusal::OnlyLogin,
      ),
    ];
    for (documents, request, verified, refusal) in cases {
      let fixture = Fixture::new(documents)?;
      let before = fixture.bytes()?;
      let error = fixture
        .run(&request, verified)
        .err()
        .ok_or_else(|| format!("expected {refusal:?}"))?;
      assert_eq!(error.downcast_ref::<Refusal>(), Some(&refusal));
      assert!(!error.to_string().contains("ACCESS_"));
      assert_eq!(fixture.bytes()?, before, "{refusal:?}");
      assert!(!fixture.journal(), "{refusal:?}");
      assert_eq!(fixture.store.active()?, None, "{refusal:?}");
      assert_eq!(Seat::load(&fixture.store)?, Seat::default(), "{refusal:?}");
    }
    Ok(())
  }

  // ST0003 AT-02.7: the only-login refusal gives way to a move that says replace, which
  // WP03's adoption decides.
  #[test]
  fn a_shared_login_no_profile_is_seated_with_is_replaced_only_on_request() -> Result<()> {
    // 0.1.8's layout: every profile holds its own login, and the shared config holds
    // one of its own.
    let [shared, a, b] = seated_a()?;
    let fixture = Fixture::new([shared, with_login(&a, &login_a()), b])?;
    let request = Move {
      to: Some(B.into()),
      replace: true,
      ..Move::default()
    };
    fixture.run(&request, None)?;
    assert_eq!(fixture.holders(&login_a())?, [A]);
    assert_eq!(fixture.holders(&login_b())?, [SHARED]);
    Ok(())
  }

  #[test]
  fn each_store_is_found_where_claude_code_keeps_it_and_a_missing_one_says_how_to_make_it()
  -> Result<()> {
    let fixture = Fixture::new(seated_a()?)?;
    let found = places(&fixture.store, &a_to_b())?;
    assert_eq!(found.shared.dir, fixture.root.join(SHARED));
    for (place, name) in [(found.from, A), (found.to, B)] {
      let place = place.ok_or("a named profile has no place")?;
      assert_eq!(place.dir, fixture.root.join(name));
      // A file store on any platform here: no Keychain item is named after a fixture.
      assert!(matches!(place.location, Location::File(ref path) if *path == fixture.file(name)));
    }
    fs::remove_file(fixture.file(B))?;
    let missing = places(&fixture.store, &a_to_b())
      .err()
      .ok_or("expected b's missing store to be refused")?;
    assert!(
      missing.to_string().contains("run ccusage login b"),
      "{missing}"
    );
    Ok(())
  }

  // ST0003 AT-03.1: a login is read from the shared store while its profile is seated,
  // from the profile's own store otherwise and with shared mode off, and neither of a
  // move's profiles is read while the move is unfinished.
  #[test]
  fn a_login_is_read_where_it_lives() -> Result<()> {
    let mut fixture = Fixture::new(seated_a()?)?;
    Seat {
      seated: Some(A.into()),
    }
    .save(&fixture.store)?;
    let shared = fixture.root.join(SHARED);
    assert_eq!(home(&fixture.store, A)?, shared);
    assert_eq!(home(&fixture.store, B)?, fixture.root.join(B));
    fixture.run(&a_to_b(), Some("ACCESS_A"))?;
    assert_eq!(home(&fixture.store, A)?, fixture.root.join(A));
    assert_eq!(home(&fixture.store, B)?, shared);
    let back = Move {
      from: Some(B.into()),
      to: Some(A.into()),
      replace: false,
    };
    let stopped = fixture.until(&back, Some("ACCESS_B"), &mut |point| match point {
      Point::Wrote(Step::Parked) => Err(crate::Error::from("interrupted")),
      _ => Ok(()),
    });
    assert!(stopped.is_err());
    for name in [A, B] {
      let refused = home(&fixture.store, name)
        .err()
        .ok_or("expected the unfinished move to refuse its profiles")?;
      assert!(refused.to_string().contains("ccusage use a"), "{refused}");
    }
    fixture.run(&back, Some("ACCESS_B"))?;
    assert_eq!(home(&fixture.store, A)?, shared);
    fixture.store.config.claude_config_dir = None;
    assert_eq!(home(&fixture.store, A)?, fixture.root.join(A));
    Ok(())
  }

  // ST0003 AT-02.8: a move waits while Claude Code holds a refresh or write lock.
  #[test]
  fn a_move_waits_for_a_lock_claude_code_holds() -> Result<()> {
    for (store, lock) in [
      (SHARED, ".oauth_refresh.lock"),
      (SHARED, ".storage-write.lock"),
      (B, ".oauth_refresh.lock"),
    ] {
      let fixture = Fixture::new(seated_a()?)?;
      let before = fixture.bytes()?;
      let held = Lock::acquire(fixture.root.join(store).join(lock), Duration::from_secs(60))?;
      let (finished, done) = std::sync::mpsc::channel();
      std::thread::scope(|scope| -> Result<()> {
        scope.spawn(|| {
          let moved = fixture
            .run(&a_to_b(), Some("ACCESS_A"))
            .map_err(|error| error.to_string());
          let _ = finished.send(moved);
        });
        assert!(
          done.recv_timeout(Duration::from_millis(500)).is_err(),
          "the move went ahead while {store}'s {lock} was held"
        );
        assert_eq!(fixture.bytes()?, before);
        drop(held);
        done
          .recv_timeout(Duration::from_secs(10))
          .map_err(|_| "the move did not finish once the lock was free")??;
        Ok(())
      })?;
      assert_eq!(fixture.holders(&login_b())?, [SHARED]);
    }
    Ok(())
  }

  // ST0003 AT-02.9: on a disposable keychain, a document over the security -i line is
  // refused before any write, and a move creates no item and keeps apple-tool: on each
  // item it writes.
  /// On a disposable keychain, as Claude Code keeps its items: a move writes only
  /// through `security`, so every item it writes keeps `apple-tool:`, and it creates
  /// no item. A document too long for one `security -i` line is refused before any
  /// write.
  #[cfg(target_os = "macos")]
  #[test]
  #[ignore = "requires the macOS Security service; uses only a disposable keychain"]
  fn a_move_keeps_claude_items_readable_by_security() -> Result<()> {
    use crate::keychain::native;
    use security_framework::os::macos::keychain::SecKeychain;
    let _no_prompts = SecKeychain::disable_user_interaction()?;
    let keychain = native::Keychain::new()?;
    let account = "ccusage-fixture";
    let documents = seated_a()?;
    let fixture = Fixture::new(documents.clone())?;
    let service = |name: &str| keychain.name(name);
    for (name, document) in [SHARED, A, B].into_iter().zip(&documents) {
      keychain.add_as_claude_code(&service(name), account, &serde_json::to_vec(document)?)?;
    }
    let created = keychain.services()?;
    let place = |name: &str| Place {
      dir: fixture.root.join(name),
      location: Location::Keychain {
        service: service(name),
        account: account.into(),
        keychain: Some(keychain.path.clone()),
      },
    };
    let places = |request: &Move| Places {
      shared: place(SHARED),
      from: request.from.as_deref().map(place),
      to: request.to.as_deref().map(place),
    };
    let read = |name: &str| -> Result<Document> {
      Ok(serde_json::from_slice(
        &keychain.read_as_claude_code(&service(name), account)?,
      )?)
    };
    let items = || -> Result<Vec<Vec<u8>>> {
      [SHARED, A, B]
        .into_iter()
        .map(|name| keychain.read_as_claude_code(&service(name), account))
        .collect()
    };
    let shared = place(SHARED);
    // Pad the shared document until, with a's login, it only just fits one security -i
    // line. With b's longer login in a's place it then cannot fit, so the move is
    // refused before it writes anything.
    let padded = |padding: usize| -> Result<Vec<u8>> {
      let mut document = documents[0].clone();
      document.insert("padding".into(), "x".repeat(padding).into());
      Ok(serde_json::to_vec(&document)?)
    };
    let fits =
      |padding: usize| -> Result<bool> { Ok(shared.location.check(&padded(padding)?).is_ok()) };
    assert!(
      fits(0)?,
      "the unpadded shared document must fit for the probe to mean anything"
    );
    let (mut low, mut high) = (0, 1);
    while fits(high)? {
      (low, high) = (high, high * 2);
    }
    while high - low > 1 {
      let middle = low + (high - low) / 2;
      if fits(middle)? {
        low = middle;
      } else {
        high = middle;
      }
    }
    shared.location.write(&padded(low)?)?;
    let before = items()?;
    let refused = run(
      &fixture.store,
      &a_to_b(),
      &places(&a_to_b()),
      Some("ACCESS_A"),
    );
    assert!(refused.is_err_and(|error| error.to_string().contains("too large")));
    assert_eq!(items()?, before);
    assert!(!fixture.journal());
    // Unpadded, the same move goes ahead.
    shared.location.write(&serde_json::to_vec(&documents[0])?)?;
    run(
      &fixture.store,
      &a_to_b(),
      &places(&a_to_b()),
      Some("ACCESS_A"),
    )?;
    assert_eq!(super::login(&read(A)?), Some(&login_a()));
    assert_eq!(super::login(&read(SHARED)?), Some(&login_b()));
    assert_eq!(super::login(&read(B)?), None);
    for name in [SHARED, A, B] {
      let access = keychain.access(&service(name))?;
      assert!(
        access.partitions.iter().any(|id| id == "apple-tool:"),
        "{name}'s partitions after the move: {:?}",
        access.partitions
      );
      assert!(
        access.decrypt.iter().any(|app| app == "/usr/bin/security"),
        "{name}'s decrypt access after the move: {:?}",
        access.decrypt
      );
      // The item is in the disposable keychain alone: no write fell back to another.
      assert!(!native::anywhere(&service(name), account)?);
    }
    assert_eq!(keychain.services()?, created);
    Ok(())
  }
}
