#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the sound of the audio cue of desk.cinq.toml in the directory SHOW
# with ffmpeg: a minute of an arpeggio around A4 at 0.3 of full scale, the
# same on both channels, whose RMS level is -13.5 dBFS on each.
#
# Usage: media.sh SHOW
set -euo pipefail
ffmpeg -loglevel error -y \
  -f lavfi -i "aevalsrc='0.3*sin(2*PI*t*440*pow(2,mod(floor(t*4),4)*4/12))':s=48000:d=60" \
  -ac 2 "$1/music.flac"
