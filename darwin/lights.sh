#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs lights.toml in cinq.app as Finder opens it, with listen.py beside it
# on the same computer.  The first GO goes before Allow on Darwin's Local
# Network prompt, to see what goes out until it is answered; Allow, an idle
# stretch of 13 seconds for the refresh and the discovery, a fade of four
# seconds, and out follow.
#
# Usage: lights.sh APP OUT, with APP the path of cinq.app
set -uo pipefail

app=$1
OUT=$2
mkdir -p "$OUT"
# shellcheck source=common.sh
. darwin/common.sh
system
codesign -dv --verbose=4 "$app" > "$OUT/codesign.txt" 2>&1 || true
codesign --verify --deep --strict --verbose=2 "$app" >> "$OUT/codesign.txt" 2>&1 || true

mark listening
python3 darwin/listen.py "$OUT/dmx.jsonl" 150 "$OUT/wire.pcap" &
listener=$!
sleep 3
shot 0-listener
# The listener is a program too, and may be asked first
allow 6 listener || true

mark "open cinq.app"
open -n --stdout "$OUT/cinq.log" --stderr "$OUT/cinq.log" "$app" --args "$PWD/darwin/lights.toml"
sleep 8
pid=$(pgrep -n -x cinq)
shot 1-opened
go "$pid" 1
sleep 4
shot 2-go1-unanswered
cp "$OUT/cinq.log" "$OUT/cinq-before-allow.log"
allow 6 cinq || true
shot 3-after-allow
mark "idle start"
sleep 13
mark "idle end"
go "$pid" 2
sleep 6
shot 4-fade-done
go "$pid" 3
sleep 5
mark "closing"
kill "$pid"
wait "$listener" || true
mark finished

python3 darwin/lights-verdict.py "$OUT" | tee "$OUT/verdict.txt"
