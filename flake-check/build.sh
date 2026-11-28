#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds the flake attributes listed in flake-check/build, from the base
# commit with flake-check/flake.patch applied.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
out=${RESULTS:-$PWD/results}
work=${WORK:-$(mktemp -d)}
url=${FLAKE_URL:-https://gitlab.com/yiyuz/flake.git}
base=$(grep -v '^#' "$here/base" | tr -d '[:space:]')
mkdir -p "$out"

git config --global user.name flake-check
git config --global user.email flake-check@localhost
git clone --quiet "$url" "$work/cand"
git -C "$work/cand" checkout --quiet "$base"
if [ -s "$here/flake.patch" ] && grep -q '^diff --git ' "$here/flake.patch"; then
  git -C "$work/cand" apply --index --whitespace=nowarn "$here/flake.patch"
  git -C "$work/cand" commit --quiet -m candidate
  (cd "$work/cand" && nix flake lock) || true
fi

r=$out/results.md
echo "## flake-check (build)" >"$r"
echo >>"$r"
status=0
while read -r t; do
  t=${t%%#*}
  t=$(echo "$t" | xargs)
  [ -z "$t" ] && continue
  if nix build --impure --keep-going --no-link -L "$work/cand#$t" >"$out/$t.log" 2>&1; then
    echo "- \`$t\`: built" >>"$r"
  else
    status=1
    echo "- \`$t\`: **failed**; last lines:" >>"$r"
    { echo; echo '```'; grep -v '^\s*$' "$out/$t.log" | tail -n 30; echo '```'; echo; } >>"$r"
  fi
done <"$here/build"
exit "$status"
