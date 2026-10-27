#!/bin/sh
# Real local Git pushes; mocked GitHub/Cargo validation. No external publication.
set -eu
root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
real_cargo=$(command -v cargo)
original_path=$PATH
sandbox=$(mktemp -d "${TMPDIR:-/tmp}/ccusage-release-tests.XXXXXX")
trap 'rm -rf "$sandbox"' 0 HUP INT TERM
count=0

fail() { echo "FAIL: $*" >&2; exit 1; }
assert_contains() { grep -F "$2" "$1" >/dev/null || fail "missing '$2' in $1"; }
configure_git() {
  git -C "$1" config user.name 'Release test'
  git -C "$1" config user.email 'release-test@example.invalid'
  git -C "$1" config commit.gpgsign false
  git -C "$1" config tag.gpgsign false
  git -C "$1" config core.hooksPath "$sandbox/empty-hooks"
}

setup() {
  count=$((count + 1))
  case_dir=$sandbox/case-$count
  source=$case_dir/Ccusage
  tap=$case_dir/homebrew-ccusage
  source_remote=$case_dir/source.git
  tap_remote=$case_dir/tap.git
  mkdir -p "$source/tools" "$source/src" "$source/tests" "$source/.github/workflows" \
    "$source/packaging/homebrew" "$tap" "$case_dir/bin" "$sandbox/empty-hooks"
  cp "$root/tools/release" "$root/tools/formula" "$source/tools/"
  cp "$root/packaging/homebrew/ccusage.rb" "$source/packaging/homebrew/"
  printf 'exit 0\n' > "$source/tests/release.sh"
  printf 'name: fixture\n' > "$source/.github/workflows/build.yml"
  printf '/target/\n' > "$source/.gitignore"
  cat > "$source/Cargo.toml" <<'TOML'
[package]
name = "ccusage"
version = "0.1.0"
edition = "2024"
TOML
  printf 'fn main() {}\n' > "$source/src/main.rs"
  (cd "$source" && "$real_cargo" generate-lockfile --offline --quiet)
  git init -q --bare --initial-branch=main "$source_remote"
  git init -q --initial-branch=main "$source"
  configure_git "$source"
  git -C "$source" add .
  git -C "$source" commit -qm initial
  git -C "$source" config "url.$source_remote.insteadOf" https://github.com/matthewsinclair/ccusage.git
  git -C "$source" remote add upstream https://github.com/matthewsinclair/ccusage.git
  git -C "$source" push -qu upstream main
  git init -q --bare --initial-branch=main "$tap_remote"
  git init -q --initial-branch=main "$tap"
  configure_git "$tap"
  printf 'Tap fixture\n' > "$tap/README.md"
  git -C "$tap" add .
  git -C "$tap" commit -qm initial
  git -C "$tap" remote add origin "$tap_remote"
  git -C "$tap" push -qu origin main
  initial_source=$(git -C "$source" rev-parse HEAD)
  initial_tap=$(git -C "$tap" rev-parse HEAD)
  cat > "$case_dir/bin/cargo" <<'MOCK'
#!/bin/sh
set -eu
echo "cargo $*" >> "$FIXTURE_LOG"
if [ "$1" = update ]; then exec "$REAL_CARGO" "$@"; fi
if [ "${FAIL_CARGO:-}" = "$1" ]; then exit 7; fi
exit 0
MOCK
  cat > "$case_dir/bin/gh" <<'MOCK'
#!/bin/sh
set -eu
echo "gh $*" >> "$FIXTURE_LOG"
case "$1 $2" in
  'api repos/matthewsinclair/ccusage'|'api repos/matthewsinclair/homebrew-ccusage') echo true ;;
  'repo view') echo matthewsinclair/homebrew-ccusage ;;
  'api repos/matthewsinclair/ccusage/commits/'*)
    tag=${2##*/}
    git -C "$FIXTURE_SOURCE" rev-parse "refs/tags/$tag^{commit}" ;;
  'run list') echo 123 ;;
  'run watch')
    case "$*" in
      *'--repo matthewsinclair/homebrew-ccusage'*) [ "${FAIL_TAP_CI:-}" != yes ] ;;
      *) [ "${FAIL_CI:-}" != yes ] ;;
    esac ;;

  'release view')
    case "$*" in
      *'--json url'*) echo "https://github.com/matthewsinclair/ccusage/releases/tag/$3" ;;
      *)
        for target in aarch64-apple-darwin x86_64-apple-darwin aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
          echo "ccusage-$3-$target.tar.gz"
        done
        if [ "${MISSING_CHECKSUM:-}" != yes ]; then echo SHA256SUMS; fi ;;
    esac ;;
  *) echo "Unexpected gh invocation" >&2; exit 2 ;;
