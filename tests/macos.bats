#!/usr/bin/env bats
# bin/devbin macos finalize against a stand-in Mac and GitHub. The fake gh serves a
# release from a directory, the fake codesign signs by appending a marker and shows
# the v0.1.5 binary's signature, the fake lipo answers as lipo does for each target's
# binary, the fake security holds the identity and fixture notary credentials, and
# the fake xcrun answers as notarytool does. Nothing here signs, notarizes, uploads or
# publishes for real, and no real Keychain item is read.

TAG=v0.1.4
TARGETS="aarch64-apple-darwin x86_64-apple-darwin aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu"
MARKER='signed by the fake codesign'

setup() {
  ROOT=$(cd "$BATS_TEST_DIRNAME/.." && pwd)
  HANDLER=$ROOT/bin/.devbin/cmd/macos
  FAKE_BIN=$BATS_TEST_TMPDIR/bin
  FAKE_RELEASE=$BATS_TEST_TMPDIR/release
  FAKE_LOG=$BATS_TEST_TMPDIR/log
  FAKE_DISPLAY_OUT=$ROOT/tests/fixtures/codesign-v0.1.5.stdout
  FAKE_DISPLAY_ERR=$ROOT/tests/fixtures/codesign-v0.1.5.stderr
  # How many signatures the fake codesign has displayed
  FAKE_DISPLAYS=$BATS_TEST_TMPDIR/displays
  # The notary credentials' prefix, and the team the fake credentials name: the signing team
  KEYCHAIN=$(sed -n 's/^keychain=//p' "$ROOT/packaging/macos/signing")
  FAKE_NOTARY_TEAM=$(sed -n 's/^team=//p' "$ROOT/packaging/macos/signing")
  # The release's repository, as config.yaml's release.repo names it
  REPO_NAME=$(sed -n 's/^  repo: //p' "$ROOT/bin/.devbin/config.yaml" | head -1)
  mkdir -p "$FAKE_BIN" "$FAKE_RELEASE"
  : > "$FAKE_LOG"
  write_fakes
  ci_release
  unset CI CCUSAGE_MACOS_MEMBER_LIMIT
  export FAKE_RELEASE FAKE_LOG FAKE_DISPLAY_OUT FAKE_DISPLAY_ERR FAKE_DISPLAYS KEYCHAIN FAKE_NOTARY_TEAM PATH="$FAKE_BIN:$PATH"
}

# How many Mac archives a release signs, one signature display each
macs() { echo "$TARGETS" | tr ' ' '\n' | grep -c apple-darwin; }

write_fakes() {
  cat > "$FAKE_BIN/gh" <<'FAKE'
#!/bin/sh
set -f
echo "gh $*" >> "$FAKE_LOG"
verb="$1 $2"
shift 2
case "$verb" in
  'release view') echo "${FAKE_DRAFT:-true}" ;;
  'release download')
    shift
    dir= patterns=
    while [ $# -gt 0 ]; do
      case "$1" in
        --repo) shift ;;
        --dir) shift; dir=$1 ;;
        --pattern) shift; patterns="$patterns $1" ;;
      esac
      shift
    done
    mkdir -p "$dir"
    # set -f keeps each --pattern a pattern, so the assets are listed rather than globbed
    for name in $(ls "$FAKE_RELEASE"); do
      if [ -z "$patterns" ]; then cp "$FAKE_RELEASE/$name" "$dir/"; continue; fi
      for pattern in $patterns; do
        case "$name" in $pattern) cp "$FAKE_RELEASE/$name" "$dir/" ;; esac
      done
    done ;;
  'release upload')
    shift
    while [ $# -gt 0 ]; do
      case "$1" in --repo) shift ;; --clobber) ;; *) cp "$1" "$FAKE_RELEASE/" ;; esac
      shift
    done
    [ -z "${FAKE_AFTER_UPLOAD:-}" ] || sh -c "$FAKE_AFTER_UPLOAD" ;;
  'release delete-asset') rm "$FAKE_RELEASE/$2" ;;
  'release edit') [ "${FAKE_EDIT_FAILS:-}" != yes ] ;;
  *) echo "the fake gh has no answer for $verb" >&2; exit 2 ;;
esac
FAKE
  cat > "$FAKE_BIN/codesign" <<'FAKE'
