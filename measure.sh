#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the level of each of the eight channels of desk.wav, which demo.sh
# recorded in OUT, between each step and the next, as the RMS level in dBFS,
# or "-" for silence.  The recording lines up with the marks of the steps by
# the onset of the music on channel 1 after GO, since parecord starts
# recording a moment after it starts.
#
# Usage: measure.sh OUT, with ffmpeg on PATH
set -euo pipefail
out=$1

at() { awk -F'\t' -v key="$1" '$2 == key { print $1; exit }' "$out/marks.tsv"; }
onset=$(ffmpeg -hide_banner -i "$out/desk.wav" -af 'pan=mono|c0=c0,silencedetect=n=-50dB:d=0.3' \
  -f null - 2>&1 | awk '/silence_end/ { print $5; exit }')
# Wall time of the first sample of the recording, taking GO to reach the
# sink within a buffer of PulseAudio, about 20 milliseconds
origin=$(awk -v go="$(at house)" -v onset="$onset" 'BEGIN { printf "%.3f", go + 0.02 - onset }')
printf 'recording starts %.3f s after parecord\n' "$(awk -v a="$origin" -v b="$(at "record desk")" 'BEGIN { print a - b }')"

levels() {
  ffmpeg -hide_banner -ss "$1" -to "$2" -i "$out/desk.wav" \
    -af astats=measure_perchannel=RMS_level:measure_overall=none -f null - 2>&1 |
    awk '/RMS level dB/ { level = $NF; printf "\t%s", (level == "-inf" || level < -70 ? "-" : sprintf("%.1f", level)) }'
}

steps=(house sub monitors house-off sub-4 panic end)
printf 'step\t1\t2\t3\t4\t5\t6\t7\t8\n'
for index in $(seq 0 $((${#steps[@]} - 2))); do
  step=${steps[$index]} next=${steps[$((index + 1))]}
  # A second for the step to apply and a fade to end, and a little before
  # the next step
  from=$(awk -v at="$(at "$step")" -v origin="$origin" 'BEGIN { printf "%.3f", at - origin + 1.5 }')
  to=$(awk -v at="$(at "$next")" -v origin="$origin" 'BEGIN { printf "%.3f", at - origin - 0.3 }')
  printf '%s%s\n' "$step" "$(levels "$from" "$to")"
done
