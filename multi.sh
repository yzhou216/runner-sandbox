#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a show on two Core Audio devices at once: BlackHole 64ch, which
# has as many channels as the Dante Virtual Soundcard offers on Darwin,
# and BlackHole 2ch beside it, as the soundcard's WDM mode on Windows
# offers each pair of channels as a device of its own.  One ffmpeg records
# both devices, so that their recordings share the clock of the host and
# tell how far apart the two devices start the cue.
#
# Usage: multi.sh CINQ SOUND OUT, with ffmpeg on PATH, CINQ the program,
# SOUND the sound of the cue, and OUT the directory for the recordings,
# the screenshots, and the levels
set -euo pipefail
cinq=$(cd "$(dirname "$1")" && pwd -P)/$(basename "$1")
sound=$(cd "$(dirname "$2")" && pwd -P)/$(basename "$2")
out=$(mkdir -p "$3" && cd "$3" && pwd)
work=$(mktemp -d)
pid='' recorder=''
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$recorder" ] && kill -INT "$recorder" 2> /dev/null || true
}
trap cleanup EXIT

shot() { screencapture -x "$out/$1.png" || true; }
keys() {
  osascript -e 'tell application "System Events" to set frontmost of process "cinq" to true' || true
  sleep 1
  osascript -e "tell application \"System Events\" to $1" || true
}

"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log"
cat "$out/devices.tsv"
id() { awk -F'\t' -v name="$1" '$2 == name { print $1; exit }' "$out/devices.tsv"; }
two=$(id "BlackHole 2ch") many=$(id "BlackHole 64ch")
[ -n "$two" ] && [ -n "$many" ] || { echo "cinq lists no BlackHole 2ch or 64ch"; exit 1; }

cp "$sound" "$work/music.flac"
output() {
  printf '[[output]]\nname = "%s"\ndevice = "%s"\ndevice_name = "%s"\nchannels = "%s"\n\n' "$@"
}
{
  output House "$two" "BlackHole 2ch" 1-2
  output Sub "$many" "BlackHole 64ch" 3
  output Monitors "$many" "BlackHole 64ch" 33-34
  output Lobby "$many" "BlackHole 64ch" 63-64
  printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\n'
  printf 'kind = "audio"\nname = "Music"\nfile = "music.flac"\n\n'
  printf '[list.cue.sends]\nHouse = 0.0\nSub = 0.0\nMonitors = -6.0\nLobby = -12.0\n'
} > "$work/multi.cinq.toml"
cp "$work/multi.cinq.toml" "$out/"

ffmpeg -hide_banner -y -f avfoundation -i ":BlackHole 2ch" -f avfoundation -i ":BlackHole 64ch" \
  -map 0:a -c:a pcm_f32le "$out/two.wav" -map 1:a -c:a pcm_f32le "$out/many.wav" \
  > "$out/record.log" 2>&1 &
recorder=$!
(cd "$work" && exec "$cinq" multi.cinq.toml > "$out/cinq.log" 2>&1) &
pid=$!
sleep 10
shot standby
keys 'keystroke " "'
sleep 8
shot playing
kill -INT "$recorder" 2> /dev/null || true
wait "$recorder" 2> /dev/null || true
recorder=''
