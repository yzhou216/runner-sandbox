#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Puts two recordings of compose.sh side by side, the first on the left and
# the second on the right, each under a label.  Both start three seconds
# before their first GO, so they share a timeline.  The left recording's
# sound goes to the left channel and the right recording's to the right one.
#
# Usage: side.sh LEFT.mp4 RIGHT.mp4 RESULT FONT LEFT-LABEL RIGHT-LABEL, with
# ffmpeg on PATH and FONT a TrueType font file
set -euo pipefail
left=$1 right=$2 result=$3 font=$4
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
printf '%s' "$5" > "$work/left.txt"
printf '%s' "$6" > "$work/right.txt"
label() { printf 'pad=iw:ih+36:0:36:black,drawtext=fontfile=%s:textfile=%s:expansion=none:fontcolor=yellow:fontsize=26:x=(w-text_w)/2:y=4' "$font" "$work/$1.txt"; }
ffmpeg -nostdin -hide_banner -loglevel error -y -i "$left" -i "$right" -filter_complex "
  [0:v]$(label left)[l]; [1:v]$(label right)[r];
  [l][r]hstack=inputs=2,scale=1920:-2,format=yuv420p[video];
  [0:a]pan=mono|c0=0.5*c0+0.5*c1[a0]; [1:a]pan=mono|c0=0.5*c0+0.5*c1[a1];
  [a0][a1]amerge=inputs=2,aformat=channel_layouts=stereo[audio]
" -map '[video]' -map '[audio]' -shortest -c:v libx264 -crf 22 -preset medium -c:a aac -b:a 160k \
  -movflags +faststart "$result"
