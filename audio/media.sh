#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Writes the sounds of the shows into the directory DIR with ffmpeg, in
# 32-bit floating point at 48 kHz, 60 seconds each:
#
# - stereo.wav: 1 kHz at 0.3 of full scale on the left and 0.15 on the
#   right, so that the left peaks at -10.5 dBFS and the right at -16.5, and
#   a mono output, which takes half of each, plays at 0.225
# - clip.wav: 100 Hz at 0.8 of full scale on both channels, which two cues
#   at 0 dB on one channel take past full scale, as one cue does when its
#   send is raised by 12 dB, which is not the case for a single cue
# - half.wav: 100 Hz at 0.5 of full scale on both channels, which a send at
#   +12 dB takes to twice full scale
#
# Usage: media.sh DIR, with ffmpeg on PATH
set -euo pipefail
dir=$1
wave() {
  ffmpeg -loglevel error -y -f lavfi -i "aevalsrc='$2*sin(2*PI*t*$1)|$3*sin(2*PI*t*$1)':s=48000:d=60" \
    -c:a pcm_f32le -f wav "$4"
}
wave 1000 0.3 0.15 "$dir/stereo.wav"
wave 100 0.8 0.8 "$dir/clip.wav"
wave 100 0.5 0.5 "$dir/half.wav"