esac
MOCK
  printf '#!/bin/sh\nexit 0\n' > "$case_dir/bin/actionlint"
  cp "$case_dir/bin/actionlint" "$case_dir/bin/shellcheck"
  chmod +x "$case_dir/bin/"* "$source/tools/"*
  FIXTURE_SOURCE=$source
  FIXTURE_LOG=$case_dir/events
  REAL_CARGO=$real_cargo
  PATH=$case_dir/bin:$original_path
  export FIXTURE_SOURCE FIXTURE_LOG REAL_CARGO PATH
  unset FAIL_CARGO FAIL_CI FAIL_TAP_CI MISSING_CHECKSUM CCUSAGE_REMOTE CCUSAGE_TAP
}

release() { sh "$source/tools/release" "$@" --tap "$tap" > "$case_dir/output" 2>&1; }
no_tag() {
  if git --git-dir="$source_remote" show-ref --verify --quiet refs/tags/v0.1.1; then fail 'unexpected published tag'; fi
}

setup
release --dry-run 0.1.1 || { cat "$case_dir/output"; fail 'dry run'; }
[ "$(git -C "$source" rev-parse HEAD)" = "$initial_source" ] || fail 'dry-run commit'
[ -z "$(git -C "$source" status --porcelain)" ] || fail 'dry-run edits'
[ ! -e "$FIXTURE_LOG" ] || fail 'dry-run invoked Cargo or GitHub'
no_tag
assert_contains "$case_dir/output" 'No files, refs, or remote state changed'
echo 'ok: dry-run is read-only'

setup
printf 'uncommitted\n' >> "$source/src/main.rs"
if release 0.1.1; then fail 'dirty tree accepted'; fi
no_tag
assert_contains "$case_dir/output" 'must be clean'
echo 'ok: dirty trees are rejected'

setup
for version in 0.1.0 0.0.9 01.1.1 v0.1.1 0.1.1-rc.1; do
  if release --dry-run "$version"; then fail "invalid version accepted: $version"; fi
done
[ -z "$(git -C "$source" status --porcelain)" ] || fail 'invalid version edited files'
echo 'ok: stable increasing versions required'

setup
release 0.1.1 || { cat "$case_dir/output"; fail 'release'; }
assert_contains "$source/Cargo.toml" 'version = "0.1.1"'
assert_contains "$source/Cargo.lock" 'version = "0.1.1"'
release_commit=$(git -C "$source" rev-parse HEAD)
[ "$(git --git-dir="$source_remote" rev-parse 'refs/tags/v0.1.1^{commit}')" = "$release_commit" ] || fail 'source tag not pushed'
assert_contains "$tap/Formula/ccusage.rb" "$release_commit"
assert_contains "$tap/Formula/ccusage.rb" 'v0.1.1'
[ "$(git --git-dir="$tap_remote" rev-parse main)" = "$(git -C "$tap" rev-parse HEAD)" ] || fail 'tap not pushed'
[ -z "$(git -C "$source" status --porcelain)" ] || fail 'release left source dirty'
assert_contains "$FIXTURE_LOG" 'cargo clippy --all-targets --locked -- -D warnings'
assert_contains "$FIXTURE_LOG" 'gh run watch 123'
echo 'ok: version bump, annotated tag, release wait, and pinned tap publication'

setup
FAIL_CI=yes
export FAIL_CI
if release 0.1.1; then fail 'failed CI accepted'; fi
[ "$(git --git-dir="$tap_remote" rev-parse main)" = "$initial_tap" ] || fail 'tap updated despite failed CI'
release_commit=$(git -C "$source" rev-parse HEAD)
unset FAIL_CI
release --resume 0.1.1 || { cat "$case_dir/output"; fail 'resume after CI'; }
[ "$(git -C "$source" rev-parse HEAD)" = "$release_commit" ] || fail 'resume created another source commit'
echo 'ok: CI failure blocks the tap; resume preserves the release commit'

setup
FAIL_CARGO=clippy
export FAIL_CARGO
if release 0.1.1; then fail 'failed local validation accepted'; fi
no_tag
[ "$(git -C "$source" rev-parse HEAD)" = "$initial_source" ] || fail 'commit despite failed validation'
assert_contains "$case_dir/output" 'stopped during validation'
echo 'ok: local validation failure creates no commit or tag'

