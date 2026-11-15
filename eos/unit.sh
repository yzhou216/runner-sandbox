#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Builds the tests of cinq once, runs those named network, light, and osc
# as cargo test does, each filter on its own, and then each test alone with
# its time.  The script writes OUT/summary.txt and OUT/tests.tsv.
#
# Usage: unit.sh SOURCE OUT, with cargo on PATH
set -uo pipefail

src=$(cd "$1" && pwd)
out=$(mkdir -p "$2" && cd "$2" && pwd)
cd "$src" || exit
export CINQ_SHARE_DIR='' CARGO_PROFILE_DEV_DEBUG=0 CARGO_TERM_COLOR=never
secs() { local d=$(( ${EPOCHREALTIME/./} - ${1/./} )); printf "%d.%02d" $((d / 1000000)) $((d % 1000000 / 10000)); }
uname -a > "$out/summary.txt"
cargo -V >> "$out/summary.txt"
rustc -V >> "$out/summary.txt"

start=$EPOCHREALTIME
cargo test --locked --no-run 2>&1 | tee "$out/build.log"
status=${PIPESTATUS[0]}
printf 'cargo test --no-run: status %s, %s s\n' "$status" "$(secs "$start")" >> "$out/summary.txt"
[ "$status" = 0 ] || exit "$status"

for filter in network light osc; do
  start=$EPOCHREALTIME
  cargo test --locked "$filter" 2>&1 | tee "$out/cargo-test-$filter.log"
  printf 'cargo test %s: status %s, %s s, %s\n' "$filter" "${PIPESTATUS[0]}" "$(secs "$start")" \
    "$(command grep -E '^test result' "$out/cargo-test-$filter.log" | head -1)" >> "$out/summary.txt"
done

exe=$(sed -n 's/^ *Executable unittests.*(\(.*\))$/\1/p' "$out/build.log" | head -1)
printf 'test executable: %s\n' "$exe" >> "$out/summary.txt"
: > "$out/tests.tsv"
"$exe" --list 2> /dev/null | sed -n 's/: test$//p' | command grep -E 'network|light|osc' |
  while IFS= read -r name; do
    start=$EPOCHREALTIME
    "$exe" --exact "$name" > "$out/one.log" 2>&1
    code=$?
    printf '%s\t%s\t%s\n' "$name" "$([ "$code" = 0 ] && echo pass || echo FAIL)" "$(secs "$start")" >> "$out/tests.tsv"
    [ "$code" = 0 ] || { echo "== $name"; cat "$out/one.log"; } >> "$out/failures.log"
  done
printf 'tests alone: %s run, %s failed\n' "$(wc -l < "$out/tests.tsv")" "$(command grep -c FAIL "$out/tests.tsv" || true)" >> "$out/summary.txt"
cat "$out/summary.txt"
