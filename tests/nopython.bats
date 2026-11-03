#!/usr/bin/env bats
# bin/devbin check nopython, over a repository holding this checkout's tracked files as
# they stand: the tree passes; a Python file, a Python #! line, and a line naming one
# of the words with no reason are each refused by path and line; a line that gives its
# reason, prose, and the words inside other names or capitalised all pass. Ported from
# Pplr's.

setup() {
  ROOT=$(cd "$BATS_TEST_DIRNAME/.." && pwd)
}

# A git repository in the sandbox, seeded with this checkout's tracked files, so
# devbin, its config and the check come with it
seed_repo() {
  REPO=$BATS_TEST_TMPDIR/repo
  mkdir -p "$REPO"
  (cd "$ROOT" && git ls-files -z | tar -c --null -T - -f -) | tar -x -f - -C "$REPO"
  git -C "$REPO" init -q
  git -C "$REPO" add -A
  git -C "$REPO" -c user.name=fixture -c user.email=fixture@example.invalid -c commit.gpgsign=false commit -q -m seed
}

# plant <path> <lines...>: one staged file in REPO
plant() {
  local path=$1
  shift
  mkdir -p "$REPO/$(dirname "$path")"
  printf '%s\n' "$@" > "$REPO/$path"
  chmod +x "$REPO/$path"
  git -C "$REPO" add -- "$path"
}

# REPO as seeded again
unplant() {
  git -C "$REPO" reset -q --hard
  git -C "$REPO" clean -q -fd
}

@test "check nopython passes the tree and refuses Python by path and line" {
  seed_repo
  run bash -c "cd '$REPO' && bin/devbin check nopython"
  [ "$status" -eq 0 ] || { echo "the tree as it stands is refused: $output" >&2; return 1; }

  # Each refused plant: the file, the path and line the refusal must name, then its lines
  local refused=(
    "lib/helper.py|lib/helper.py: |print(1)"
    "tools/helper|tools/helper:1: |#!/usr/bin/env python3|print(1)" # nopython: a plant, never run
    "tools/run-it|tools/run-it:2: |#!/bin/sh|uv run thing" # nopython: a plant, never run
    "src/words.rs|src/words.rs:1: |// see python3 -m json.tool" # nopython: a plant, never run
  )
  local row path names
  local -a lines
  for row in "${refused[@]}"; do
    IFS='|' read -r -a lines <<< "$row"
    path=${lines[0]} names=${lines[1]}
    plant "$path" "${lines[@]:2}"
    run bash -c "cd '$REPO' && bin/devbin check nopython"
    [ "$status" -ne 0 ] || { echo "$path passed" >&2; return 1; }
    [[ $output == *"$names"* ]] || { echo "$path refused without naming $names: $output" >&2; return 1; }
    unplant
  done

  # Each plant that passes: the file, then its lines
  local passed=(
    "tools/run-it|#!/bin/sh|uv run thing # nopython: a planted line with its reason"
    "notes/how.md|Run it with uv run thing, or python3 -m it." # nopython: a plant, never run
    "src/words.rs|// Python's repr, as_python_wrote, nopython, uvula"
  )
  for row in "${passed[@]}"; do
    IFS='|' read -r -a lines <<< "$row"
    path=${lines[0]}
    plant "$path" "${lines[@]:1}"
    run bash -c "cd '$REPO' && bin/devbin check nopython"
    [ "$status" -eq 0 ] || { echo "$path was refused: $output" >&2; return 1; }
    unplant
  done
}
