#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# compose.sh <shots directory> <output.mp4> <font file>
#
# Burns the captions of captions.tsv into a band under demo.mov and ends
# the video three seconds after the last caption, since screencapture
# gives its last frame a duration of many minutes.
set -eu
shots=$1 output=$2 font=$3
texts=$(mktemp -d)

mapfile -t times < <(cut -f1 "$shots/captions.tsv")
mapfile -t captions < <(cut -f2 "$shots/captions.tsv")
end=$(echo "${times[-1]} + 3" | bc)
filters="fps=30,pad=iw:ih+60:0:0:black"
for i in "${!captions[@]}"; do
  printf '%s' "${captions[$i]}" > "$texts/$i.txt"
  until=${times[$((i + 1))]:-$end}
  filters+=",drawtext=fontfile=$font:textfile=$texts/$i.txt:expansion=none"
  filters+=":fontcolor=white:fontsize=26:x=(w-text_w)/2:y=h-45"
  filters+=":enable='between(t,${times[$i]},$until)'"
done
ffmpeg -loglevel error -y -i "$shots/demo.mov" -t "$end" -vf "$filters" \
  -c:v libx264 -crf 23 -pix_fmt yuv420p -movflags +faststart "$output"
