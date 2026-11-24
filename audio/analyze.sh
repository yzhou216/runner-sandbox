#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Turns the recordings of record.sh in OUT into house.tsv and booth.tsv, the RMS
# level in dB of each window of 100 ms (-120 for silence) with the wall clock
# time of its start, which the calibration beep of each sink fixes, since
# parecord starts about 1.5 seconds after it is started.  Prints the offset.
#
# Usage: analyze.sh OUT, with ffmpeg and gawk on PATH
set -euo pipefail
out=$1
for sink in house booth; do
  ffmpeg -nostdin -hide_banner -loglevel error -i "$out/$sink.wav" \
    -af "asetnsamples=n=4800:p=0,astats=metadata=1:reset=1,ametadata=print:key=lavfi.astats.Overall.RMS_level:file=-" -f null - |
    gawk '/^frame:/ { match($0, /pts_time:[0-9.]+/); time = substr($0, RSTART + 9, RLENGTH - 9) }
      /RMS_level=/ { split($0, v, "="); level = (v[2] == "-inf" ? -120 : v[2]); printf "%.1f\t%.1f\n", time, level }' \
    > "$out/$sink.rec.tsv"
  calibration=$(gawk -F'\t' -v sink="$sink" '$2 == "calibrate-" sink { print $1 }' "$out/marks.tsv")
  # The first window above -40 dB after the first second is the beep
  gawk -F'\t' -v wall="$calibration" -v sink="$sink" '
    $1 > 1 && $2 > -40 && !found { offset = $1 - wall; found = 1; printf "%s: beep at %.1f s in the recording, offset %.2f s\n", sink, $1, offset > "/dev/stderr" }
    { rows[NR] = $0 }
    END { for (i = 1; i <= NR; i++) { split(rows[i], f, "\t"); printf "%.2f\t%s\n", f[1] - offset, f[2] } }' \
    "$out/$sink.rec.tsv" > "$out/$sink.tsv"
done
