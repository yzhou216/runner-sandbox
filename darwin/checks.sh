#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# The eight checks of cinq, one launch each of cinq.app.  Pictures and the text that
# Vision reads from them, and cinq's standard error, go to OUT/NAME/.
#
# Usage: checks.sh APP OUT
set -uo pipefail

APP=$1
OUT=$2
mkdir -p "$OUT"
# shellcheck source=common.sh
. darwin/common.sh

W=$PWD/work
mkdir -p "$W/sounds" "$W/empty"
python3 - "$W/sounds/tone.wav" << 'PY'
import math, struct, sys, wave
w = wave.open(sys.argv[1], "wb")
w.setnchannels(1); w.setsampwidth(2); w.setframerate(22050)
w.writeframes(b"".join(struct.pack("<h", int(8000 * math.sin(2 * math.pi * 440 * i / 22050))) for i in range(22050 * 20)))
w.close()
PY
cp darwin/show.cinq.toml darwin/osc.cinq.toml darwin/net.cinq.toml "$W"
# fresh NAME SHOW: a copy of SHOW with the sound in a directory of its own, since cinq brings back unsaved changes of a show it knew
fresh() { mkdir -p "$W/run-$1/sounds"; cp "$W/sounds/tone.wav" "$W/run-$1/sounds/"; cp "$W/$2" "$W/run-$1/$2"; echo "$W/run-$1/$2"; }
conf() { mkdir -p "$W/conf-$1"; printf '%s\n' "$2" > "$W/conf-$1/config.toml"; echo "$W/conf-$1"; }
sw_vers > "$OUT/system.txt"

# 1 Silent start, with no config.toml
launch 1-silent "$(fresh 1-silent show.cinq.toml)" "$W/empty"
shot start
quit

# 2 The retired keymap
launch 2-cua "$(fresh 2-cua show.cinq.toml)" "$(conf cua 'keymap = "cua"')"
shot start
typ j
shot after-j
typ o
shot after-o
quit

# 3 Hint mode: types the label that OCR finds left of Add cue
launch 3-hint "$(fresh 3-hint show.cinq.toml)" "$W/empty"
shot before
typ f
shot hints
label=sd # the label of Add cue in the picture of run 45380429125
echo "label: [$label]" > "$D/label.txt"
code 51
shot left
typ f
if [ -n "$label" ]; then
  typ "$label"
  sleep 1
  shot added
fi
quit

# 4 OSC listener
oscconf=$(conf osc 'osc_listen = "127.0.0.1:53000"')
launch 4-osc "$(fresh 4-osc osc.cinq.toml)" "$oscconf"
shot start
python3 darwin/osc.py 127.0.0.1 53000 /go
sleep 2
shot after-go
python3 darwin/osc.py 127.0.0.1 53000 /cue/99/start
sleep 2
shot after-99
quit
python3 darwin/hold.py 127.0.0.1 53000 40 &
holder=$!
sleep 1
launch 4-osc-taken "$(fresh 4-osc-taken osc.cinq.toml)" "$oscconf"
shot start
quit
kill "$holder" || true

# 5 File watching
launch 5-watch "$(fresh 5-watch show.cinq.toml)" "$W/empty"
shot before
mv "$W/run-5-watch/sounds/tone.wav" "$W/run-5-watch/sounds/tone.wav.away"
sleep 4
shot deleted
mv "$W/run-5-watch/sounds/tone.wav.away" "$W/run-5-watch/sounds/tone.wav"
sleep 4
shot restored
quit

# 6 Paste
launch 6-paste "$(fresh 6-paste show.cinq.toml)" "$W/empty"
typ gg
typ yy
typ p
sleep 1
shot at-once
sleep 5
shot after-5s
quit

# 7 Errors in words
mkdir -p "$W/adir"
launch 7-directory "$W/adir" "$W/empty"
shot start
quit
mkdir -p "$W/ro/sounds"
cp "$W/show.cinq.toml" "$W/ro/"
cp "$W/sounds/tone.wav" "$W/ro/sounds/"
chmod 555 "$W/ro"
launch 7-save "$W/ro/show.cinq.toml" "$W/empty"
typ o
shot edited
ctrl s
sleep 1
shot saved
quit
chmod 755 "$W/ro"

# 8 A network cue to a closed port, fired by F9
launch 8-network "$(fresh 8-network net.cinq.toml)" "$(conf net '[keys]
go = ["Space", "F9"]')"
code 101
sleep 3
shot fired
quit

# Verdicts as greps, to read beside the pictures
{
  for pair in 1-silent/start:Space 1-silent/start:Escape 1-silent/start:help 1-silent/start:hint \
    2-cua/start:cua 4-osc/after-99:99 4-osc-taken/start:listen 5-watch/deleted:BROKEN 5-watch/deleted:"not found" \
    6-paste/after-5s:loading 7-directory/start:"Cannot open" 7-save/saved:"Cannot save" 8-network/fired:"SEND FAILED" 8-network/fired:"not sent"; do
    printf '%s: ' "$pair"
    grep -c -i -F -- "${pair#*:}" "$OUT/${pair%%:*}.txt"
  done
  for log in "$OUT"/*/cinq.log; do
    printf '%s: %s bytes\n' "$log" "$(wc -c < "$log")"
  done
} > "$OUT/verdict.txt" 2>&1
cat "$OUT/verdict.txt"
