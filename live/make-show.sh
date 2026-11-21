#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the show of the recording in DIRECTORY: one sound of 10 seconds in
# three parts, whose levels draw its structure on the graph, and
# devamp.cinq.toml with the cues that loop and devamp it.  The introduction,
# from 0 to 2 s, is a tone of 330 Hz at -30 dBFS.  The vamp, from 2 to 6 s, is
# a tone of 440 Hz whose level rises steadily from -36 dBFS to -10 dBFS, so
# that each pass round the loop draws one tooth of a sawtooth.  The ending,
# from 6 to 10 s, is a tone of 550 Hz whose level falls steadily from -10 dBFS
# to -70 dBFS.  Each part starts and ends on a whole cycle, so that the loop
# and the joins click no more than the level does.  The list Main holds the
# looping sound and a devamp cue on it.
#
# Usage: make-show.sh DIRECTORY, with ffmpeg on PATH
set -euo pipefail
dir=$1
mkdir -p "$dir"
ffmpeg -nostdin -hide_banner -loglevel error -y -f lavfi \
  -i "aevalsrc='if(lt(t,2),10^(-30/20)*sin(2*PI*330*t),if(lt(t,6),10^((-36+26*(t-2)/4)/20)*sin(2*PI*440*t),10^((-10-60*(t-6)/4)/20)*sin(2*PI*550*t)))':s=48000:d=10" \
  -c:a flac -fflags +bitexact "$dir/scene.flac"
cat > "$dir/devamp.cinq.toml" << 'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Scene music"
file = "scene.flac"
loop = true
loop_start = 2.0
loop_end = 6.0

[[list.cue]]
id = 2
number = "2"
kind = "devamp"
name = "Leave the vamp"
target = 1
TOML
