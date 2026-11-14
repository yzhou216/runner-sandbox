#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs the light cues of show.toml in cinq on Darwin, where the network that
# multicast leaves by is chosen otherwise than on GNU/Linux and Windows,
# and records the sACN and Art-Net that come back to the same computer.
#
# Usage: darwin.sh CINQ OUT
set -euo pipefail

cinq=$1
out=$2
mkdir -p "$out"
ifconfig > "$out/ifconfig.txt"
python3 eos/listen.py "$out/dmx.jsonl" 45 "$out/wire.pcap" &
listener=$!
sleep 1
ICED_BACKEND=tiny-skia "$cinq" eos/show.toml > "$out/cinq.log" 2>&1 &
pid=$!
sleep 10
for cue in 1 2; do
  osascript -e "tell application \"System Events\" to set frontmost of (first process whose unix id is $pid) to true" || true
  osascript -e 'tell application "System Events" to keystroke " "' || true
  sleep 8
  screencapture -x "$out/cue-$cue.png"
done
wait "$listener"
kill "$pid" || true
