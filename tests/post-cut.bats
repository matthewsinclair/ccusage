#!/usr/bin/env bats
# bin/devbin post-cut, the cut's one after: line, in a repository of its own: a copy of
# the command beside the real config.yaml, a stand-in bin/devbin that logs each step it
# is asked for and the yes it was handed, annotated release tags, and a stand-in brew
# whose repository holds a clone of a local tap. Nothing here signs, publishes or
# pushes for real, and this Mac's own tap is never touched.

setup() {
  ROOT=$(cd "$BATS_TEST_DIRNAME/.." && pwd)
  REPO=$BATS_TEST_TMPDIR/repo
  FAKE_BIN=$BATS_TEST_TMPDIR/bin
  FAKE_LOG=$BATS_TEST_TMPDIR/log
  FAKE_BREW=$BATS_TEST_TMPDIR/homebrew
  # Where brew keeps its clone of config.yaml's formula tap, matthewsinclair/homebrew-ccusage
  TAP_CLONE=$FAKE_BREW/Library/Taps/matthewsinclair/homebrew-ccusage
  TAP_REMOTE=$BATS_TEST_TMPDIR/tap.git
  TAP_WORK=$BATS_TEST_TMPDIR/tap-work
  mkdir -p "$REPO/bin/.devbin/cmd" "$FAKE_BIN"
  : > "$FAKE_LOG"
  cp "$ROOT/bin/.devbin/cmd/post-cut" "$REPO/bin/.devbin/cmd/"
  cp "$ROOT/bin/.devbin/config.yaml" "$REPO/bin/.devbin/"
  printf '[package]\nname = "ccusage"\nversion = "0.1.7"\n' > "$REPO/Cargo.toml"
  write_fakes
  git init -q -b main "$REPO"
  git_in "$REPO" add -A
  git_in "$REPO" commit -q -m "release: v0.1.6"
  git_in "$REPO" tag -a v0.1.6 -m v0.1.6
  git_in "$REPO" commit -q --allow-empty -m "release: v0.1.7"
  git_in "$REPO" tag -a v0.1.7 -m v0.1.7
  make_tap
  unset DEVBIN_RELEASE_CONFIRMED FAKE_FAILS
  export FAKE_LOG FAKE_BREW PATH="$FAKE_BIN:$PATH"
}

# git_in <dir> <git arguments>: git there, with a fixture identity and no signing
git_in() {
  local dir=$1
  shift
  git -C "$dir" -c user.name=fixture -c user.email=fixture@example.invalid -c commit.gpgsign=false \
    -c tag.gpgsign=false "$@"
}

write_fakes() {
  # bin/devbin, standing in for each step: it logs the step and the yes it was handed,
  # and fails the step FAKE_FAILS names
  cat > "$REPO/bin/devbin" <<'FAKE'
#!/bin/sh
echo "devbin $* yes=${DEVBIN_RELEASE_CONFIRMED:-}" >> "$FAKE_LOG"
case "$*" in "${FAKE_FAILS:-none}"*) exit 1 ;; esac
FAKE
  cat > "$FAKE_BIN/brew" <<'FAKE'
#!/bin/sh
echo "brew $*" >> "$FAKE_LOG"
[ "$*" = --repository ] || { echo "the fake brew has no answer for $*" >&2; exit 2; }
echo "$FAKE_BREW"
FAKE
  chmod +x "$REPO/bin/devbin" "$FAKE_BIN/brew"
}

# The tap: a bare repository standing in for GitHub's, brew's clone of it, and a
# publish since that clone was made, which a pull brings in
make_tap() {
  git init -q --bare -b main "$TAP_REMOTE"
  git init -q -b main "$TAP_WORK"
  git_in "$TAP_WORK" commit -q --allow-empty -m "ccusage 0.1.6"
  git_in "$TAP_WORK" push -q "$TAP_REMOTE" main
  mkdir -p "$(dirname "$TAP_CLONE")"
  git clone -q "$TAP_REMOTE" "$TAP_CLONE"
  git_in "$TAP_WORK" commit -q --allow-empty -m "ccusage 0.1.7"
  git_in "$TAP_WORK" push -q "$TAP_REMOTE" main
}

post_cut() { run "$REPO/bin/.devbin/cmd/post-cut" "$@"; }

# failed <message>: the run failed, saying so
failed() {
  [ "$status" -ne 0 ] || { echo "it succeeded: $output"; return 1; }
  [[ $output == *"$1"* ]] || { echo "output: $output"; return 1; }
}

