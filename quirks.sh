#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a tone of 1 kHz on BlackHole 16ch and BlackHole 2ch under the
# quirks of Core Audio devices, and records them with capture.py, each take
# as NAME.wav in OUT with a screenshot as NAME.png:
#
# - fallback: a show made on another Mac, whose outputs name a device ID
#   that this one lacks, along with the name of BlackHole 16ch;
# - rate-before and rate-after: BlackHole 16ch before and after its rate
#   goes from 48 to 96 kHz midway through the cue, as Dante Controller
#   changes that of the Dante Virtual Soundcard;
# - overrun: outputs on channels 1-2 of BlackHole 16ch and on 17-18, past
#   its channels, of which the first should play and the second warn;
# - default-before and default-after, each of both devices: an output on
#   the default device, before and after the default output of the system
#   goes from BlackHole 2ch to BlackHole 16ch midway through the cue.
#
# Usage: quirks.sh CINQ OUT, with Python with sounddevice and numpy, swift,
# and ffmpeg on PATH
set -euo pipefail
cinq=$(cd "$(dirname "$1")" && pwd -P)/$(basename "$1")
out=$(mkdir -p "$2" && cd "$2" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
pid=''
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  swift "$here/coreaudio.swift" rate "BlackHole 16ch" 48000 > /dev/null 2>&1 || true
}
trap cleanup EXIT

core() { swift "$here/coreaudio.swift" "$@" | tee -a "$out/coreaudio.log"; }
shot() { screencapture -x "$out/$1.png" || true; }
keys() {
  osascript -e 'tell application "System Events" to set frontmost of process "cinq" to true' || true
  sleep 1
  osascript -e "tell application \"System Events\" to $1" || true
}
# Records device $1 into $2.wav for $3 seconds, at rate $4
record() {
  python "$here/capture.py" record "$1" "$out/$2.wav" "${4:-48000}" > "$out/record-$2.log" 2>&1 &
  local recorder=$!
  sleep "$3"
  kill -INT "$recorder" 2> /dev/null || true
  wait "$recorder" 2> /dev/null || true
}
launch() {
  (cd "$work" && exec "$cinq" "$1.cinq.toml" > "$out/cinq-$1.log" 2>&1) &
  pid=$!
  sleep 8
  keys 'keystroke " "'
  sleep 2
}
quit() {
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 2
}
output() {
  printf '[[output]]\nname = "%s"\n' "$1"
  [ -n "$2" ] && printf 'device = "%s"\ndevice_name = "%s"\n' "$2" "$3"
  printf 'channels = "%s"\n\n' "$4"
}
cue() {
  printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "audio"\nname = "Tone"\nfile = "tone.wav"\n\n[list.cue.sends]\n'
  printf '%s\n' "$@"
}

core rate "BlackHole 16ch" 48000
core rate "BlackHole 2ch" 48000
core default "BlackHole 2ch"
ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=90" \
  -af volume=0.5 -ac 2 -c:a pcm_f32le "$work/tone.wav"
"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log"
cat "$out/devices.tsv"
sixteen=$(awk -F'\t' '$2 == "BlackHole 16ch" { print $1; exit }' "$out/devices.tsv")

{
  output "Ch 1" coreaudio:NotThisMac_UID "BlackHole 16ch" 1
  output "Ch 2" coreaudio:NotThisMac_UID "BlackHole 16ch" 2
  output "Ch 3" coreaudio:NotThisMac_UID "BlackHole 16ch" 3
  cue '"Ch 1" = 0.0' '"Ch 2" = -3.0' '"Ch 3" = -6.0'
} > "$work/fallback.cinq.toml"
{
  output Main "$sixteen" "BlackHole 16ch" 1-2
  cue 'Main = 0.0'
} > "$work/rate.cinq.toml"
{
  output Main '' '' 1-2
  cue 'Main = 0.0'
} > "$work/default.cinq.toml"
{
  output Low "$sixteen" "BlackHole 16ch" 1-2
  output High "$sixteen" "BlackHole 16ch" 17-18
  cue 'Low = 0.0' 'High = -6.0'
} > "$work/overrun.cinq.toml"
cp "$work"/*.cinq.toml "$out/"

launch overrun
record "BlackHole 16ch" overrun 5
shot overrun
quit

launch fallback
record "BlackHole 16ch" fallback 5
shot fallback
quit

launch rate
record "BlackHole 16ch" rate-before 4
core rate "BlackHole 16ch" 96000
sleep 1
record "BlackHole 16ch" rate-after 8 96000
shot rate-changed
core rate "BlackHole 16ch" 48000
quit

launch default
record "BlackHole 2ch" default-before-2ch 4 &
both=$!
record "BlackHole 16ch" default-before-16ch 4
wait "$both"
core default "BlackHole 16ch"
sleep 5
shot default-changed
record "BlackHole 2ch" default-after-2ch 4 &
both=$!
record "BlackHole 16ch" default-after-16ch 4
wait "$both"
core default "BlackHole 2ch"
quit
