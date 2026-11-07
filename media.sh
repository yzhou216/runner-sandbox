#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the files of video.cinq.toml in the directory $1 with ffmpeg: a
# chord of four seconds, which loops without a seam, since each of its
# notes fits a whole number of cycles in it, and a video of ten seconds
# whose picture shows its own time to the hundredth of a second, with a
# 440 Hz tone as its sound
set -euo pipefail
ffmpeg -loglevel error -y -f lavfi \
  -i 'aevalsrc=0.15*sin(2*PI*262*t)+0.15*sin(2*PI*330*t)+0.15*sin(2*PI*392*t):d=4' \
  "$1/chord.flac"
ffmpeg -loglevel error -y -f lavfi -i testsrc=size=960x540:rate=30:decimals=2 \
  -f lavfi -i sine=frequency=440 -t 10 -c:v libx264 -preset veryfast \
  -profile:v high -pix_fmt yuv420p -c:a aac "$1/clock.mp4"