#!/bin/sh
echo "codesign $*" >> "$FAKE_LOG"
for binary; do :; done
case "$1" in
  --force)
    [ "${FAKE_SIGN_FAILS:-}" != yes ] || exit 1
    printf '\nsigned by the fake codesign' >> "$binary" ;;
  --verify) grep -q 'signed by the fake codesign' "$binary" ;;
  -d)
    shown=$(( $(cat "$FAKE_DISPLAYS" 2>/dev/null || echo 0) + 1 ))
    echo "$shown" > "$FAKE_DISPLAYS"
    out=$FAKE_DISPLAY_OUT err=$FAKE_DISPLAY_ERR
    case "$(head -c 32 "$binary")" in x86_64-apple-darwin*) out=${FAKE_DISPLAY_OUT_X86:-$out} ;; esac
    # From display FAKE_DISPLAY_ERR_FROM on, the details are FAKE_DISPLAY_ERR_LATER's
    if [ -n "${FAKE_DISPLAY_ERR_FROM:-}" ] && [ "$shown" -ge "$FAKE_DISPLAY_ERR_FROM" ]; then
      err=$FAKE_DISPLAY_ERR_LATER
    fi
    cat "$out"
    cat "$err" >&2 ;;
  *) exit 2 ;;
esac
FAKE
  cat > "$FAKE_BIN/lipo" <<'FAKE'
#!/bin/sh
echo "lipo $*" >> "$FAKE_LOG"
if [ -n "${FAKE_ARCH:-}" ]; then echo "$FAKE_ARCH"; exit 0; fi
case "$(head -c 32 "$2")" in
  aarch64-apple-darwin*) echo arm64 ;;
  x86_64-apple-darwin*) echo x86_64 ;;
  *) exit 1 ;;
esac
FAKE
  cat > "$FAKE_BIN/security" <<'FAKE'
#!/bin/sh
echo "security $*" >> "$FAKE_LOG"
case "$1" in
  find-identity)
    # security itself failing, as it does on a locked keychain
    [ "${FAKE_FIND_IDENTITY_FAILS:-}" != yes ] || exit 51
    if [ "${FAKE_NO_IDENTITY:-}" = yes ]; then echo '     0 valid identities found'; exit 0; fi
    echo '  1) 0123456789ABCDEF0123456789ABCDEF01234567 "Developer ID Application: Geodica Pty Ltd (76BQL8L47U)"'
    echo '     1 valid identities found' ;;
  find-generic-password)
    # -a <account> -s <service> -w: a fixture value for each notary credential
    service=
    while [ $# -gt 0 ]; do
      case "$1" in -s) shift; service=$1 ;; esac
      shift
    done
    case "$service" in
      "${FAKE_NO_CREDENTIAL:-}") exit 44 ;;
      "${FAKE_EMPTY_CREDENTIAL:-}") echo ;;
      "$KEYCHAIN.APPLE_TEAM_ID") echo "$FAKE_NOTARY_TEAM" ;;
      "$KEYCHAIN".*) echo "fixture-${service#"$KEYCHAIN".}" ;;
      *) exit 44 ;;
    esac ;;
  *) echo "the fake security has no answer for $1" >&2; exit 2 ;;
esac
FAKE
  cat > "$FAKE_BIN/xcrun" <<'FAKE'
#!/bin/sh
echo "xcrun $*" >> "$FAKE_LOG"
answer=${FAKE_NOTARY:-}
[ -n "$answer" ] || answer='{"id":"2efe2717-52ef-43a5-96dc-0797e4ca1041","status":"Accepted"}'
[ "$answer" = none ] || printf '%s\n' "$answer"
exit "${FAKE_NOTARY_EXIT:-0}"
FAKE
  cat > "$FAKE_BIN/ditto" <<'FAKE'
