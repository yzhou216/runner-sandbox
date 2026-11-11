#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the sound of the audio cue and the film of the video cue of
# devices.cinq.toml in the directory SHOW with ffmpeg: an arpeggio around
# A4, and a film whose sound is a lower tone, so that each cue sounds apart.
#
# Usage: media.sh SHOW
set -euo pipefail
show=$1
make() { ffmpeg -loglevel error -y "$@"; }
make -f lavfi -i "aevalsrc='0.3*sin(2*PI*t*440*pow(2,mod(floor(t*4),4)*4/12))':s=48000:d=60" \
  -ac 2 "$show/music.flac"
make -f lavfi -i "mandelbrot=size=1280x720:rate=30" \
  -f lavfi -i "sine=frequency=220:sample_rate=48000" -t 30 \
  -c:v libx264 -profile:v high -pix_fmt yuv420p -c:a aac -ac 2 "$show/film.mp4"
