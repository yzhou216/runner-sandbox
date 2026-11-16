#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 5 and check 2 on Darwin, with cinq of the tree in cinq/, whose log
# shows what each key does.  BlackHole 16ch and BlackHole 2ch, virtual Core
# Audio devices, stand in for sound cards, and capture.py records them.  Keys
# go through System Events, and GO is F5, so that no Space and no Escape
# reaches cinq except where the check of Panic needs Escape.
#
# - default: with BlackHole 16ch as the default output, Left on channel 1 and
#   Right on channel 2 of the default device, where the outputs view takes
#   `l`, `j`, `o`, `d d`, `p`, and Control+o, the cue plays, and the Levels
#   tab, reached with `g t` twice, takes Tab three times to the send of Right,
#   to which -12 and Enter give -12 dB; each take is recorded before and after
# - explicit: the same show on BlackHole 16ch by its ID
# - two: House on BlackHole 16ch and Booth on BlackHole 2ch, both recorded,
#   with Panic (Escape) stopping both
#
# Usage: darwin.sh CINQ SOUND OUT, with Python with sounddevice and numpy, and
# ffmpeg on PATH, SOUND the directory of media.sh's sounds, and OUT the
# directory for the logs, the recordings, and the screenshots
set -euo pipefail
cinq=$(cd "$(dirname "$1")" && pwd -P)/$(basename "$1")
sound=$(cd "$2" && pwd -P)
out=$(mkdir -p "$3" && cd "$3" && pwd)
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
pid='' recorders=()
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ "${#recorders[@]}" -gt 0 ] && kill -INT "${recorders[@]}" 2> /dev/null || true
}
trap cleanup EXIT
cp "$sound/stereo.wav" "$work/"

mark() { printf '%s\t%s\n' "$(python -c 'import time; print(f"{time.time():.3f}")')" "$2" >> "$out/$1.marks"; }
shot() { screencapture -x "$out/$1.png" || true; }
# send PHASE LABEL STATEMENT: System Events statement, after raising cinq
send() {
  osascript -e 'tell application "System Events" to set frontmost of process "cinq" to true' || true
  sleep 0.5
  mark "$1" "$2"
  osascript -e "tell application \"System Events\" to $3" || true
  sleep 1.5
}
record() { # NAME DEVICE: records DEVICE until stopped, into OUT/NAME.wav
  python "$here/capture.py" record "$2" "$out/$1.wav" > "$out/$1.rec.log" 2>&1 &
  recorders+=("$!")
}
stop_recorders() {
  kill -INT "${recorders[@]}" 2> /dev/null || true
  wait "${recorders[@]}" 2> /dev/null || true
  recorders=()
}
start() { # PHASE: starts cinq on the show PHASE.cinq.toml of the work directory
  mkdir -p "$work/cfg-$1"
  printf 'keymap = "vi"\n\n[keys]\ngo = ["F5"]\n' > "$work/cfg-$1/config.toml"
  cp "$work/$1.cinq.toml" "$out/"
  : > "$out/$1.marks"
  (cd "$work" && exec "$cinq" --config-dir "$work/cfg-$1" "$1.cinq.toml" 2> "$out/$1.log") &
  pid=$!
  mark "$1" start
  sleep 10
}
stop() {
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 3
}

"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log"
cat "$out/devices.tsv"
id16=$(awk -F'\t' '$2 == "BlackHole 16ch" { print $1; exit }' "$out/devices.tsv")
id2=$(awk -F'\t' '$2 == "BlackHole 2ch" { print $1; exit }' "$out/devices.tsv")

for phase in default explicit; do
  if [ "$phase" = default ]; then
    bash "$here/shows.sh" keys - - > "$work/$phase.cinq.toml"
  else
    bash "$here/shows.sh" keys "$id16" "BlackHole 16ch" > "$work/$phase.cinq.toml"
  fi
  start "$phase"
  shot "$phase-start"
  send "$phase" 'outputs l' 'keystroke "l"'
  send "$phase" 'outputs j' 'keystroke "j"'
  send "$phase" 'outputs o' 'keystroke "o"'
  shot "$phase-outputs"
  send "$phase" 'outputs d d' 'keystroke "dd"'
  send "$phase" 'outputs p' 'keystroke "p"'
  send "$phase" 'outputs ctrl-o' 'keystroke "o" using control down'
  shot "$phase-back"
  record "$phase-before" "BlackHole 16ch"
  sleep 1
  send "$phase" 'go F5' 'key code 96'
  sleep 5
  shot "$phase-playing"
  stop_recorders
  send "$phase" 'levels g t 1' 'keystroke "gt"'
  send "$phase" 'levels g t 2' 'keystroke "gt"'
  shot "$phase-levels"
  for n in 1 2 3; do send "$phase" "levels tab $n" 'key code 48'; done
  send "$phase" 'levels select all' 'keystroke "a" using command down'
  send "$phase" 'levels send' 'keystroke "-12"'
  send "$phase" 'levels enter' 'key code 36'
  shot "$phase-sent"
  record "$phase-after" "BlackHole 16ch"
  sleep 6
  stop_recorders
  stop
done

bash "$here/shows.sh" two "$id16" "BlackHole 16ch" "$id2" "BlackHole 2ch" > "$work/two.cinq.toml"
record two-house "BlackHole 16ch"
record two-booth "BlackHole 2ch"
start two
send two 'go F5' 'key code 96'
sleep 6
send two 'panic Escape' 'key code 53'
sleep 4
shot two-after
stop_recorders
stop
