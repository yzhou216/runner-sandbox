#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Burns captions into a band under the picture of a video.  CAPTIONS holds
# one line for each caption, the second at which it appears, a tab, and its
# text, and ends with a line whose text is empty, which ends the last
# caption.
#
# Usage: burn.sh FONT IN CAPTIONS OUT, with ffmpeg and ffprobe on PATH and
# FONT the path of a TrueType font
set -euo pipefail

font=$1 in=$2 captions=$3 out=$4
width=$(ffprobe -v error -select_streams v:0 -show_entries stream=width -of csv=p=0 "$in")
# A size at which a caption of 95 characters fits, and a band of even height
size=$((width / 56))
band=$((size * 3 / 2 * 2))
work=$(mktemp -d)
filters=()
mapfile -t lines < <(tr -d '\r' < "$captions")
for ((i = 0; i + 1 < ${#lines[@]}; i++)); do
  IFS=$'\t' read -r at text <<< "${lines[i]}"
  IFS=$'\t' read -r next _ <<< "${lines[i + 1]}"
  printf '%s' "$text" > "$work/$i.txt"
  filters+=("drawtext=fontfile=$font:textfile=$work/$i.txt:expansion=none:fontsize=$size:fontcolor=white:x=(w-text_w)/2:y=h-$band+($band-text_h)/2:enable='between(t,$at,$next)'")
done
IFS=,
ffmpeg -loglevel error -y -i "$in" -vf "pad=iw:ih+$band:0:0:black,${filters[*]}" \
  -c:v libx264 -crf 22 -pix_fmt yuv420p "$out"
