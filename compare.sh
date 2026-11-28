#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Compares two lists of drvPaths from drv.sh: $1 and $2 are the files, $3 the
# extended regular expression of the "kind system output" keys that may
# differ ("^$" for none).  Prints each key that differs and fails when one
# does not match.
set -uo pipefail
a=$1 b=$2 allowed=$3
keys() { cut -d' ' -f1-3 "$1"; }
if ! diff <(keys "$a") <(keys "$b"); then
  echo "the outputs differ"
  exit 1
fi
changed=$(diff "$a" "$b" | sed -n 's/^> \([^ ]* [^ ]* [^ ]*\) .*/\1/p')
echo "$(wc -l < "$a") outputs, $(printf %s "$changed" | grep -c . || true) changed"
[ -z "$changed" ] && exit 0
echo "$changed"
bad=$(echo "$changed" | command grep -Ev -- "$allowed" || true)
if [ -n "$bad" ]; then
  echo "UNEXPECTED:"
  echo "$bad"
  exit 1
fi
echo "only the expected outputs changed"