#!/bin/sh
echo "ditto $*" >> "$FAKE_LOG"
FAKE
  chmod +x "$FAKE_BIN"/*
}

# stage <dir> <binary contents>: the four files an archive holds
stage() {
  mkdir -p "$1"
  printf LICENSE > "$1/LICENSE"
  printf README.md > "$1/README.md"
  printf ccusage-logo.svg > "$1/ccusage-logo.svg"
  printf '%s' "$2" > "$1/ccusage"
}

# CI's draft: each archive as `tar -czf ... -C staging .` writes it, the binary holding
# its target's triple, and CI's sums as SHA256SUMS.ci. Files made here carry
# com.apple.provenance, and COPYFILE_DISABLE stops bsdtar adding a ._ entry for each,
# as CI's packaging step does.
ci_release() {
  local target
  for target in $TARGETS; do
    stage "$BATS_TEST_TMPDIR/stage-$target" "$target"
    COPYFILE_DISABLE=1 bsdtar -czf "$FAKE_RELEASE/ccusage-$TAG-$target.tar.gz" -C "$BATS_TEST_TMPDIR/stage-$target" .
  done
  mkdir -p "$BATS_TEST_TMPDIR/ci"
  cp "$FAKE_RELEASE"/*.tar.gz "$BATS_TEST_TMPDIR/ci/"
  write_ci_sums
}

write_ci_sums() {
  local target
  (cd "$FAKE_RELEASE" && for target in $TARGETS; do shasum -a 256 "ccusage-$TAG-$target.tar.gz"; done) \
    > "$FAKE_RELEASE/SHA256SUMS.ci"
}

# arm <bsdtar arguments>: the Apple silicon archive rebuilt from them, with CI's sums
# rewritten to match, so only the archive's own checks stand in the way
arm() {
  COPYFILE_DISABLE=1 bsdtar -czf "$FAKE_RELEASE/ccusage-$TAG-aarch64-apple-darwin.tar.gz" "$@"
  write_ci_sums
}

finalize() { run "$HANDLER" finalize "$TAG"; }

# failed <message>: the run failed, saying so
failed() {
  [ "$status" -ne 0 ] || { echo "it succeeded: $output"; return 1; }
  [[ $output == *"$1"* ]] || { echo "output: $output"; return 1; }
}

# absent <pattern>: no call in the log matches it. bats runs a test under set -e, which
# never fails on `! grep`, so every check that something did not happen comes here.
absent() {
  if grep -qE "$1" "$FAKE_LOG"; then
    echo "the log has: $(grep -E "$1" "$FAKE_LOG")"
    return 1
  fi
}

# refused <message>: the run failed, saying so, before anything was signed. Each step
# returns its own failure: called left of ||, as a loop calls it, a function runs with
# set -e off, so a failing first step would not stop it.
refused() {
  failed "$1" || return 1
  absent '^codesign --force'
}

# unpublished: nothing reached the release's publish step
unpublished() { absent '^gh release edit'; }

# a copy of the display fixture with <sed expression> applied
display() {
  local out=$BATS_TEST_TMPDIR/display.$RANDOM
  sed "$2" "$1" > "$out"
  printf '%s' "$out"
}

# copy_tree <config sed> [<signing sed>]: the handler, config.yaml and the signing
# facts in a tree of their own, with each sed expression applied to its file; prints
# the copy's handler
copy_tree() {
  local tree=$BATS_TEST_TMPDIR/tree
  mkdir -p "$tree/bin/.devbin/cmd" "$tree/packaging/macos"
  cp "$HANDLER" "$tree/bin/.devbin/cmd/"
  sed "$1" "$ROOT/bin/.devbin/config.yaml" > "$tree/bin/.devbin/config.yaml"
  sed "${2:-}" "$ROOT/packaging/macos/signing" > "$tree/packaging/macos/signing"
  printf '%s' "$tree/bin/.devbin/cmd/macos"
}

@test "a draft is signed, uploaded, verified again, then published" {
  finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [[ $output == *"published $TAG"* ]]
  local upload last_download edit
  upload=$(grep -n '^gh release upload' "$FAKE_LOG" | cut -d: -f1)
  last_download=$(grep -n '^gh release download' "$FAKE_LOG" | tail -1 | cut -d: -f1)
  edit=$(grep -n '^gh release edit' "$FAKE_LOG" | cut -d: -f1)
  [ "$upload" -lt "$last_download" ]
  [ "$last_download" -lt "$edit" ]
  [ "$(grep '^gh ' "$FAKE_LOG" | tail -1)" = "gh release edit $TAG --repo $REPO_NAME --draft=false" ]
  [ ! -e "$FAKE_RELEASE/SHA256SUMS.ci" ]
  (cd "$FAKE_RELEASE" && shasum -a 256 -c SHA256SUMS)
  local target unpacked
  for target in aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
    cmp "$BATS_TEST_TMPDIR/ci/ccusage-$TAG-$target.tar.gz" "$FAKE_RELEASE/ccusage-$TAG-$target.tar.gz"
  done
  for target in aarch64-apple-darwin x86_64-apple-darwin; do
    unpacked=$BATS_TEST_TMPDIR/out-$target
    mkdir "$unpacked"
    bsdtar -xzf "$FAKE_RELEASE/ccusage-$TAG-$target.tar.gz" -C "$unpacked"
    [ "$(cat "$unpacked/ccusage")" = "$(printf '%s\n%s' "$target" "$MARKER")" ]
    [ "$(stat -f %Lp "$unpacked/ccusage")" = 755 ]
    [ "$(bsdtar -tvzf "$FAKE_RELEASE/ccusage-$TAG-$target.tar.gz" | awk '{ print $3 $4 }' | sort -u)" = 00 ]
  done
  grep -qxF "codesign --force --sign Developer ID Application: Geodica Pty Ltd (76BQL8L47U) --identifier com.geodica.ccusage --options runtime --timestamp $(grep '^codesign --force' "$FAKE_LOG" | head -1 | awk '{ print $NF }')" "$FAKE_LOG"
  [ "$(grep -c '^codesign --force' "$FAKE_LOG")" -eq "$(echo "$TARGETS" | tr ' ' '\n' | grep -c apple-darwin)" ]
}

@test "every gh call names the repository config.yaml's release.repo names" {
  local handler
  handler=$(copy_tree 's#^  repo: .*#  repo: example/elsewhere#')
  run "$handler" finalize "$TAG"
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [ "$(grep -c '^gh ' "$FAKE_LOG")" -eq "$(grep -c '^gh .* --repo example/elsewhere ' "$FAKE_LOG")" ]
  absent "$REPO_NAME"
}

@test "a config.yaml that names no release repo refuses before any gh call" {
  local handler
  handler=$(copy_tree '/^  repo: /d')
  run "$handler" finalize "$TAG"
  refused "names no release repo"
  absent '^gh '
}

@test "a signing file without one of its facts refuses before any gh call, naming it" {
  local key handler
  # Each key the file holds, read on fd 3 so each run keeps its own stdin
  while IFS= read -r key <&3; do
    : > "$FAKE_LOG"
    handler=$(copy_tree '' "/^$key=/d")
    run "$handler" finalize "$TAG"
    refused "packaging/macos/signing has no $key" || { echo "for $key"; return 1; }
    absent '^gh ' || { echo "for $key"; return 1; }
  done 3< <(sed -n 's/^\([a-z]*\)=.*/\1/p' "$ROOT/packaging/macos/signing")
}

