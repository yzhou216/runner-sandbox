#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 7: a config.toml that starts with the UTF-8 byte order mark and binds GO to F9.
# Starts silent, F9 is GO, an audio device chosen in the sidebar lands in the file after
# the mark, and a second start is silent too.
#
# Usage: bom.sh APPS OUT
set -uo pipefail
APPS=$1
OUT=$2
# shellcheck source=common2.sh
. darwin/common2.sh

mkdir -p "$W/tbom/sounds"
tone "$W/tbom/sounds/tone.wav" 20
cp darwin/show.cinq.toml "$W/tbom/show.cinq.toml"
APP=$APPS/cinq-new.app
mkdir -p "$W/conf-bom"
printf '\357\273\277[keys]\ngo = ["Space", "F9"]\n' > "$W/conf-bom/config.toml"
"$APP/Contents/MacOS/cinq" --audio-devices > "$OUT/devices.txt" 2>&1
{ echo "devices:"; cat "$OUT/devices.txt"; echo "config before:"; xxd "$W/conf-bom/config.toml"; } | tee "$OUT/bom.txt"

launch 7-bom "$(fresh bom1 tbom)" "$W/conf-bom"
shot start
echo "first start: standard error $(wc -c < "$D/cinq.log") bytes" | tee -a "$OUT/bom.txt"
code 101
sleep 2
shot after-f9
grep -c -F 'Active (1' "$D/after-f9.txt" | sed 's/^/Active (1 lines after F9: /' | tee -a "$OUT/bom.txt"
code 53
sleep 1
typ f
sleep 1
shot hints
label=$(python3 darwin/pick.py label "$D/hints.txt")
echo "audio device hint label: [$label]" | tee -a "$OUT/bom.txt"
if [ -n "$label" ]; then
  typ "$label"
  sleep 1
  shot list-open
  read -r x y < <(python3 darwin/pick.py item "$D/list-open.txt")
  echo "first entry below Default device: [$x $y]" | tee -a "$OUT/bom.txt"
  [ -n "${x:-}" ] && ./ui at "$x" "$y"
  sleep 2
  shot chosen
fi
quit
{ echo "config after the choice:"; xxd "$W/conf-bom/config.toml"; cat "$W/conf-bom/config.toml"; } | tee -a "$OUT/bom.txt"

launch 7-bom-again "$(fresh bom2 tbom)" "$W/conf-bom"
shot start
echo "second start: standard error $(wc -c < "$D/cinq.log") bytes" | tee -a "$OUT/bom.txt"
code 101
sleep 2
shot after-f9
grep -c -F 'Active (1' "$D/after-f9.txt" | sed 's/^/Active (1 lines after F9: /' | tee -a "$OUT/bom.txt"
quit
{ echo "config at the end:"; xxd "$W/conf-bom/config.toml"; } | tee -a "$OUT/bom.txt"
