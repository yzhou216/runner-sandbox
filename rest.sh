#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the RMS level in dBFS of the left channel of FILE from half a
# second on, and that of what is left of it once four notches at TONE Hz
# take the tone away, which interpolation adds to it.
#
# Usage: rest.sh FILE TONE, with ffmpeg on PATH
set -euo pipefail
file=$1 tone=$2
level() {
  ffmpeg -hide_banner -ss 0.5 -i "$file" -af "pan=mono|c0=c0$1,astats=measure_perchannel=RMS_level:measure_overall=none" \
    -f null - 2>&1 | awk '/RMS level dB/ { print $NF; exit }'
}
notch=",bandreject=f=$tone:width_type=h:w=300"
printf '%s\t%s\n' "$(level '')" "$(level "$notch$notch$notch$notch")"