@test "security failing to list identities is said as that, not as a missing identity" {
  FAKE_FIND_IDENTITY_FAILS=yes finalize
  refused "security cannot list this keychain's code-signing identities"
  [[ $output != *"not in this keychain"* ]] || { echo "output: $output"; return 1; }
  absent '^gh release download'
}

@test "the manifest keeps the layout published releases carry, bound to SHA256SUMS" {
  finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  local manifest=$FAKE_RELEASE/MACOS-SIGNING.json
  [ "$(head -5 "$manifest")" = "$(printf '{\n  "tag": "%s",\n  "identifier": "com.geodica.ccusage",\n  "team_id": "76BQL8L47U",\n  "archives": {' "$TAG")" ]
  local target name
  for target in aarch64-apple-darwin x86_64-apple-darwin; do
    name=ccusage-$TAG-$target.tar.gz
    [ "$(jq -r --arg n "$name" '.archives[$n].sha256' "$manifest")" = "$(grep " $name\$" "$FAKE_RELEASE/SHA256SUMS" | cut -d' ' -f1)" ]
    [ "$(jq -r --arg n "$name" '.archives[$n].requirement' "$manifest")" = "$(cat "$FAKE_DISPLAY_OUT")" ]
    [ "$(jq -r --arg n "$name" '.archives[$n].notarized' "$manifest")" = true ]
  done
  [ "$(jq -r '.archives | keys | join(" ")' "$manifest")" = "ccusage-$TAG-aarch64-apple-darwin.tar.gz ccusage-$TAG-x86_64-apple-darwin.tar.gz" ]
}

@test "a release already public is verified and left alone" {
  finalize
  [ "$status" -eq 0 ]
  : > "$FAKE_LOG"
  FAKE_DRAFT=false finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [[ $output == *"public already"* ]]
  grep -q '^gh release download .*--pattern MACOS-SIGNING.json' "$FAKE_LOG"
  absent '^gh release (upload|edit|delete-asset)'
  absent '^codesign --force'
  # Verifying needs neither the identity nor a notary credential
  absent '^security '
  absent '^xcrun '
}

@test "a public release whose manifest predates notarization still verifies" {
  finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  jq '.archives[].notarized = false' "$FAKE_RELEASE/MACOS-SIGNING.json" > "$BATS_TEST_TMPDIR/manifest"
  mv "$BATS_TEST_TMPDIR/manifest" "$FAKE_RELEASE/MACOS-SIGNING.json"
  : > "$FAKE_LOG"
  FAKE_DRAFT=false finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [[ $output == *"public already"* ]]
}

