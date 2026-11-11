#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# compose.sh <shots directory> <output.mp4> <font file>
#
# Burns the captions of captions.tsv into a band under demo.mkv, whose
# video began lead.txt seconds before the captions' clock, from that clock's
# start to three seconds after the last caption.  The font size of a 56th
# of the width fits the longest caption on one line.
set -eu
shots=$1 output=$2 font=$3
texts=$(mktemp -d)

lead=$(cat "$shots/lead.txt")
skip=$(echo "if ($lead > 0) $lead else 0" | bc)
mapfile -t times < <(cut -f1 "$shots/captions.tsv" | while read -r t; do echo "$t + $lead - $skip" | bc; done)
mapfile -t captions < <(cut -f2 "$shots/captions.tsv")
end=$(echo "${times[-1]} + 3" | bc)
filters="fps=30,pad=iw:ih+44:0:0:black"
for i in "${!captions[@]}"; do
  printf '%s' "${captions[$i]}" > "$texts/$i.txt"
  until=${times[$((i + 1))]:-$end}
  filters+=",drawtext=fontfile=$font:textfile=$texts/$i.txt:expansion=none"
  filters+=":fontcolor=white:fontsize=w/56:x=(w-text_w)/2:y=h-31"
  filters+=":enable='between(t,${times[$i]},$until)'"
done
ffmpeg -loglevel error -y -ss "$skip" -i "$shots/demo.mkv" -t "$end" -vf "$filters" \
  -c:v libx264 -crf 23 -pix_fmt yuv420p -movflags +faststart "$output"