# unpulled: brew's clone of the tap is where it was made
unpulled() {
  [ "$(git -C "$TAP_CLONE" log -1 --format=%s)" = "ccusage 0.1.6" ] || { echo "the tap clone was pulled"; return 1; }
}

# untouched: no step ran, and the tap clone was not pulled
untouched() {
  if grep -q '^devbin ' "$FAKE_LOG"; then
    echo "the log has: $(grep '^devbin ' "$FAKE_LOG")"
    return 1
  fi
  unpulled
}

@test "finalize, then formula publish, under the cut's yes, then this Mac's tap is pulled" {
  DEVBIN_RELEASE_CONFIRMED=v0.1.7 post_cut
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [ "$(grep '^devbin ' "$FAKE_LOG")" = "$(printf '%s\n' 'devbin macos finalize v0.1.7 yes=v0.1.7' 'devbin formula publish yes=v0.1.7')" ]
  [ "$(git -C "$TAP_CLONE" rev-parse HEAD)" = "$(git -C "$TAP_REMOTE" rev-parse main)" ]
  [[ $output == *"this Mac's tap: ccusage 0.1.7"* ]]
  [[ $output == *"ccusage v0.1.7: signed, notarized and public, and the tap published"* ]]
}

@test "with no tag named and no cut, the tag is v and Cargo.toml's version, and publish is handed no yes" {
  post_cut
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [ "$(grep '^devbin ' "$FAKE_LOG")" = "$(printf '%s\n' 'devbin macos finalize v0.1.7 yes=' 'devbin formula publish yes=')" ]
}

@test "the tag named comes before the cut's, and the cut's before Cargo.toml's" {
  # An older tag in each place is refused, which shows that place was the one read
  DEVBIN_RELEASE_CONFIRMED=v0.1.6 post_cut
  failed "'v0.1.6' is not the newest release (v0.1.7)"
  DEVBIN_RELEASE_CONFIRMED=v0.1.7 post_cut v0.1.6
  failed "'v0.1.6' is not the newest release (v0.1.7)"
  untouched
}

@test "a tag that is not an annotated tag here is refused, and nothing runs" {
  git_in "$REPO" tag v0.1.8
  post_cut v0.1.8
  failed "'v0.1.8' is not an annotated tag here"
  post_cut v9.9.9
  failed "'v9.9.9' is not an annotated tag here"
  untouched
}

@test "a tag that is not the newest release is refused, since formula publish publishes the newest" {
  post_cut v0.1.6
  failed "'v0.1.6' is not the newest release (v0.1.7)"
  untouched
}

@test "a failed finalize stops the run there, and names the command that resumes it" {
  FAKE_FAILS="macos finalize" post_cut
  failed "run bin/devbin post-cut v0.1.7"
  [ "$(grep '^devbin ' "$FAKE_LOG")" = "devbin macos finalize v0.1.7 yes=" ]
  unpulled
}

@test "a failed formula publish stops the run there, and names the command that resumes it" {
  FAKE_FAILS="formula publish" post_cut
  failed "run bin/devbin post-cut v0.1.7"
  grep -q '^devbin formula publish' "$FAKE_LOG"
  unpulled
}

@test "a tap clone that cannot be pulled is named, after the publish" {
  # A commit of its own: the clone has diverged, so a fast-forward pull refuses
  git_in "$TAP_CLONE" commit -q --allow-empty -m "local"
  post_cut
  failed "cannot pull $TAP_CLONE"
  grep -q '^devbin formula publish' "$FAKE_LOG"
}

@test "with no clone of the tap here, the pull is skipped" {
  rm -rf "$TAP_CLONE"
  post_cut
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [[ $output != *"this Mac's tap"* ]]
}

@test "it refuses off macOS, and more than one argument" {
  local other=$BATS_TEST_TMPDIR/other
  mkdir -p "$other"
  printf '#!/bin/sh\necho Linux\n' > "$other/uname"
  chmod +x "$other/uname"
  PATH="$other:$PATH" post_cut
  failed "on macOS only; this is Linux"
  post_cut v0.1.7 extra
  failed "usage: bin/devbin post-cut [<tag>]"
  untouched
}

@test "help prints the usage block" {
  post_cut --help
  [ "$status" -eq 0 ]
  [[ $output == *"bin/devbin post-cut [<tag>]"* ]]
  [[ $output != *"In order"* ]]
}

@test "the cut's one after: line is bin/devbin post-cut" {
  local after
  after=$(awk '/^  after:$/ { on = 1; next } on && /^    - / { print substr($0, 7); next } on { exit }' "$ROOT/bin/.devbin/config.yaml")
  [ "$after" = "bin/devbin post-cut" ]
}
