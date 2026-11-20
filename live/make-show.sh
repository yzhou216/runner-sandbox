#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the show of the recording in DIRECTORY: a tone of 440 Hz whose level
# rises steadily from -40 dBFS to -6 dBFS in 20 seconds, so that where the
# tone is shows in its level, and control.cinq.toml with the cues that act on
# it.  The list Main holds the tone, a pause cue, a start cue, and a GoTo
# cue, all on the tone; the list Effects holds a start cue on the tone.
#
# Usage: make-show.sh DIRECTORY, with ffmpeg on PATH
set -euo pipefail
dir=$1
mkdir -p "$dir"
ffmpeg -nostdin -hide_banner -loglevel error -y -f lavfi \
  -i "aevalsrc='10^((-40+34*t/20)/20)*sin(2*PI*440*t)':s=48000:d=20" \
  -c:a flac -fflags +bitexact "$dir/tone.flac"
cat > "$dir/control.cinq.toml" << 'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Rising tone"
file = "tone.flac"

[[list.cue]]
id = 2
number = "2"
kind = "pause"
name = "Hold the tone"
target = 1

[[list.cue]]
id = 3
number = "3"
kind = "start"
name = "Resume the tone"
target = 1

[[list.cue]]
id = 4
number = "4"
kind = "goto"
name = "Back to the tone"
target = 1

[[list]]
name = "Effects"

[[list.cue]]
id = 5
number = "1"
kind = "start"
name = "Start the tone from Effects"
target = 1
TOML
