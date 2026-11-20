#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the show of the recordings in DIRECTORY: a tone of 440 Hz at -3 dBFS,
# four seconds long, which holds a whole number of cycles so that it loops
# without a click, and levels.cinq.toml with three cues that act on it.  The
# swell fades to SWELL dB, -6 unless the environment says otherwise.
#
# Usage: make-show.sh DIRECTORY, with ffmpeg on PATH
set -euo pipefail
dir=$1
swell=${SWELL:--6.0}
mkdir -p "$dir"
ffmpeg -nostdin -hide_banner -loglevel error -y -f lavfi \
  -i "aevalsrc='10^(-3/20)*sin(2*PI*440*t)':s=48000:d=4" \
  -c:a flac -fflags +bitexact "$dir/tone.flac"
cat > "$dir/levels.cinq.toml" << TOML
[[list]]
name = "Levels"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Tone"
file = "tone.flac"
loop = true
level = -20.0

[[list.cue]]
id = 2
number = "2"
kind = "fade"
name = "Duck"
target = 1
duration = 2.0
level = -30.0
stop_target = false

[[list.cue]]
id = 3
number = "3"
kind = "fade"
name = "Swell"
target = 1
duration = 12.0
level = $swell
stop_target = false
TOML
