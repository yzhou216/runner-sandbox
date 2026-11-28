#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Summarizes the per-commit results of a flake-check/series run, whose
# artifacts results-<mode>-<k> are unpacked under the directory given.
#
# Usage: summarize-series.sh <artifacts directory>
set -uo pipefail

dir=$1
here=$(cd "$(dirname "$0")" && pwd)
mapfile -t series < <(ls "$here"/series/*.patch | sort)
echo "## flake-check series: each commit against the commits before it"
echo
echo "| # | Commit | GNU/Linux | macOS |"
echo "| --- | --- | --- | --- |"
cell() { # mode k
  local d="$dir/results-$1-$2"
  [ -f "$d/results.tar" ] && [ ! -d "$d/results" ] && tar -xf "$d/results.tar" -C "$d"
  local r="$d/results/results.md"
  if [ ! -f "$r" ]; then
    [ "$1" = darwin ] && echo "not run" || echo "**no results**"
    return
  fi
  local odd
  odd=$(grep -E '^\| `' "$r" | awk -F'|' '$3 !~ /^ *same *$/ { gsub(/`| /, "", $2); gsub(/^ +| +$/, "", $3); printf "%s: %s; ", $2, $3 }')
  echo "${odd:-all same}"
}
for i in "${!series[@]}"; do
  k=$((i + 1))
  s=$(grep -m 1 '^Subject:' "${series[$i]}" | sed -E 's/^Subject: (\[PATCH[^]]*\] )?//')
  echo "| $k | $s | $(cell linux "$k") | $(cell darwin "$k") |"
done
echo
echo "Each commit's full results.md is in results-<mode>-<k>/results.tar."
