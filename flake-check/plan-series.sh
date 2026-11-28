#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the GitHub Actions outputs of the matrix for flake-check/series:
# every commit index for GNU/Linux, and for macOS the indexes of commits
# that touch files the Darwin configurations can read.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
mapfile -t series < <(ls "$here"/series/*.patch | sort)
linux=()
darwin=()
for i in "${!series[@]}"; do
  k=$((i + 1))
  linux+=("$k")
  if git apply --numstat "${series[$i]}" | cut -f 3 |
    grep -qE '^(flake\.nix|flake\.lock|systems/[^/]+\.nix|systems/darwin/|machines/darwin/|patched\.nix|lib/)'; then
    darwin+=("$k")
  fi
done
json() { local IFS=,; echo "[$*]"; }
echo "linux=$(json "${linux[@]}")"
echo "darwin=$(json "${darwin[@]}")"
