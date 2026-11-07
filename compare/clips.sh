#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the H.264 clips that compare-h264 checks, with x264 through ffmpeg,
# in the directory $1, and the MD5 that FFmpeg's own decoder gives each of
# their pictures
set -euo pipefail
cd "$1"
encode() {
  local name=$1 source=$2
  shift 2
  ffmpeg -hide_banner -loglevel error -y -f lavfi -i "$source" -c:v libx264 -pix_fmt yuv420p "$@" "$name.mp4"
  ffmpeg -hide_banner -loglevel error -y -i "$name.mp4" -f framemd5 -pix_fmt yuv420p "$name.framemd5"
}
encode high-1080p 'testsrc2=size=1920x1080:rate=30:duration=4' -profile:v high -preset medium -crf 23
encode high-720p-noise 'testsrc2=size=1280x720:rate=30:duration=3,noise=alls=24:allf=t+u' -profile:v high -preset medium -crf 20
encode main-540p 'mandelbrot=size=960x540:rate=30,trim=duration=4' -profile:v main -preset medium -crf 23
encode baseline-360p 'testsrc=size=640x360:rate=30:duration=4' -profile:v baseline -preset fast -crf 23
encode high-1080p-refs 'testsrc2=size=1920x1080:rate=30:duration=3' -profile:v high -preset slow -crf 21 -x264-params ref=6:bframes=4:b-pyramid=normal:weightp=2:direct=auto
encode high-2160p 'testsrc2=size=3840x2160:rate=30:duration=2' -profile:v high -preset veryfast -crf 23
encode high-1080i 'testsrc2=size=1920x1080:rate=30:duration=3' -profile:v high -preset fast -crf 23 -flags +ildct+ilme -x264-params tff=1
ls -l
