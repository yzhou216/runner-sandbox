#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs cinq's test program over and over, to tell how often a test fails by
# chance.  Usage: stress.sh <test program> <times of a single test> <times of
# the video tests> <times of every test>
set -u
exe=$1
single=$2
video=$3
every=$4

# Runs `$exe args...` $n times, and prints how many of them failed, with the
# output of the first that did
repeat() {
  local label=$1 n=$2 failed=0 shown=0 out
  shift 2
  for _ in $(seq 1 "$n"); do
    if ! out=$("$exe" "$@" 2>&1); then
      failed=$((failed + 1))
      if [ "$shown" -lt 3 ]; then
        shown=$((shown + 1))
        echo "--- failure $shown of $label"
        echo "$out" | tr -d '\r' | grep -vE '^test .* ok$' | head -40
      fi
    fi
  done
  echo "$label: $failed of $n runs failed"
}

repeat "the test that overflowed" "$single" \
  picture_that_the_next_go_starts_is_made_ready_ahead_for_its_run_to_take
repeat "the video tests" "$video" back::video
repeat "every test" "$every"