@test "a public release whose manifest names another tag, identifier or team is refused" {
  finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  local signed=$BATS_TEST_TMPDIR/signed.json forged
  cp "$FAKE_RELEASE/MACOS-SIGNING.json" "$signed"
  for forged in '.tag = "v0.0.1"' '.identifier = "com.example.ccusage"' '.team_id = "0000000000"'; do
    jq "$forged" "$signed" > "$FAKE_RELEASE/MACOS-SIGNING.json"
    : > "$FAKE_LOG"
    FAKE_DRAFT=false finalize
    refused "Release signing identity mismatch" || { echo "for $forged"; return 1; }
  done
}

@test "signed archives that do not verify are never uploaded" {
  # Signing reads one display per Mac binary. Every display after those has no
  # timestamp, so only the signed set's own check, before the upload, can refuse first.
  FAKE_DISPLAY_ERR_FROM=$(( $(macs) + 1 )) FAKE_DISPLAY_ERR_LATER=$(display "$FAKE_DISPLAY_ERR" '/^Timestamp=/d') finalize
  failed "requires a secure timestamp"
  absent '^gh release upload'
}

@test "CI names its sums SHA256SUMS.ci, so devbin's formula cannot render from an unsigned draft" {
  local workflow=$ROOT/.github/workflows/build.yml
  grep -qF 'sha256sum ccusage-*.tar.gz > SHA256SUMS.ci' "$workflow"
  # The draft's upload: archives, then CI's sums, on gh release create's line
  grep -qF 'ccusage-*.tar.gz SHA256SUMS.ci' "$workflow"
}

@test "a public release whose manifest sha256 disagrees with SHA256SUMS is refused" {
  finalize
  local name=ccusage-$TAG-aarch64-apple-darwin.tar.gz
  jq --arg n "$name" '.archives[$n].sha256 = ("0" * 64)' "$FAKE_RELEASE/MACOS-SIGNING.json" > "$BATS_TEST_TMPDIR/forged"
  mv "$BATS_TEST_TMPDIR/forged" "$FAKE_RELEASE/MACOS-SIGNING.json"
  : > "$FAKE_LOG"
  FAKE_DRAFT=false finalize
  refused "Signed archive checksum mismatch"
}

@test "a public release whose signature no longer matches its manifest is refused" {
  finalize
  : > "$FAKE_LOG"
  FAKE_DRAFT=false FAKE_DISPLAY_OUT=$(display "$FAKE_DISPLAY_OUT" 's/76BQL8L47U/0000000000/') finalize
  refused "Release designated requirement mismatch"
}

@test "a failed signing publishes nothing" {
  FAKE_SIGN_FAILS=yes finalize
  failed "codesign could not sign"
  absent '^gh release upload'
  unpublished
}

@test "bytes that change on their way to GitHub are never published" {
  # A Linux archive: no signature covers it, so only its checksum can catch this
  FAKE_AFTER_UPLOAD="printf x >> '$FAKE_RELEASE/ccusage-$TAG-x86_64-unknown-linux-gnu.tar.gz'" finalize
  failed "Release archive checksum mismatch"
  grep -q '^gh release upload' "$FAKE_LOG"
  unpublished
}

@test "a manifest that changes on its way to GitHub is never published" {
  FAKE_AFTER_UPLOAD="sed -i '' s/76BQL8L47U/0000000000/ '$FAKE_RELEASE/MACOS-SIGNING.json'" finalize
  failed "Release signing identity mismatch"
  unpublished
}

@test "a manifest that loses its notarized record on its way to GitHub is never published" {
  local manifest=$FAKE_RELEASE/MACOS-SIGNING.json changed=$BATS_TEST_TMPDIR/changed
  FAKE_AFTER_UPLOAD="jq '.archives[].notarized = false' '$manifest' > '$changed' && mv '$changed' '$manifest'" finalize
  failed "does not record ccusage-$TAG-"
  [[ $output == *"as notarized"* ]]
  unpublished
}

@test "a draft that does not match CI's sums is never signed" {
  printf x >> "$FAKE_RELEASE/ccusage-$TAG-aarch64-apple-darwin.tar.gz"
  finalize
  refused "Release archive checksum mismatch"
}

@test "a draft with no sums at all is never signed" {
  rm "$FAKE_RELEASE/SHA256SUMS.ci"
  finalize
  refused "neither CI's SHA256SUMS.ci nor a SHA256SUMS"
}

