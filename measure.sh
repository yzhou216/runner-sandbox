#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the level of each of the eight channels of board.wav, which
# dante.sh recorded in OUT, between each step and the next, as the RMS
# level in dBFS, or "-" for silence.  The recording lines up with the marks
# by the onset of the music on channel 1 after the first GO of cinq.
#
# Usage: measure.sh OUT, with ffmpeg on PATH
set -euo pipefail
out=$1
wav=$out/board.wav

at() { awk -F'\t' -v key="$1" '$2 == key { print $1; exit }' "$out/marks.tsv"; }
since() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%.3f", a - b }'; }
start=$(at "record board")

levels() {
  ffmpeg -hide_banner -ss "$1" -to "$2" -i "$wav" \
    -af astats=measure_perchannel=RMS_level:measure_overall=none -f null - 2>&1 |
    awk '/RMS level dB/ { level = $NF; printf "\t%s", (level == "-inf" || level < -70 ? "-" : sprintf("%.1f", level)) }'
}
row() { printf '%s%s\n' "$1" "$(levels "$2" "$3")"; }

printf 'step\t1\t2\t3\t4\t5\t6\t7\t8\n'
# Tones play steadily, so the marks alone place them
row tones "$(since "$(at tones-heard)" "$start")" "$(since "$(at tones-end)" "$start")"

# First sound on channel 1 after the GO of cinq, about 20 milliseconds of
# buffers after the key
from=$(since "$(at house)" "$start")
onset=$(ffmpeg -hide_banner -ss "$from" -i "$wav" -af 'pan=mono|c0=c0,silencedetect=n=-50dB:d=0.3' \
  -f null - 2>&1 | awk '/silence_end/ { print $5; exit }')
if [ -z "$onset" ]; then
  echo "no sound after GO" >&2
  exit 1
fi
# Seconds of the recording at each wall time, from the onset
shift=$(awk -v go="$from" -v onset="$onset" 'BEGIN { printf "%.3f", onset - 0.02 }')
printf 'onset of the music %.3f s after GO\n' "$shift" >&2
place() { awk -v at="$1" -v start="$start" -v shift="$shift" -v add="$2" 'BEGIN { printf "%.3f", at - start + shift + add }'; }
steps=(house sub monitors house-off sub-4 panic desk-end)
for index in $(seq 0 $((${#steps[@]} - 2))); do
  step=${steps[$index]} next=${steps[$((index + 1))]}
  row "$step" "$(place "$(at "$step")" 1.5)" "$(place "$(at "$next")" -0.3)"
done
row partial "$(place "$(at partial)" 1.5)" "$(place "$(at partial-end)" -0.3)"
