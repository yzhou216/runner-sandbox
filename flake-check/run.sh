#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Evaluates the outputs of github.com/yzhou216/flake listed in
# flake-check/targets at the commit in flake-check/base, and again with
# flake-check/flake.patch applied, then reports which derivations,
# evaluation warnings, and lint findings differ between the two.
#
# Usage: run.sh [linux|darwin]
set -euo pipefail

mode=${1:-linux}
here=$(cd "$(dirname "$0")" && pwd)
out=${RESULTS:-$PWD/results}
work=${WORK:-$(mktemp -d)}
url=${FLAKE_URL:-https://gitlab.com/yiyuz/flake.git}
base=$(grep -v '^#' "$here/base" | tr -d '[:space:]')
mkdir -p "$out"

git config --global user.name flake-check
git config --global user.email flake-check@localhost
git config --global advice.detachedHead false

git clone --quiet "$url" "$work/base"
git -C "$work/base" checkout --quiet "$base"
git clone --quiet "$work/base" "$work/cand"
git -C "$work/cand" checkout --quiet "$base"

patched=no
if [ -s "$here/flake.patch" ] && grep -q '^diff --git ' "$here/flake.patch"; then
  git -C "$work/cand" apply --index --whitespace=nowarn "$here/flake.patch"
  git -C "$work/cand" commit --quiet -m candidate
  patched=yes
fi

# Lock inputs that the candidate adds, without updating the others
lock_note=
if [ "$patched" = yes ] && [ "$mode" = linux ]; then
  if (cd "$work/cand" && nix flake lock 2>"$out/lock.err"); then
    if ! git -C "$work/cand" diff --quiet -- flake.lock; then
      cp "$work/cand/flake.lock" "$out/flake.lock.new"
      git -C "$work/cand" commit --quiet -am "candidate: lock"
      lock_note="nix flake lock changed flake.lock; the new file is flake.lock.new in the results artifact."
    fi
  else
    lock_note="nix flake lock failed: $(tail -n 5 "$out/lock.err")"
  fi
fi

targets=()
while read -r t; do
  t=${t%%#*}
  t=$(echo "$t" | xargs)
  [ -z "$t" ] && continue
  case "$mode:$t" in
    darwin:darwinConfigurations.*) targets+=("$t") ;;
    linux:darwinConfigurations.*) ;;
    darwin:*) ;;
    *) targets+=("$t") ;;
  esac
done <"$here/targets"

evaluate() { # tree
  local tree=$1 t f
  local impure=(--impure)
  # PURE evaluates the candidate without --impure, to show that it can be
  if [ "$tree" = cand ] && [ -n "${PURE:-}" ]; then impure=(); fi
  mkdir -p "$out/$tree"
  for t in "${targets[@]}"; do
    f="$out/$tree/$t"
    if nix eval "${impure[@]}" --raw "$work/$tree#$t.drvPath" >"$f.drv" 2>"$f.err"; then
      :
    else
      echo ERROR >"$f.drv"
    fi
  done
}

# Evaluate both trees at once; each fetches the same inputs
evaluate base &
pid=$!
evaluate cand
wait "$pid"

r=$out/results.md
{
  echo "## flake-check ($mode)"
  echo
  echo "Base \`$base\`; candidate: $( [ "$patched" = yes ] && git -C "$work/cand" diff --shortstat "$base" HEAD || echo 'no patch, same as base')"
  echo
  [ -n "$lock_note" ] && { echo "$lock_note"; echo; }
  [ -n "${PURE:-}" ] && { echo "The candidate is evaluated without --impure, the base with it."; echo; }
  echo "| Target | Result | Base | Candidate |"
  echo "| --- | --- | --- | --- |"
  for t in "${targets[@]}"; do
    b=$(cat "$out/base/$t.drv")
    c=$(cat "$out/cand/$t.drv")
    if [ "$c" = ERROR ] && [ "$b" = ERROR ]; then res="error in both"
    elif [ "$c" = ERROR ]; then res="**candidate error**"
    elif [ "$b" = ERROR ]; then res="fixed (base error)"
    elif [ "$b" = "$c" ]; then res=same
    else res="**changed**"
    fi
    echo "| \`$t\` | $res | \`$(basename "$b" | cut -c1-12)\` | \`$(basename "$c" | cut -c1-12)\` |"
  done
  echo
  echo "### Evaluation errors and warnings"
  echo
  for tree in base cand; do
    cat "$out/$tree"/*.err 2>/dev/null |
      grep -E 'warning|error|trace' |
      sed -E "s#$work/$tree#<tree>#g; s#/nix/store/[a-z0-9]{32}-#/nix/store/…-#g" |
      sort -u >"$out/$tree.messages"
  done
  if cmp -s "$out/base.messages" "$out/cand.messages"; then
    echo "Same in both ($(wc -l <"$out/cand.messages") distinct lines):"
    echo
    echo '```'
    head -n 60 "$out/cand.messages"
    echo '```'
  else
    echo "Only in base:"
    echo
    echo '```'
    comm -23 "$out/base.messages" "$out/cand.messages" | head -n 60
    echo '```'
    echo
    echo "Only in candidate:"
    echo
    echo '```'
    comm -13 "$out/base.messages" "$out/cand.messages" | head -n 60
    echo '```'
  fi
  echo
  echo "Full messages per target are in base/*.err and cand/*.err of the results artifact."
} >"$r"

if [ "$mode" = linux ]; then
  {
    echo
    echo "### Lints"
    echo
    for tree in base cand; do
      (
        cd "$work/$tree"
        mapfile -t files < <(git ls-files '*.nix')
        nix run --quiet nixpkgs#nixfmt -- --check "${files[@]}" >"$out/$tree.nixfmt" 2>&1 || true
        nix run --quiet nixpkgs#deadnix -- "${files[@]}" >"$out/$tree.deadnix" 2>&1 || true
        nix run --quiet nixpkgs#statix -- check . >"$out/$tree.statix" 2>&1 || true
      )
    done
    for l in nixfmt deadnix statix; do
      echo "- $l: base $(grep -c . "$out/base.$l" || true) lines, candidate $(grep -c . "$out/cand.$l" || true) lines"
    done
    echo
    echo "nixfmt on the candidate:"
    echo
    echo '```'
    head -n 40 "$out/cand.nixfmt"
    echo '```'
    echo
    echo "Full deadnix and statix output is in cand.deadnix and cand.statix of the results artifact."
    changed=()
    for t in "${targets[@]}"; do
      b=$(cat "$out/base/$t.drv")
      c=$(cat "$out/cand/$t.drv")
      if [ "$b" != ERROR ] && [ "$c" != ERROR ] && [ "$b" != "$c" ]; then
        changed+=("$t")
        nix run --quiet nixpkgs#nix-diff -- --color never "$b" "$c" >"$out/cand/$t.nix-diff" 2>&1 || true
      fi
    done
    if [ "${#changed[@]}" -gt 0 ]; then
      echo
      echo "### nix-diff of changed targets"
      for t in "${changed[@]}"; do
        echo
        echo "<details><summary>$t</summary>"
        echo
        echo '```'
        head -n 80 "$out/cand/$t.nix-diff"
        echo '```'
        echo "</details>"
      done
    fi
  } >>"$r"
fi
