#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the sounds and the two shows of the audio checks in DIRECTORY: tone.flac,
# a mono 440 Hz tone at -3 dBFS that lasts 40 seconds; vamp.flac, six seconds of
# 440 Hz in three parts (an introduction at -20 dBFS, a loop section at -6 dBFS,
# an ending at -14 dBFS); plain.cinq.toml, one cue on the default device; and
# outputs.cinq.toml, two outputs and the cues of checks 2 to 8.
#
# Usage: make-show.sh DIRECTORY, with ffmpeg on PATH
set -euo pipefail
dir=$1
mkdir -p "$dir"
make() { ffmpeg -nostdin -hide_banner -loglevel error -y "$@"; }
make -f lavfi -i "aevalsrc='10^(-3/20)*sin(2*PI*440*t)':s=48000:d=40" -ac 1 -c:a flac "$dir/tone.flac"
make -f lavfi -i "aevalsrc='sin(2*PI*440*t)*(lt(t,2)*10^(-20/20)+between(t,2,4)*10^(-6/20)+gte(t,4)*10^(-14/20))':s=48000:d=6" \
  -ac 1 -c:a flac "$dir/vamp.flac"
cat > "$dir/plain.cinq.toml" << TOML
[[list]]
name = "Plain"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Tone"
file = "tone.flac"
level = -6.0
TOML
cat > "$dir/outputs.cinq.toml" << TOML
[[output]]
name = "House"
device = "alsa:house"
channels = "1-2"

[[output]]
name = "Booth"
device = "alsa:booth"
channels = "1-2"

[[list]]
name = "Outputs"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Split"
file = "tone.flac"
level = -6.0
[list.cue.sends]
House = 0.0
Booth = -10.0

[[list.cue]]
id = 2
number = "2"
kind = "fade"
name = "Fade"
target = 1
duration = 3.0
level = -20.0
stop_target = false

[[list.cue]]
id = 3
number = "3"
kind = "stop"
name = "Stop"
target = 1

[[list.cue]]
id = 4
number = "4"
kind = "audio"
name = "Vamp"
file = "vamp.flac"
loop = true
loop_start = 2.0
loop_end = 4.0
[list.cue.sends]
House = 0.0

[[list.cue]]
id = 5
number = "5"
kind = "devamp"
name = "Devamp"
target = 4

[[list.cue]]
id = 6
number = "6"
kind = "audio"
name = "Panic tone"
file = "tone.flac"
[list.cue.sends]
House = 0.0
Booth = 0.0

[[list.cue]]
id = 7
number = "7"
kind = "audio"
name = "Outage tone"
file = "tone.flac"
[list.cue.sends]
House = 0.0
Booth = 0.0

[[list.cue]]
id = 8
number = "8"
kind = "audio"
name = "Nowhere tone"
file = "tone.flac"
[list.cue.sends]
House = 0.0
Booth = -10.0
TOML
