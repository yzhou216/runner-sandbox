#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs the light cues of show.toml in cinq on Darwin, where the network that
# multicast leaves by is chosen otherwise than on GNU/Linux and Windows, and
# records the sACN and Art-Net that come back to the same computer, as
# Finder opens cinq.app: first as the build signs it, then signed again as a
# whole bundle.  Darwin asks before a program first reaches the local
# network, with the reason in cinq.app's Info.plist; ui, built from
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
python3 eos/listen.py "$out/dmx.jsonl" 150 "$out/wire.pcap" &
listener=$!
(sleep 180 && kill "$listener" 2> /dev/null) &
sleep 3
shot 0-listener
allow 10

# Opens APP as Finder does, presses Allow, and GO twice, under LABEL
opened() {
  mark "$2"
  open -n --stdout "$out/$2.log" --stderr "$out/$2.log" "$1" --args "$PWD/eos/show.toml"
  sleep 8
  shot "$2"
  allow 5
  local pid
  pid=$(pgrep -n -x cinq)
  go "$pid" "$2"
  kill "$pid"
  sleep 3
}

# cinq.app as the build signs it, its program alone under the name of the
# file that rustc wrote, and then a copy signed as a whole bundle, under the
# identifier of its Info.plist, which Darwin's privacy settings go by
codesign -dv --verbose=4 "$app" > "$out/codesign-built.txt" 2>&1
opened "$app" 1-built
# Files from the Nix store are read-only, which codesign needs to write
cp -R "$app" signed.app
chmod -R u+w signed.app
codesign --force --sign - --identifier io.yiyuzhou.cinq signed.app > "$out/codesign-signing.txt" 2>&1
codesign -dv --verbose=4 signed.app > "$out/codesign-signed.txt" 2>&1
opened "$PWD/signed.app" 2-signed
wait "$listener"
mark finished
log show --last 10m --style compact \
  --predicate 'process == "nehelper" OR subsystem == "com.apple.networkextension" OR eventMessage CONTAINS[c] "local network"' \
  > "$out/privacy.log" 2>&1 || true
