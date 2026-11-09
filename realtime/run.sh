#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays each clip in the directory $1 in real time through cinq's video
# output, by the test in realtime.rs, with up to two builds of cinq: `base`,
# cinq's master at 798c620, and `threaded`, the tree in cinq/ built against
# the rust_h264 in rust_h264/, when both exist.  Prints a line for each
# play: the pictures shown against those in the clip, and the processor
# time of the test process.
set -euo pipefail
clips=$(cd "$1" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
root=$(dirname "$here")
work=$root/realtime-work
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$work/target} CINQ_SHARE_DIR=''
rm -rf "$work/base" "$work/threaded"
mkdir -p "$work"
git clone -q https://github.com/yzhou216/cinq "$work/base"
git -C "$work/base" checkout -q 798c620
trees=(base)
if [ -d "$root/cinq" ] && [ -d "$root/rust_h264" ]; then
  # The path of the patch in Cargo.toml is ../rust_h264
  mkdir "$work/threaded"
  cp -R "$root/cinq" "$work/threaded/cinq"
  cp -R "$root/rust_h264" "$work/threaded/rust_h264"
  trees+=(threaded)
fi
dir() { if [ "$1" = threaded ]; then echo "$work/threaded/cinq"; else echo "$work/$1"; fi; }
# Each tree builds its test program, which is copied out at once, since
# the trees are one package to cargo, whose next build replaces it
for tree in "${trees[@]}"; do
  tests=$(dir "$tree")/src/back/video/tests
  cp "$here/realtime.rs" "$tests/realtime.rs"
  perl -0pi -e 's/^mod player;$/mod player;\nmod realtime;/m' "$tests.rs"
  grep -q '^mod realtime;$' "$tests.rs"
  program=$(cd "$(dir "$tree")" && cargo test --release --locked --no-run --message-format=json |
    grep -o '"executable":"[^"]*"' | tail -1 | cut -d'"' -f4 | sed 's|\\\\|/|g')
  case $program in
    *.exe) suffix=.exe ;;
    *) suffix='' ;;
  esac
  cp "$program" "$work/test-$tree$suffix"
done
for round in 1 2; do
  for clip in "$clips"/*.mp4; do
    name=$(basename "$clip")
    count=$(awk -v n="$name" '$1 == n { print $2 }' "$clips/counts.txt")
    # Path as the native programs of Windows take it
    native=$(cygpath -m "$clip" 2> /dev/null || echo "$clip")
    for tree in "${trees[@]}"; do
      line=$(cd "$(dir "$tree")" && CINQ_REALTIME_CLIP=$native "$work/test-$tree$suffix" \
        realtime --ignored --nocapture | grep '^REALTIME')
      shown=$(echo "$line" | sed -E 's/.*: ([0-9]+) pictures.*/\1/')
      cpu=$(echo "$line" | sed -E 's/.*cpu ([0-9.]+) user ([0-9.]+) system.*/\1 user + \2 system/')
      first=$(echo "$line" | sed -E 's/.*first picture ([0-9.a-zA-Z]+) s after.*/\1 s/')
      echo "round $round  $name  $tree  shown $shown of $count  cpu $cpu  first picture $first after the GO"
      echo "  $line"
    done
  done
done
