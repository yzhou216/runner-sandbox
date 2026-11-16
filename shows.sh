#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Writes a show to standard output, for the scripts of Windows and Darwin:
#
# - keys DEVICE NAME: Left on channel 1 and Right on channel 2 of the
#   device, the default one where DEVICE is empty, and one cue of
#   stereo.wav sending to both at 0 dB
# - clamp DEVICE NAME SHOW: Ch 1 on channel 1 of the device, and the cue or
#   cues of the show SHOW of clamp.sh: single, two, or boost
# - two DEVICE NAME DEVICE NAME: House and Booth on channels 1-2 of two
#   devices, and one cue of stereo.wav sending to House at 0 dB and to Booth
#   at -6 dB
#
# Usage: shows.sh KIND ARGUMENT...
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=show.sh
. "$here/show.sh"
kind=$1
shift
case $kind in
  keys)
    output Left "$1" 1 "$2"
    output Right "$1" 2 "$2"
    list
    cue 1 Tone stereo.wav manual 'Left=0.0' 'Right=0.0'
    ;;
  clamp)
    output 'Ch 1' "$1" 1 "$2"
    list
    case $3 in
      single) cue 1 Loud clip.wav manual 'Ch 1=0.0' ;;
      two)
        cue 1 First clip.wav continue 'Ch 1=0.0'
        cue 2 Second clip.wav manual 'Ch 1=0.0'
        ;;
      boost) cue 1 Boosted half.wav manual 'Ch 1=12.0' ;;
    esac
    ;;
  two)
    output House "$1" 1-2 "$2"
    output Booth "$3" 1-2 "$4"
    list
    cue 1 Tone stereo.wav manual 'House=0.0' 'Booth=-6.0'
    ;;
esac
