#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Clones the flake at the commit in flake-check/base into the given
# directory and commits flake-check/flake.patch on top, which stands in
# for the actions/checkout step of the flake's own workflows.
#
# Usage: flake-ci/checkout.sh <directory>
set -euo pipefail

dir=$1
here=$(cd "$(dirname "$0")" && pwd)
url=${FLAKE_URL:-https://gitlab.com/yiyuz/flake.git}
base=$(grep -v '^#' "$here/../flake-check/base" | tr -d '[:space:]')

git config --global user.name flake-ci
git config --global user.email flake-ci@localhost
git config --global advice.detachedHead false
git clone --quiet "$url" "$dir"
git -C "$dir" checkout --quiet "$base"
patch=$here/../flake-check/flake.patch
if [ -s "$patch" ] && grep -q '^diff --git ' "$patch"; then
  git -C "$dir" apply --index --whitespace=nowarn "$patch"
  git -C "$dir" commit --quiet -m candidate
fi
git -C "$dir" log --oneline -1
