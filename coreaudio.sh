#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a show on BlackHole 16ch, a virtual Core Audio device of sixteen
# channels that stands in for the Dante Virtual Soundcard, which is a Core
# Audio device of up to 64 channels on Darwin, and records the sixteen
# channels that BlackHole loops back.  Midway through the cue, the device
# goes away and comes back, as the Dante Virtual Soundcard does when it
# restarts: its driver leaves the plug-ins of Core Audio, which restarts,
# and then returns.  cinq should take the device up again by itself and go
# on with the cue from where it is.
#
# Usage: coreaudio.sh CINQ SOUND OUT, with ffmpeg on PATH, CINQ the
# program, SOUND the sound of the cue, and OUT the directory for the
# recordings, the screenshots, and marks.tsv
set -euo pipefail
cinq=$1 sound=$2
out=$(mkdir -p "$3" && cd "$3" && pwd)
work=$(mktemp -d)
driver=/Library/Audio/Plug-Ins/HAL/BlackHole16ch.driver
pid='' recorder=''
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$recorder" ] && kill -INT "$recorder" 2> /dev/null || true
  [ -d "$work/BlackHole16ch.driver" ] && sudo mv "$work/BlackHole16ch.driver" "$driver" || true
}
trap cleanup EXIT

mark() { printf '%s\t%s\n' "$(date +%s)" "$*" >> "$out/marks.tsv"; }
shot() { screencapture -x "$out/$1.png" || true; }
# Records the sixteen channels of BlackHole into $1 until stopped
record() {
  ffmpeg -loglevel warning -y -f avfoundation -i ":BlackHole 16ch" -c:a pcm_f32le \
    "$out/$1.wav" > "$out/$1.log" 2>&1 &
  recorder=$!
}
stop() {
  kill -INT "$recorder" 2> /dev/null || true
  wait "$recorder" 2> /dev/null || true
  recorder=''
}
restart_core_audio() {
  sudo killall coreaudiod || true
  sleep 5
}

"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log"
cat "$out/devices.tsv"
id=$(awk -F'\t' '$2 == "BlackHole 16ch" { print $1; exit }' "$out/devices.tsv")
[ -n "$id" ] || { echo "cinq lists no BlackHole 16ch"; exit 1; }
ffmpeg -hide_banner -f avfoundation -list_devices true -i "" > "$out/avfoundation.log" 2>&1 || true

cp "$sound" "$work/music.flac"
output() {
  printf '[[output]]\nname = "%s"\ndevice = "%s"\ndevice_name = "BlackHole 16ch"\nchannels = "%s"\n\n' \
    "$1" "$id" "$2"
}
{
  output House 1-2
  output Sub 3
  output Monitors 5-6
  output Lobby 15-16
  printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\n'
  printf 'kind = "audio"\nname = "Music"\nfile = "music.flac"\n\n'
  printf '[list.cue.sends]\nHouse = 0.0\nSub = 0.0\nMonitors = -6.0\nLobby = -12.0\n'
} > "$work/blackhole.cinq.toml"
cp "$work/blackhole.cinq.toml" "$out/"

record before
(cd "$work" && exec "$cinq" blackhole.cinq.toml > "$out/cinq.log" 2>&1) &
pid=$!
sleep 10
osascript -e 'tell application "System Events" to set frontmost of process "cinq" to true' || true
sleep 1
osascript -e 'tell application "System Events" to keystroke " "' || true
mark go
sleep 8
shot playing
mark gone
stop
sudo mv "$driver" "$work/"
restart_core_audio
shot gone
sleep 5
shot waiting
sudo mv "$work/BlackHole16ch.driver" "$driver"
restart_core_audio
mark back
# cinq tries every two seconds
sleep 4
record after
sleep 8
shot back
stop
mark end
"$cinq" --audio-devices > "$out/devices-after.tsv" 2>&1 || true
