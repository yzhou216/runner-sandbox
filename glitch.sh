#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints, for channel CHANNEL of FILE from START to END seconds, which holds
# a steady tone of TONE Hz, the RMS level of the tone in dBFS, the lowest
# level of any quarter of a second, and how far below the tone the rest
# lies in the worst quarter, once four notches take the tone away.  A
# dropout lowers the lowest quarter, and a click, a gap, or a tone of the
# wrong pitch raises the rest of its quarter.
#
# Usage: glitch.sh FILE CHANNEL TONE START END, with ffmpeg on PATH and
# CHANNEL counted from 1
set -euo pipefail
file=$1 channel=$(($2 - 1)) tone=$3 start=$4 end=$5
rate=$(ffprobe -v error -select_streams a:0 -show_entries stream=sample_rate -of csv=p=0 "$file")
quarter=$((rate / 4))
notch=",bandreject=f=$tone:width_type=h:w=300"
# RMS of each quarter, in dBFS, one a line.  The notches run from the start
# of the file, so that they have settled before the quarters begin.
quarters() {
  ffmpeg -hide_banner -i "$file" \
    -af "pan=mono|c0=c$channel$1,atrim=start=$start:end=$end,asetnsamples=n=$quarter,astats=metadata=1:reset=1,ametadata=mode=print:key=lavfi.astats.1.RMS_level:file=-" \
    -f null - 2> /dev/null | sed -n 's/^lavfi.astats.1.RMS_level=//p' | sed 's/^-inf$/-999/'
}
level=$(ffmpeg -hide_banner -ss "$start" -to "$end" -i "$file" \
  -af "pan=mono|c0=c$channel,astats=measure_perchannel=RMS_level:measure_overall=none" -f null - 2>&1 |
  awk '/RMS level dB/ { print $NF; exit }')
lowest=$(quarters '' | awk 'NR == 1 || $1 < min { min = $1 } END { print min }')
worst=$(quarters "$notch$notch$notch$notch" | awk 'NR == 1 || $1 > max { max = $1 } END { print max }')
awk -v level="$level" -v lowest="$lowest" -v worst="$worst" \
  'BEGIN { printf "%s\t%s\t%.1f\n", level, lowest, level - worst }'
