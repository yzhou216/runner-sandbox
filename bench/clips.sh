#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the H.264 clips that cinq-bench pass decodes, with x264 through
# ffmpeg, in the directory $1: FFmpeg's test pattern, which is mostly flat
# and decodes fast, and the same under moving noise, which takes the
# decoder about as long as filmed footage does
set -euo pipefail
cd "$1"
encode() {
  local name=$1 source=$2
  shift 2
  ffmpeg -hide_banner -loglevel error -y -f lavfi -i "$source" -c:v libx264 -pix_fmt yuv420p \
    -profile:v high "$@" "$name.mp4"
}
encode 720p-noise 'testsrc2=size=1280x720:rate=30:duration=6,noise=alls=24:allf=t+u' -preset medium -crf 20
encode 1080p 'testsrc2=size=1920x1080:rate=30:duration=6' -preset medium -crf 23
encode 1080p-noise 'testsrc2=size=1920x1080:rate=30:duration=4,noise=alls=24:allf=t+u' -preset medium -crf 20
encode 2160p 'testsrc2=size=3840x2160:rate=30:duration=4' -preset veryfast -crf 23
encode 2160p-noise 'testsrc2=size=3840x2160:rate=30:duration=2,noise=alls=24:allf=t+u' -preset veryfast -crf 20
ls -l
