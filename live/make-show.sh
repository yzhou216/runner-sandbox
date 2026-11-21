#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the show of the recording in DIRECTORY: two sounds, whose levels draw
# the scene on the graph, and groups.cinq.toml with the cues that use them.
# The rain lasts 4 s and is a steady tone of 440 Hz at -30 dBFS, and the thunder
# lasts 3 s and is a steady tone of 660 Hz at -14 dBFS.  Both hold whole cycles,
# so that the loop of the rain clicks no more than its level does.  The list Main
# holds the group Scene, which holds the looping rain and the thunder after a
# pre-wait of 2 s, and a stop cue below the group that targets it.
#
# Usage: make-show.sh DIRECTORY, with ffmpeg on PATH
set -euo pipefail
dir=$1
mkdir -p "$dir"
sound() { # NAME HZ DB SECONDS
  ffmpeg -nostdin -hide_banner -loglevel error -y -f lavfi \
    -i "aevalsrc='10^($3/20)*sin(2*PI*$2*t)':s=48000:d=$4" \
    -c:a flac -fflags +bitexact "$dir/$1.flac"
}
sound rain 440 -30 4
sound thunder 660 -14 3
cat > "$dir/groups.cinq.toml" << 'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "group"
name = "Scene"

[[list.cue]]
id = 2
number = "1.1"
kind = "audio"
name = "Rain"
file = "rain.flac"
loop = true
depth = 1

[[list.cue]]
id = 3
number = "1.2"
kind = "audio"
name = "Thunder"
file = "thunder.flac"
pre_wait = 2.0
depth = 1

[[list.cue]]
id = 4
number = "2"
kind = "stop"
name = "End the scene"
target = 1
TOML
