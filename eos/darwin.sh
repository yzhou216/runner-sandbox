#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs the light cues of show.toml in cinq on Darwin, where the network that
# multicast leaves by is chosen otherwise than on GNU/Linux and Windows, and
# records the sACN and Art-Net that come back to the same computer: first
# the program of cinq.app as a shell starts it, then cinq.app as Finder
# opens it.  Darwin asks before a program first reaches the local network,
# the second time with the reason in cinq.app's Info.plist; ui, built from
# screen.swift, presses Allow as the operator would.
#
# Usage: darwin.sh APP OUT, with APP the path of cinq.app
set -uo pipefail

app=$1
out=$2
mkdir -p "$out"
swiftc -O -o ui eos/screen.swift
ifconfig > "$out/ifconfig.txt"
# Times of what happens, in seconds since the epoch, which BSD date gives
# whole
mark() { printf '%s %s\n' "$(python3 -c 'import time; print(time.time())')" "$1" >> "$out/marks.txt"; }
shot() { screencapture -x "$out/$1.png"; ./ui list > "$out/$1.txt" 2>&1; }
# Presses every Allow that comes up within the first SECONDS
allow() {
  for _ in 1 2 3; do
    # Exact text, since Don't Allow holds Allow too and comes first
    ./ui click Allow "$1" || break
    mark "pressed Allow"
    sleep 1
  done
}
# Presses GO twice in the cinq of process PID, with a screenshot after each
go() {
  for cue in 1 2; do
    osascript -e "tell application \"System Events\" to set frontmost of (first process whose unix id is $1) to true" || true
    osascript -e 'tell application "System Events" to keystroke " "' || true
    mark "GO $cue"
    sleep 8
    shot "$2-cue-$cue"
  done
}

mark listening
python3 eos/listen.py "$out/dmx.jsonl" 220 "$out/wire.pcap" &
listener=$!
(sleep 250 && kill "$listener" 2> /dev/null) &
sleep 3
shot 0-listener
allow 10

mark binary
"$app/Contents/MacOS/cinq" eos/show.toml > "$out/binary.log" 2>&1 &
pid=$!
sleep 8
shot 1-binary
allow 5
go "$pid" 1-binary
kill "$pid"
sleep 3

mark app
open -n --stdout "$out/app.log" --stderr "$out/app.log" "$app" --args "$PWD/eos/show.toml"
sleep 8
shot 2-app
allow 5
pid=$(pgrep -n -x cinq)
go "$pid" 2-app
kill "$pid"
sleep 3

# Same app again, now that the operator has allowed it
mark again
open -n --stdout "$out/again.log" --stderr "$out/again.log" "$app" --args "$PWD/eos/show.toml"
sleep 8
shot 3-again
allow 5
pid=$(pgrep -n -x cinq)
go "$pid" 3-again
kill "$pid"
wait "$listener"
mark finished
