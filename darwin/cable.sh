#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs cable.toml in cinq.app against a stand-in console at the Ion Xe's
# address, 10.101.100.20, aliased on lo0, with pf as the cable: a block of
# TCP to and from the console's port 3037 drops its packets as a pulled cable
# does.  A cue goes before the cable is pulled, one while it is out for 15
# seconds, two once it is back, then the cable comes out for 7 seconds and a
# cue goes a second after it came out, and one cue goes after it is back.  A
# cue sent into a cable that is out must never arrive, even once the cable
# is back (TCP delivered one 13 seconds late before cinq made a new
# connection after each quiet second); every other cue must arrive within a
# second of its GO.
#
# Usage: cable.sh APP OUT
set -uo pipefail

app=$1
OUT=$2
mkdir -p "$OUT"
# shellcheck source=common.sh
. darwin/common.sh

console=10.101.100.20
sudo ifconfig lo0 alias "$console"
ifconfig lo0 > "$OUT/lo0.txt"
printf 'block drop quick on lo0 proto tcp from any to %s port 3037\nblock drop quick on lo0 proto tcp from %s port 3037 to any\n' "$console" "$console" > "$OUT/pf-out.conf"
printf '# Cable in\n' > "$OUT/pf-in.conf"
cable() {
  if [ "$1" = out ]; then conf=pf-out.conf; else conf=pf-in.conf; fi
  # shellcheck disable=SC2024 # The file is the runner's own
  sudo pfctl -f "$OUT/$conf" >> "$OUT/pfctl.txt" 2>&1
  mark "cable $1"
}
# Whether a plain connection to the console gets through within 2 seconds
reach() {
  if nc -z -w 2 "$console" 3037 > /dev/null 2>&1; then r=reaches; else r="does not reach"; fi
  mark "nc $r the console ($1)"
}
# shellcheck disable=SC2024 # The file is the runner's own
sudo pfctl -E >> "$OUT/pfctl.txt" 2>&1
cable in
python3 darwin/fakeeos.py "$OUT/console.txt" 140 3037 8000 "$console" &
standin=$!
sleep 1
reach "cable in"
cable out
reach "cable out"
cable in
reach "cable back"

mark "open cinq.app"
: > "$OUT/cinq.log"
stamped "$OUT/cinq.log"
open -n --stdout "$OUT/cinq.log" --stderr "$OUT/cinq.log" "$app" --args "$PWD/darwin/cable.toml"
sleep 8
pid=$(pgrep -n -x cinq)
shot 1-opened
allow 3 cable || true
go "$pid" before
sleep 3
cable out
sleep 15
reach "cable out 15 s"
shot 3-cable-pulled
go "$pid" "while out"
sleep 3
shot 4-go-while-out
cable in
sleep 3
go "$pid" after
sleep 3
go "$pid" "after again"
sleep 3
cable out
sleep 1
go "$pid" blip
sleep 6
cable in
sleep 20
shot 5-blip
go "$pid" "after blip"
sleep 4
shot 6-end
kill "$pid"
# shellcheck disable=SC2154 # Set by stamped in common.sh
kill "$watcher" 2> /dev/null || true
# shellcheck disable=SC2024 # The file is the runner's own
sudo pfctl -d >> "$OUT/pfctl.txt" 2>&1 || true
wait "$standin" 2> /dev/null || true
mark finished

# What arrived, and how long after its GO, against what each GO should have
# sent: everything but what went while the cable was out
python3 darwin/cable-verdict.py "$OUT" | tee "$OUT/verdict.txt"
