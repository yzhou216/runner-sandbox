#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays each clip in the directory $1 in real time through cinq's video
# output, by the test in realtime.rs, with three builds of cinq: `base`,
# cinq's master at ec61ead; `change`, the snapshot in cinq/, whose pictures
# convert to RGBA in words of two pixels; and `threaded`, the snapshot with
# the decoder of the prototype in decoder-threaded.rs.  Prints a line for
# each play, after the processor time of the test process.
set -euo pipefail
clips=$(cd "$1" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
root=$(dirname "$here")
work=$root/realtime-work
export CARGO_TARGET_DIR=$work/target CINQ_SHARE_DIR=''
rm -rf "$work/base" "$work/change" "$work/threaded"
mkdir -p "$work"
git clone -q https://github.com/yzhou216/cinq "$work/base"
git -C "$work/base" checkout -q ec61eadfa4d75a258054d1967e2242f3abdd8fad
cp -R "$root/cinq" "$work/change"
cp -R "$root/cinq" "$work/threaded"
cp "$here/decoder-threaded.rs" "$work/threaded/src/back/video/decoder.rs"
for tree in base change threaded; do
  cp "$here/realtime.rs" "$work/$tree/src/back/video/tests/realtime.rs"
  perl -0pi -e 's/^mod player;$/mod player;\nmod realtime;/m' "$work/$tree/src/back/video/tests.rs"
  grep -q '^mod realtime;$' "$work/$tree/src/back/video/tests.rs"
  (cd "$work/$tree" && cargo test --release --locked --no-run --quiet)
done
TIMEFORMAT='%U %S'
for round in 1 2; do
  for clip in "$clips"/*.mp4; do
    # Path as the native programs of Windows take it
    native=$(cygpath -m "$clip" 2> /dev/null || echo "$clip")
    for tree in base change threaded; do
      seconds=$({ time (cd "$work/$tree" && CINQ_REALTIME_CLIP=$native cargo test --release --locked \
        --quiet realtime -- --ignored --nocapture | grep '^REALTIME' > "$work/line"); } 2>&1)
      echo "round $round  $tree  cpu $seconds  $(cat "$work/line")"
    done
  done
done
