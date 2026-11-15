#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Drives cinq's vi and cua keys on Darwin with CGEvent keys, as the keyboard
# posts them, and the other mouse buttons 3 and 4 with CGEvent.  cinq's
# standard error goes to OUT/vi.err and OUT/cua.err, and each step writes the
# time and its name to vi.marks or cua.marks, so that verdict.sh can put the
# lines of cinq between the marks of the keys that caused them.
#
# Usage: darwin.sh APP OUT, with APP the path of cinq.app
set -uo pipefail

here=$(cd "$(dirname "$0")" && pwd)
app=$1
OUT=$(mkdir -p "$2" && cd "$2" && pwd)
work=$(mktemp -d)

now() { perl -MTime::HiRes=time -e 'printf "%.3f", time'; }
marks=
mark() { printf '%s MARK %s\n' "$(now)" "$1" >> "$marks"; }
shot() { screencapture -x "$OUT/$1.png" || true; }
# Brings cinq to the front, which System Events sometimes cannot find for a
# moment (-1728)
focus() {
  for _ in 1 2 3 4 5; do
    osascript -e "tell application \"System Events\" to set frontmost of (first process whose unix id is $pid) to true" && return
    sleep 1
  done
  return 0
}
# wait_log PATTERN COUNT: waits until cinq has logged COUNT lines that match
# PATTERN, up to 40 seconds
wait_log() {
  for _ in $(seq 80); do
    [ "$(grep -c -- "$1" "$OUT/$phase.err")" -ge "$2" ] && return
    sleep 0.5
  done
  mark "timeout: fewer than $2 lines of $1"
}
# key LABEL SPEC: marks the step, then posts the key with CGEvent
key() {
  local label=$1 spec
  shift
  focus
  sleep 0.4
  mark "$label"
  for spec in "$@"; do
    "$work/inject" key "$spec" >> "$OUT/inject.txt" 2>&1
    sleep 0.3
  done
  presses=$((presses + $#))
  wait_log ' press Press ' "$presses"
  sleep 1
}
# button LABEL NUMBER: moves the pointer to the middle of the window of cinq, which
# start put at 60, 60 with the size 900 by 640,
# then presses the other button NUMBER
button() {
  focus
  local x=510 y=380
  mark "$1"
  "$work/inject" button "$2" "$x" "$y" >> "$OUT/inject.txt" 2>&1
  buttons=$((buttons + 1))
  wait_log ' mouse button pressed ' "$buttons"
  sleep 1
}

swiftc -O -o "$work/inject" "$here/inject.swift" 2>&1 | tail -n 20
{ sw_vers; uname -m; osascript -e 'tell application "Finder" to get bounds of window of desktop'; } > "$OUT/system.txt" 2>&1 || true

start() {
  phase=$1
  presses=0
  buttons=0
  marks=$OUT/$1.marks
  : > "$marks"
  mkdir -p "$work/$1"
  cp "$here/$1.toml" "$work/$1/config.toml"
  open -n --stdout "$OUT/$1.out" --stderr "$OUT/$1.err" "$app" --args --config-dir "$work/$1" "$here/show.cinq.toml"
  sleep 8
  pid=$(pgrep -n -x cinq)
  # Window of 900 by 640, so that the list shows fewer rows than the window
  # could
  osascript -e "tell application \"System Events\" to tell (first process whose unix id is $pid) to set position of window 1 to {60, 60}" || true
  osascript -e "tell application \"System Events\" to tell (first process whose unix id is $pid) to set size of window 1 to {900, 640}" || true
  focus
  sleep 3
  shot "$1-0-start"
}
stop() {
  kill "$pid" || true
  sleep 2
}

start vi
# 1. Control+i after Control+o
key 1-l l
key 1-G shift+g
key 1-ctrl-o ctrl+o
key 1-ctrl-i ctrl+i
shot vi-1-back-forward
# 3. Half pages
key 3-gg g g
key 3-ctrl-d ctrl+d
shot vi-3-ctrl-d
key 3-ctrl-u ctrl+u
# 4. Copy, paste, cut, undo
key 4-gg g g
key 4-yy y y
key 4-p p; key 4-p p; key 4-p p
shot vi-4-pasted
key 4-dd d d
key 4-u u
key 4-3u 3 u
shot vi-4-undone
# 5. Control+o opens nothing, Control+l opens the file dialog
key 5-ctrl-o ctrl+o
shot vi-5-after-ctrl-o
key 5-ctrl-l ctrl+l
sleep 3
shot vi-5-dialog
osascript "$here/windows.applescript" > "$OUT/vi-5-windows.txt" 2>&1 || true
mark 5-cancel
osascript "$here/cancel.applescript" > "$OUT/vi-5-cancel.txt" 2>&1 || true
sleep 2
shot vi-5-after-cancel
osascript "$here/windows.applescript" > "$OUT/vi-5-windows-after.txt" 2>&1 || true
# 6. Other buttons 3 and 4 of the mouse
key 6-gg g g
key 6-l l
key 6-G shift+g
button 6-button3-back 3
button 6-button4-forward 4
shot vi-6-buttons
stop

start cua
# 2. Option with the arrows, posted with CGEvent and then through System Events
key 2-ctrl-pagedown ctrl+pagedown
key 2-end end
key 2-alt-left alt+left
key 2-alt-right alt+right
shot cua-2-alt-arrows
focus
mark 2-system-events-alt-left
osascript -e 'tell application "System Events" to key code 123 using {option down}' >> "$OUT/inject.txt" 2>&1
sleep 1
mark 2-system-events-alt-right
osascript -e 'tell application "System Events" to key code 124 using {option down}' >> "$OUT/inject.txt" 2>&1
sleep 1
shot cua-2-system-events
stop
