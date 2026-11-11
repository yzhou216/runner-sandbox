#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Usage: record.sh RECORDING CONFIG CHECK_SCORES STEP
#
# Replay test_data/RECORDING.json with the game's --test mode, which checks
# every state against the recorded snapshots, and join a screenshot of every
# STEP-th frame into videos/RECORDING.mp4.  In test mode each frame advances
# the game by exactly 1/64 s, so the 32 fps video plays at STEP/2 times the
# game's speed.  Fail unless the game reports that the test passed.

set -euo pipefail

name=$1 config=$2 check_scores=$3 step=$4
root=$(cd "$(dirname "$0")" && pwd)
data=$root/milestone-1/test_data
work=$root/capture/$name
mkdir -p "$work" "$root/videos"

# Frames up to the last input, then three seconds to show the result
last=$(python3 -I -c '
import json, math, sys
t = json.load(open(sys.argv[1]))["events"][-1]["time"]
print(math.ceil((t["secs"] + t["nanos"] / 1e9) * 64) + 192)
' "$data/$name.json")

{
  echo '(events: ['
  seq 4 "$step" "$last" | awk '{ printf "(%d, NamedScreenshot(\"%06d\")),\n", $1, $1 }'
  echo "($((last + 32)), AppExit),"
  echo '])'
} > "$work/ci.ron"

(
  cd "$work"
  CI_TESTING_CONFIG=$work/ci.ron \
    cargo run --locked --features test \
    --manifest-path "$root/milestone-1/Cargo.toml" -- \
    "$data/$config" --test "$data/$name.json" --check-scores "$check_scores"
) 2>&1 | tee "$root/videos/$name.log"

frames=("$work"/screenshot-*.png)
cp "${frames[-1]}" "$root/videos/$name-last.png"
size=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height \
  -of csv=s=x:p=0 "${frames[-1]}")
ffmpeg -loglevel error -y -framerate 32 -pattern_type glob \
  -i "$work/screenshot-*.png" \
  -vf "scale=${size/x/:},pad=ceil(iw/2)*2:ceil(ih/2)*2" \
  -c:v libx264 -crf 18 -pix_fmt yuv420p "$root/videos/$name.mp4"
echo "$name: ${#frames[@]} frames at $size"

grep -q 'Passed with' "$root/videos/$name.log"
