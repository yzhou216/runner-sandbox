#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the show, records cinq playing it, and composes the video NAME.mp4
# in OUT, together with levels.md, the sound's level at the end of each
# wait.
#
# Usage: run.sh CINQ NAME OUT, with the tools of record.sh and compose.sh on
# PATH, CINQ a cinq program or a directory with bin/cinq, ALSA_PLUGIN_DIR
# naming the directory of the pulse plugin of ALSA, and FONTS naming the
# directory of the share/fonts of DejaVu
set -euo pipefail
here=$(dirname "$(readlink -f "$0")")
cinq=$(readlink -f "$1") name=$2
[ -d "$cinq" ] && cinq=$cinq/bin/cinq
test -x "$cinq"
out=$(mkdir -p "$3" && readlink -f "$3")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

bash "$here/make-show.sh" "$work/show"
# One try more, as a runner can be slow to start sway or to take a key
for attempt in 1 2; do
  rm -rf "${out:?}"/*
  if timeout -k 10 300 bash "$here/record.sh" "$cinq" "$work/show" "$out" "$FONTS"; then break; fi
  [ "$attempt" = 2 ] && exit 1
done
bash "$here/compose.sh" "$out" "$FONTS/truetype/DejaVuSans.ttf" "$out/$name.mp4" \
  "Heard level: RMS in dB relative to full scale, over 100 ms ($name)"
