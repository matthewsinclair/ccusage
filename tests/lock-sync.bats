#!/usr/bin/env bats
# bin/devbin lock-sync, the release's derived: step, on a crate of its own: a version
# stamped into Cargo.toml reaches Cargo.lock and the path is printed, and a change to
# anything else in the lock puts the old lock back and refuses.

setup() {
  ROOT=$(cd "$BATS_TEST_DIRNAME/.." && pwd)
  CRATE=$BATS_TEST_TMPDIR/crate
  mkdir -p "$CRATE/src" "$CRATE/bin/.devbin/cmd"
  cp "$ROOT/bin/.devbin/cmd/lock-sync" "$CRATE/bin/.devbin/cmd/"
  printf '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2024"\n' > "$CRATE/Cargo.toml"
  printf 'fn main() {}\n' > "$CRATE/src/main.rs"
  (cd "$CRATE" && cargo generate-lockfile --offline --quiet)
}

@test "a stamped version reaches Cargo.lock, and lock-sync prints the path it wrote" {
  sed -i '' 's/^version = "0.1.0"$/version = "0.2.0"/' "$CRATE/Cargo.toml"
  run "$CRATE/bin/.devbin/cmd/lock-sync"
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [ "$output" = Cargo.lock ]
  grep -qx 'version = "0.2.0"' "$CRATE/Cargo.lock"
}

@test "a lock change beyond the crate's own version is put back and refused" {
  local before
  before=$(cat "$CRATE/Cargo.lock")
  # A second sourceless package moves more than the crate's own version line
  mkdir -p "$CRATE/extra/src"
  printf '[package]\nname = "extra"\nversion = "0.1.0"\nedition = "2024"\n' > "$CRATE/extra/Cargo.toml"
  printf '' > "$CRATE/extra/src/lib.rs"
  printf '\n[dependencies]\nextra = { path = "extra" }\n' >> "$CRATE/Cargo.toml"
  run "$CRATE/bin/.devbin/cmd/lock-sync"
  [ "$status" -ne 0 ]
  [[ $output == *"changed more than ccusage's own version"* ]] || { echo "$output"; return 1; }
  [ "$(cat "$CRATE/Cargo.lock")" = "$before" ]
}

@test "lock-sync takes no arguments" {
  run "$CRATE/bin/.devbin/cmd/lock-sync" extra
  [ "$status" -ne 0 ]
  [[ $output == *"takes no arguments"* ]]
}
