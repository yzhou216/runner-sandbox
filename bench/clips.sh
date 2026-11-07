#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the H.264 clips that cinq-bench pass decodes, with x264 through
# ffmpeg, in the directory $1: FFmpeg's test pattern, which is mostly flat
# and decodes fast, and the same under light moving grain at the bitrates
# of video made for playback in a show, 20 Mbit/s for 1080p and 50 for
# 2160p, which takes the decoder about as long as filmed footage does
set -euo pipefail
cd "$1"
encode() {
  local name=$1 source=$2
  shift 2
  ffmpeg -hide_banner -loglevel error -y -f lavfi -i "$source" -c:v libx264 -pix_fmt yuv420p \
    -profile:v high "$@" "$name.mp4"
}
grain=noise=alls=8:allf=t
encode 1080p 'testsrc2=size=1920x1080:rate=30:duration=6' -preset medium -crf 23
encode 2160p 'testsrc2=size=3840x2160:rate=30:duration=4' -preset veryfast -crf 23
encode 1080p30-grain "testsrc2=size=1920x1080:rate=30:duration=6,$grain" -preset medium \
  -b:v 20M -maxrate 25M -bufsize 40M
encode 1080p60-grain "testsrc2=size=1920x1080:rate=60:duration=4,$grain" -preset medium \
  -b:v 20M -maxrate 25M -bufsize 40M
encode 2160p30-grain "testsrc2=size=3840x2160:rate=30:duration=3,$grain" -preset veryfast \
  -b:v 50M -maxrate 60M -bufsize 100M
ls -l
