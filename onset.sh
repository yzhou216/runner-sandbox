#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the time in seconds at which channel CHANNEL of FILE first
# sounds, from START, the time of the first sample of FILE.
#
# Usage: onset.sh FILE CHANNEL START, with ffmpeg on PATH and CHANNEL
# counted from 1
set -euo pipefail
file=$1 channel=$(($2 - 1)) start=$3
onset=$(ffmpeg -hide_banner -i "$file" -af "pan=mono|c0=c$channel,silencedetect=n=-50dB:d=0.3" \
  -f null - 2>&1 | awk '/silence_end/ { print $5; exit }')
[ -n "$onset" ] || { echo "channel $2 of $file is silent" >&2; exit 1; }
awk -v start="$start" -v onset="$onset" 'BEGIN { printf "%.4f\n", start + onset }'