@test "a run whose publish did not finish is finished by the next" {
  FAKE_EDIT_FAILS=yes finalize
  failed "could not publish"
  [ -e "$FAKE_RELEASE/SHA256SUMS" ]
  [ ! -e "$FAKE_RELEASE/SHA256SUMS.ci" ]
  : > "$FAKE_LOG"
  finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  (cd "$FAKE_RELEASE" && shasum -a 256 -c SHA256SUMS)
}

@test "an unreadable draft status stops before any download" {
  FAKE_DRAFT=maybe finalize
  refused "Could not determine draft status"
  absent '^gh release download'
}

@test "SHA256SUMS lines need exactly a lowercase digest and a name, once each" {
  local sums=$FAKE_RELEASE/SHA256SUMS.ci good
  cp "$sums" "$BATS_TEST_TMPDIR/sums"
  good=$BATS_TEST_TMPDIR/sums
  sed '1s/$/ extra/' "$good" > "$sums"; finalize; refused "Invalid release checksum manifest"
  : > "$FAKE_LOG"; awk 'NR == 1 { $1 = toupper($1) } 1' OFS='  ' "$good" > "$sums"; finalize; refused "Invalid release checksum manifest"
  : > "$FAKE_LOG"; { cat "$good"; head -1 "$good"; } > "$sums"; finalize; refused "Invalid release checksum manifest"
  : > "$FAKE_LOG"; sed '1s/^\(.\{63\}\)./\1/' "$good" > "$sums"; finalize; refused "Invalid release checksum manifest"
}

@test "SHA256SUMS must name exactly the release's four archives" {
  local sums=$FAKE_RELEASE/SHA256SUMS.ci good=$BATS_TEST_TMPDIR/sums
  cp "$sums" "$good"
  sed '$d' "$good" > "$sums"; finalize; refused "Unexpected or missing release archives"
  : > "$FAKE_LOG"; { cat "$good"; printf '%s  ccusage-%s-riscv64gc-unknown-linux-gnu.tar.gz\n' "$(printf '%064d' 0)" "$TAG"; } > "$sums"
  finalize; refused "Unexpected or missing release archives"
}

@test "archives with a name other than the four files are refused" {
  local from=$BATS_TEST_TMPDIR/stage-aarch64-apple-darwin rename
  for rename in ../LICENSE sub/LICENSE license ./sub/LICENSE; do
    : > "$FAKE_LOG"
    arm -C "$from" -s ",^LICENSE\$,$rename," LICENSE README.md ccusage ccusage-logo.svg
    finalize
    refused "Unexpected release archive member" || { echo "for $rename"; return 1; }
  done
  : > "$FAKE_LOG"
  arm -P -C "$from" -s ',^LICENSE$,/LICENSE,' LICENSE README.md ccusage ccusage-logo.svg
  finalize
  refused "Unexpected release archive member"
}

@test "archives with a member too many, too few or twice are refused" {
  local from=$BATS_TEST_TMPDIR/stage-aarch64-apple-darwin
  printf extra > "$BATS_TEST_TMPDIR/extra"
  arm -C "$from" LICENSE README.md ccusage ccusage-logo.svg -C "$BATS_TEST_TMPDIR" extra
  finalize; refused "Unexpected release archive member"
  : > "$FAKE_LOG"; arm -C "$from" LICENSE ccusage ccusage-logo.svg
  finalize; refused "Unexpected release archive member"
  : > "$FAKE_LOG"; arm -C "$from" LICENSE LICENSE README.md ccusage ccusage-logo.svg
  finalize; refused "Unexpected release archive member"
}

@test "archives whose binary is a link or a directory are refused" {
  local odd=$BATS_TEST_TMPDIR/odd
  stage "$odd" aarch64-apple-darwin
  rm "$odd/ccusage" && ln -s LICENSE "$odd/ccusage"
  arm -C "$odd" LICENSE README.md ccusage ccusage-logo.svg
  finalize; refused "Unexpected release archive member"
  : > "$FAKE_LOG"; rm "$odd/ccusage" && ln "$odd/LICENSE" "$odd/ccusage"
  arm -C "$odd" LICENSE README.md ccusage ccusage-logo.svg
  finalize; refused "Unexpected release archive member"
  : > "$FAKE_LOG"; rm "$odd/ccusage" && mkdir "$odd/ccusage"
  arm -C "$odd" LICENSE README.md ccusage ccusage-logo.svg
  finalize; refused "Unexpected release archive member"
}

