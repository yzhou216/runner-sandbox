#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs net.toml in cinq.app against fakeeos.py on the same computer: TCP and
# UDP with three kinds of argument, a destination that refuses, a burst of
# three cues 0.3 seconds apart, three cues 4 seconds apart, and a stand-in
# that stops for one GO and comes back for the next.  Times of what arrives
# and of the warnings on standard error go to console.txt and
# cinq.log.stamped.
#
# Usage: net.sh APP OUT
set -uo pipefail

app=$1
OUT=$2
mkdir -p "$OUT"
# shellcheck source=common.sh
. darwin/common.sh

python3 darwin/fakeeos.py "$OUT/console.txt" 150 3037 8000 127.0.0.1 &
console=$!
sleep 1
mark "open cinq.app"
: > "$OUT/cinq.log"
stamped "$OUT/cinq.log"
open -n --stdout "$OUT/cinq.log" --stderr "$OUT/cinq.log" "$app" --args "$PWD/darwin/net.toml"
sleep 8
pid=$(pgrep -n -x cinq)
shot 1-opened
# Reaching the LAN may ask again for this run's program
allow 3 net || true
go "$pid" tcp
sleep 3
go "$pid" udp
sleep 3
go "$pid" refused
sleep 4
shot 2-refused
go "$pid" burst 3 0.3
sleep 4
go "$pid" apart-1
sleep 4
go "$pid" apart-2
sleep 4
go "$pid" apart-3
sleep 4
# Console gone for one GO, then back
kill "$console"
wait "$console" 2> /dev/null || true
mark "console stopped"
sleep 1
go "$pid" down
sleep 4
shot 3-console-down
python3 darwin/fakeeos.py "$OUT/console.txt" 25 3037 8000 127.0.0.1 &
console=$!
mark "console restarted"
sleep 2
go "$pid" back
sleep 4
shot 4-console-back
kill "$pid"
# shellcheck disable=SC2154 # Set by stamped in common.sh
kill "$watcher" 2> /dev/null || true
wait "$console" 2> /dev/null || true
mark finished

python3 darwin/net-verdict.py "$OUT" | tee "$OUT/verdict.txt"
