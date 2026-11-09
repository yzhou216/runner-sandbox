#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the H.264 clips that realtime/run.sh plays, with x264 through
# ffmpeg, in the directory $1: FFmpeg's test pattern under light moving
# grain, about ten seconds each, at the bitrate of video made for playback
# in a show, 20 Mbit/s, with x264's default of three B-frames.  Writes the
# number of pictures of each clip to counts.txt, for the plays to compare
# with.
set -euo pipefail
cd "$1"
encode() {
  local name=$1 source=$2
  shift 2
  ffmpeg -hide_banner -loglevel error -y -f lavfi -i "$source" -c:v libx264 -pix_fmt yuv420p \
    -profile:v high -b:v 20M -maxrate 25M -bufsize 40M "$@" "$name.mp4"
}
grain=noise=alls=8:allf=t
encode 1080p60 "testsrc2=size=1920x1080:rate=60:duration=10,$grain" -preset medium
encode 2160p30 "testsrc2=size=3840x2160:rate=30:duration=10,$grain" -preset veryfast
for clip in *.mp4; do
  echo "$clip $(ffprobe -v error -count_packets -select_streams v:0 \
    -show_entries stream=nb_read_packets -of csv=p=0 "$clip")"
done > counts.txt
ls -l
cat counts.txt
