#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays the show of show.sh on BlackHole 16ch, which Over on channels 17-18
# overruns, takes screenshots of the main window, the Levels tab, the
# outputs view, and the inspector of Stage L, and records BlackHole 16ch
# with capture.py as house.wav, all in OUT.
#
# Usage: darwin.sh CINQ OUT, with Python with sounddevice and numpy, and
# ffmpeg on PATH
set -euo pipefail
cinq=$(cd "$(dirname "$1")" && pwd -P)/$(basename "$1")
out=$(mkdir -p "$2" && cd "$2" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
pid=''
trap '[ -n "$pid" ] && kill "$pid" 2> /dev/null || true' EXIT

shot() { screencapture -x "$out/$1.png" || true; }
keys() {
  osascript -e 'tell application "System Events" to set frontmost of process "cinq" to true' || true
  sleep 0.5
  osascript -e "tell application \"System Events\" to keystroke \"$1\"" || true
  sleep 1
}

ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=60" \
  -af volume=0.5 -ac 2 -c:a pcm_f32le "$work/tone.wav"
"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log"
id=$(awk -F'\t' '$2 == "BlackHole 16ch" { print $1; exit }' "$out/devices.tsv")
bash "$here/show.sh" "$id" "BlackHole 16ch" 17-18 coreaudio:DanteVirtualSoundcard_UID > "$work/ui.cinq.toml"
cp "$work/ui.cinq.toml" "$out/"

(cd "$work" && exec "$cinq" ui.cinq.toml > "$out/cinq.log" 2>&1) &
pid=$!
sleep 8
keys ' '
python "$here/capture.py" record "BlackHole 16ch" "$out/house.wav" > "$out/record.log" 2>&1 &
recorder=$!
sleep 1
shot main
keys gt
keys gt
shot levels
keys l
shot outputs
keys jjj
shot inspector
sleep 1
kill -INT "$recorder" 2> /dev/null || true
wait "$recorder" 2> /dev/null || true
