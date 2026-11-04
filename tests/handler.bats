#!/usr/bin/env bats
# bin/.devbin/cmd/common/handler.zsh, the library the project handlers source, through
# a probe handler of their shape in a tree of its own.

bats_require_minimum_version 1.5.0

setup() {
  ROOT=$(cd "$BATS_TEST_DIRNAME/.." && pwd)
  TREE=$BATS_TEST_TMPDIR/tree
  PROBE=$TREE/bin/.devbin/cmd/probe
  mkdir -p "$TREE/bin/.devbin/cmd"
  cp -R "$ROOT/bin/.devbin/cmd/common" "$TREE/bin/.devbin/cmd/"
  # As the real handlers are: a usage block in the header, the options, then the library
  cat > "$PROBE" <<'PROBE'
#!/usr/bin/env zsh
# probe: what the library gives a handler.
#
#   bin/devbin probe <what>   the usage block's first line
#                             and its second
#
# After the usage block.

emulate -L zsh
setopt err_exit no_unset pipe_fail

source ${0:A:h}/common/handler.zsh

case $1 in
  --help) usage ;;
  where) print -r -- "$root $PWD" ;;
  say) say "said" ;;
  die) die "died" ;;
esac
PROBE
  chmod +x "$PROBE"
}

@test "a handler works from its own tree, whatever devbin's PROJECT_ROOT names" {
  local tree
  tree=$(cd "$TREE" && pwd -P)
  PROJECT_ROOT=$ROOT run "$PROBE" where
  [ "$status" -eq 0 ] || { echo "$output"; return 1; }
  [ "$output" = "$tree $tree" ] || { echo "output: $output"; return 1; }
}

@test "die and say name the handler, die on stderr with status 1" {
  run --separate-stderr "$PROBE" say
  [ "$status" -eq 0 ]
  [ "$output" = "probe: said" ]
  [ -z "$stderr" ]
  run --separate-stderr "$PROBE" die
  [ "$status" -eq 1 ]
  [ "$stderr" = "probe: died" ]
  [ -z "$output" ]
}

@test "usage prints the header's usage block, from its first line to the bare # after it" {
  run "$PROBE" --help
  [ "$status" -eq 0 ]
  [[ $output == "  bin/devbin probe <what>   the usage block's first line"$'\n'* ]] || { echo "output: $output"; return 1; }
  [[ $output == *$'\n'"                            and its second" ]] || { echo "output: $output"; return 1; }
  [[ $output != *"what the library gives"* && $output != *"After the usage block"* ]]
  # $output drops trailing newlines, so the bare # itself is checked on the last line
  # usage prints, which an empty line there would be
  [ -n "$("$PROBE" --help | tail -n 1)" ] || { echo "usage ends in a blank line"; return 1; }
}
