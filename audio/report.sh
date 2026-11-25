#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the levels that each check of the audio session measured, from the
# house.tsv and booth.tsv of analyze.sh and the marks of record.sh in OUT
#
# Usage: report.sh OUT, with gawk on PATH
set -euo pipefail
out=$1
m() { gawk -F'\t' -v name="$1" '$2 == name { print $1 }' "$out/marks.tsv" | head -1; }
# Mean level of SINK from mark A plus OFFSET1 to mark B plus OFFSET2
mean() {
  gawk -F'\t' -v lo="$(gawk -v a="$(m "$2")" -v o="$3" 'BEGIN { printf "%.3f", a + o }')" \
    -v hi="$(gawk -v a="$(m "$4")" -v o="$5" 'BEGIN { printf "%.3f", a + o }')" \
    '$1 >= lo && $1 < hi { s += $2; n++ } END { if (n) printf "%.1f", s / n; else printf "n/a" }' "$out/$1.tsv"
}
# Levels of SINK in steps of STEP seconds from mark A plus OFFSET1 for N steps
steps() {
  gawk -F'\t' -v sink="$1" -v a="$(m "$2")" -v o="$3" -v step="$4" -v n="$5" '
    { t[NR] = $1; v[NR] = $2 }
    END { for (i = 0; i < n; i++) {
            lo = a + o + i * step; hi = lo + step; s = 0; c = 0
            for (j = 1; j <= NR; j++) if (t[j] >= lo && t[j] < hi) { s += v[j]; c++ }
            printf "%+.1f:%s ", o + i * step, (c ? sprintf("%.1f", s / c) : "n/a")
          }
          print "" }' "$out/$1.tsv"
}
# Seconds from mark A until SINK stays below -60 dB for 0.3 s, and the last window above it
silence() {
  gawk -F'\t' -v a="$(m "$2")" '
    $1 >= a { n++; t[n] = $1; v[n] = $2 }
    END { for (i = 1; i <= n - 2; i++) if (v[i] < -60 && v[i+1] < -60 && v[i+2] < -60) { printf "silent from %.1f s after the mark (last sound %.1f s)", t[i] - a, (i > 1 ? t[i-1] - a : 0); exit }
          print "not silent" }' "$out/$1.tsv"
}
echo "check 1: house $(mean house go-plain 2 end-plain -0.3) dB, booth $(mean booth go-plain 2 end-plain -0.3) dB (expected house -12.0 RMS)"
echo "check 2: house $(mean house go-1b 2 go-1b 5) dB, booth $(mean booth go-1b 2 go-1b 5) dB (expected -12.0 and -22.0 RMS)"
echo "check 6: before house $(mean house typed-level -2 typed-level -0.3) booth $(mean booth typed-level -2 typed-level -0.3); after house $(mean house typed-level 1.5 end-6 0) booth $(mean booth typed-level 1.5 end-6 0) (expected -18.0 and -28.0)"
echo "check 3 (house, offsets from the mark of GO, 0.5 s steps):"; steps house go-2 -1 0.5 16
echo "check 3 (booth):"; steps booth go-2 -1 0.5 16
echo "check 5 stop: house $(silence house go-3); booth $(silence booth go-3); before $(mean house go-3 -1 go-3 0)"
echo "check 5 stop (house, 0.1 s steps):"; steps house go-3 -0.5 0.1 25
echo "check 5 panic: house $(silence house panic); booth $(silence booth panic); before $(mean house panic -1 panic 0)"
echo "check 5 panic (house, 0.1 s steps):"; steps house panic -0.5 0.1 25
echo "check 4 (house, 0.5 s steps from GO of cue 4):"; steps house go-4 0 0.5 32
echo "check 4 devamp mark at +$(gawk -v a="$(m go-4)" -v b="$(m go-5)" 'BEGIN { printf "%.1f", b - a }') s"
echo "check 8: before house $(mean house cleared-house -3 cleared-house 0) booth $(mean booth cleared-house -3 cleared-house 0); house after its send cleared: $(silence house cleared-house); booth after the last send cleared: $(silence booth cleared-sends); booth before it $(mean booth cleared-sends -1 cleared-sends -0.2)"
echo "check 8 (house, 0.1 s steps):"; steps house cleared-sends -0.5 0.1 25
echo "check 7: before unload house $(mean house unload-booth -2 unload-booth 0) booth $(mean booth unload-booth -2 unload-booth 0); after house $(mean house unload-booth 1 end-7 0) booth $(mean booth unload-booth 1 end-7 0)"
echo "check 7 (booth, 0.2 s steps):"; steps booth unload-booth -0.4 0.2 20