@test "archives with an AppleDouble member are refused" {
  # macOS bsdtar folds a ._name entry into the next one as Mac metadata unless its
  # mac-ext option is off, so a listing that left it on would never show it
  local from=$BATS_TEST_TMPDIR/stage-aarch64-apple-darwin
  printf 'not metadata' > "$from/._ccusage"
  COPYFILE_DISABLE=1 arm -C "$from" LICENSE README.md ._ccusage ccusage ccusage-logo.svg
  finalize
  refused "Unexpected release archive member"
}

@test "bare names unpack as well as CI's ./ root" {
  arm -C "$BATS_TEST_TMPDIR/stage-aarch64-apple-darwin" LICENSE README.md ccusage ccusage-logo.svg
  finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
}

@test "members over the size limit are refused" {
  # The Apple silicon binary is that archive's largest member
  local largest
  largest=$(stat -f %z "$BATS_TEST_TMPDIR/stage-aarch64-apple-darwin/ccusage")
  CCUSAGE_MACOS_MEMBER_LIMIT=$((largest - 1)) finalize
  refused "Unexpected release archive member"
}

@test "a binary of another architecture is refused" {
  local reported
  for reported in i386 "x86_64 arm64"; do
    : > "$FAKE_LOG"
    FAKE_ARCH=$reported finalize
    refused "Release binary has the wrong architecture"
  done
}

@test "a signature without a secure timestamp is refused" {
  FAKE_DISPLAY_ERR=$(display "$FAKE_DISPLAY_ERR" '/^Timestamp=/d') finalize
  failed "requires a secure timestamp"
  unpublished
}

@test "a signature without the hardened runtime is refused" {
  FAKE_DISPLAY_ERR=$(display "$FAKE_DISPLAY_ERR" 's/flags=0x10000(runtime)/flags=0x0(none)/') finalize
  failed "requires the hardened runtime"
  : > "$FAKE_LOG"
  # Runtime Version= is not the flag
  FAKE_DISPLAY_ERR=$(display "$FAKE_DISPLAY_ERR" 's/flags=0x10000(runtime)/flags=0x0(none)/;s/^Runtime Version/runtime/') finalize
  failed "requires the hardened runtime"
  unpublished
}

@test "a missing or cdhash-pinned designated requirement is refused" {
  FAKE_DISPLAY_OUT=/dev/null finalize
  failed "does not have a stable designated requirement"
  : > "$FAKE_LOG"
  FAKE_DISPLAY_OUT=$(display "$FAKE_DISPLAY_OUT" 's/.*/designated => cdhash H"0509a96c0991831c182058c8c8f4f4ccd433a87c"/') finalize
  failed "does not have a stable designated requirement"
  unpublished
}

@test "architectures signed by different identities are refused" {
  FAKE_DISPLAY_OUT_X86=$(display "$FAKE_DISPLAY_OUT" 's/76BQL8L47U/0000000000/') finalize
  failed "do not share the same signing identity"
  absent '^gh release upload'
}

@test "every signed binary is notarized with the keychain's credentials, read before any download" {
  finalize
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [ "$(jq '[.archives[].notarized] | all' "$FAKE_RELEASE/MACOS-SIGNING.json")" = true ]
  local name
  for name in APPLE_ID APPLE_APP_PASSWORD APPLE_TEAM_ID; do
    grep -qxF "security find-generic-password -a $USER -s $KEYCHAIN.$name -w" "$FAKE_LOG"
  done
  local submit="xcrun notarytool submit .* --apple-id fixture-APPLE_ID --password fixture-APPLE_APP_PASSWORD --team-id $FAKE_NOTARY_TEAM --wait --output-format json"
  [ "$(grep -c "^$submit\$" "$FAKE_LOG")" -eq "$(grep -c '^codesign --force' "$FAKE_LOG")" ]
  # Each wait is said before it starts, naming its binary's architecture
  [ "$(grep -c '^-- notarizing ccusage for .* (notarytool waits)$' <<<"$output")" -eq "$(grep -c '^codesign --force' "$FAKE_LOG")" ]
  [[ $output == *"-- notarizing ccusage for arm64 (notarytool waits)"* ]]
  [[ $output == *"-- notarizing ccusage for x86_64 (notarytool waits)"* ]]
  local last_read first_download
  last_read=$(grep -n '^security find-generic-password' "$FAKE_LOG" | tail -1 | cut -d: -f1)
  first_download=$(grep -n '^gh release download' "$FAKE_LOG" | head -1 | cut -d: -f1)
  [ "$last_read" -lt "$first_download" ]
}