setup
MISSING_CHECKSUM=yes
export MISSING_CHECKSUM
if release 0.1.1; then fail 'missing checksums accepted'; fi
[ "$(git --git-dir="$tap_remote" rev-parse main)" = "$initial_tap" ] || fail 'tap updated without checksums'
echo 'ok: incomplete release assets block tap publication'

setup
git --git-dir="$tap_remote" config core.hooksPath "$tap_remote/hooks"
printf '#!/bin/sh\nexit 1\n' > "$tap_remote/hooks/pre-receive"
chmod +x "$tap_remote/hooks/pre-receive"
if release 0.1.1; then fail 'rejected tap push accepted'; fi
pending_tap=$(git -C "$tap" rev-parse HEAD)
[ "$pending_tap" != "$initial_tap" ] || fail 'no pending tap commit'
rm "$tap_remote/hooks/pre-receive"
release --resume 0.1.1 || { cat "$case_dir/output"; fail 'resume tap push'; }
[ "$(git -C "$tap" rev-parse HEAD)" = "$pending_tap" ] || fail 'resume duplicated tap commit'
[ "$(git --git-dir="$tap_remote" rev-parse main)" = "$pending_tap" ] || fail 'pending tap commit not pushed'
echo 'ok: resume retries an already-committed tap update'

setup
git -C "$source" switch -qc feature
if release 0.1.1; then fail 'non-main release accepted'; fi
no_tag
echo 'ok: source releases require main'

setup
printf 'uncommitted\n' >> "$tap/README.md"
if release 0.1.1; then fail 'dirty tap accepted'; fi
no_tag
echo 'ok: dirty tap rejected before source publication'

setup
git --git-dir="$source_remote" config core.hooksPath "$source_remote/hooks"
printf '#!/bin/sh\nexit 1\n' > "$source_remote/hooks/pre-receive"
chmod +x "$source_remote/hooks/pre-receive"
if release 0.1.1; then fail 'rejected atomic source push accepted'; fi
no_tag
[ "$(git --git-dir="$source_remote" rev-parse main)" = "$initial_source" ] || fail 'partial atomic push'
release_commit=$(git -C "$source" rev-parse HEAD)
rm "$source_remote/hooks/pre-receive"
release --resume 0.1.1 || { cat "$case_dir/output"; fail 'resume source push'; }
[ "$(git -C "$source" rev-parse HEAD)" = "$release_commit" ] || fail 'resume duplicated release commit'
echo 'ok: atomic push rejection leaves remote unchanged; resume retries safely'

setup
git clone -q "$source_remote" "$case_dir/other"
configure_git "$case_dir/other"
printf 'remote update\n' > "$case_dir/other/README.md"
git -C "$case_dir/other" add README.md
git -C "$case_dir/other" commit -qm 'remote update'
git -C "$case_dir/other" push -q origin main
if release 0.1.1; then fail 'behind source branch accepted'; fi
assert_contains "$case_dir/output" 'behind or diverged'
assert_contains "$source/Cargo.toml" 'version = "0.1.0"'
no_tag
echo 'ok: behind main rejected without a version bump'

setup
mkdir "$source/.git/ccusage-release.lock"
if release 0.1.1; then fail 'concurrent release accepted'; fi
[ -d "$source/.git/ccusage-release.lock" ] || fail 'another release lock removed'
no_tag
echo 'ok: concurrent releases rejected without removing an existing lock'

setup
git -C "$source" remote rename upstream origin
release --dry-run 0.1.1 || { cat "$case_dir/output"; fail 'origin default'; }
assert_contains "$case_dir/output" 'Source remote: origin'
echo 'ok: origin is used when upstream is absent'

setup
FAIL_TAP_CI=yes
export FAIL_TAP_CI
if release 0.1.1; then fail 'failed Homebrew CI accepted'; fi
assert_contains "$case_dir/output" 'stopped during Homebrew CI'
release_commit=$(git -C "$source" rev-parse HEAD)
released_tap=$(git -C "$tap" rev-parse HEAD)
unset FAIL_TAP_CI
release --resume 0.1.1 || { cat "$case_dir/output"; fail 'resume after tap CI'; }
[ "$(git -C "$source" rev-parse HEAD)" = "$release_commit" ] || fail 'tap CI resume duplicated source commit'
[ "$(git -C "$tap" rev-parse HEAD)" = "$released_tap" ] || fail 'tap CI resume duplicated tap commit'
echo 'ok: Homebrew CI failure is reported and can be resumed without new commits'

echo '15 release integration scenarios passed; no GitHub requests were made.'


