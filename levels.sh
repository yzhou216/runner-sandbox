#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the RMS level in dBFS of each channel of FILE, from START seconds
# to END seconds, or the last seconds of the file for a negative START,
# tab-separated, with "-" for silence.
#
# Usage: levels.sh FILE START [END], with ffmpeg on PATH
set -euo pipefail
file=$1 start=$2 end=${3:-}
if [ "${start#-}" != "$start" ]; then
  window=(-sseof "$start")
else
  window=(-ss "$start")
fi
[ -n "$end" ] && window+=(-to "$end")
ffmpeg -hide_banner -f wav "${window[@]}" -i "$file" \
  -af astats=measure_perchannel=RMS_level:measure_overall=none -f null - 2>&1 |
  awk '/RMS level dB/ { level = $NF; printf "%s%s", (n++ ? "\t" : ""), (level == "-inf" || level < -70 ? "-" : sprintf("%.1f", level)) } END { print "" }'