@test "a missing or empty notary credential, or another team's, refuses before any download" {
  local name
  for name in APPLE_ID APPLE_APP_PASSWORD APPLE_TEAM_ID; do
    : > "$FAKE_LOG"
    FAKE_NO_CREDENTIAL=$KEYCHAIN.$name finalize
    refused "no $KEYCHAIN.$name in the login keychain" || { echo "for $name"; return 1; }
    absent '^gh release download'
    absent '^xcrun '
    : > "$FAKE_LOG"
    FAKE_EMPTY_CREDENTIAL=$KEYCHAIN.$name finalize
    refused "$KEYCHAIN.$name in the login keychain is empty" || { echo "for $name"; return 1; }
    absent '^gh release download'
  done
  : > "$FAKE_LOG"
  FAKE_NOTARY_TEAM=0000000000 finalize
  refused "name another team than packaging/macos/signing's"
  absent '^gh release download'
}

@test "a notarization that is not accepted publishes nothing, and names its submission alone" {
  local id=2efe2717-52ef-43a5-96dc-0797e4ca1041
  # notarytool exits non-zero on an Invalid verdict, and the verdict is still read
  FAKE_NOTARY="{\"id\":\"$id\",\"status\":\"Invalid\",\"message\":\"fixture diagnostic\"}" FAKE_NOTARY_EXIT=1 finalize
  failed "Notarization was not accepted (submission $id)"
  [[ $output != *"fixture diagnostic"* ]] || { echo "output: $output"; return 1; }
  absent '^gh release upload'
  unpublished
  local answer
  for answer in '{"status":"Invalid"}' '{}' '{"id":"not an id","status":"Invalid"}'; do
    rm -rf "$FAKE_RELEASE" && mkdir "$FAKE_RELEASE" && ci_release && : > "$FAKE_LOG"
    FAKE_NOTARY=$answer finalize
    failed "Notarization was not accepted" || { echo "for $answer"; return 1; }
    [[ $output != *"(submission"* ]] || { echo "for $answer: $output"; return 1; }
    unpublished
  done
}

@test "a submission notarytool cannot make publishes nothing" {
  FAKE_NOTARY=none FAKE_NOTARY_EXIT=1 finalize
  failed "notarytool could not submit"
  absent '^gh release upload'
  unpublished
}

@test "verification holds each binary to packaging/macos/signing's requirement, which macOS parses" {
  finalize
  [ "$status" -eq 0 ]
  local requirement
  requirement=$(sed -n 's/^requirement=//p' "$ROOT/packaging/macos/signing")
  grep -qF "codesign --verify --strict -R =$requirement " "$FAKE_LOG"
  # Without its leading =, codesign and csreq would read the requirement as a file name
  /usr/bin/csreq -r "=$requirement" -b "$BATS_TEST_TMPDIR/requirement"
}

@test "the formula checks the requirement the signing does" {
  # The Ruby string's pieces, each in single or double quotes, joined
  local line='' text='' started='' single="'([^']*)'" double='"(.*)"'
  while IFS= read -r line; do
    [[ -n $started || $line == *'requirement = '* ]] || continue
    started=1
    if [[ $line =~ $single ]] || [[ $line =~ $double ]]; then text+=${BASH_REMATCH[1]}; fi
    [[ $line == *"\\" ]] || break
  done < "$ROOT/packaging/homebrew/ccusage.rb"
  [ "$text" = "=$(sed -n 's/^requirement=//p' "$ROOT/packaging/macos/signing")" ]
}

@test "help prints the usage block, from its first line to the bare # after it" {
  run "$HANDLER" --help
  [ "$status" -eq 0 ]
  [[ $output == *"bin/devbin macos finalize <tag>"* ]]
  [[ $output == *"left alone." ]] || { echo "output: $output"; return 1; }
  [[ $output != *"An archive is unpacked"* ]]
}

@test "it refuses on CI, without the signing identity, and for a tag that is not stable" {
  CI=true finalize
  refused "refusing to run on CI"
  absent '^gh '
  FAKE_NO_IDENTITY=yes finalize
  refused "signing identity is not in this keychain"
  absent '^gh release download'
  : > "$FAKE_LOG"
  local tag
  for tag in 0.1.4 v0.1 v01.1.4 v0.1.4-rc.1; do
    run "$HANDLER" finalize "$tag"
    refused "expected a stable release tag"
  done
  absent '^gh '
}
