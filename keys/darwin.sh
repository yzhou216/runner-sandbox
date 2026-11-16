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
# key LABEL SPEC...: marks the step, posts the keys with CGEvent, then waits
# until cinq has logged one more line that matches $pattern for each key.  The
# keys that the keymap reads log a press, while those typed into the search
# line or the menu of a drop-down list log what the caller's pattern names.
key() {
  local label=$1 spec pat=${pattern:-' press Press '} before
  shift
  focus
  sleep 0.4
  before=$(grep -c -- "$pat" "$OUT/$phase.err")
  mark "$label"
  for spec in "$@"; do
    "$work/inject" key "$spec" >> "$OUT/inject.txt" 2>&1
    sleep 0.3
  done
  wait_log "$pat" $((before + $#))
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
swiftc -O -o "$work/ui" "$here/screen.swift" 2>&1 | tail -n 20
{ sw_vers; uname -m; osascript -e 'tell application "Finder" to get bounds of window of desktop'; } > "$OUT/system.txt" 2>&1 || true

# start PHASE KEYMAP SHOW: opens cinq with the keymap of KEYMAP.toml and a copy
# of SHOW.cinq.toml of its own, so that no phase finds the unsaved changes of
# another, logging to PHASE.err and PHASE.marks
start() {
  phase=$1
  buttons=0
  marks=$OUT/$1.marks
  : > "$marks"
  mkdir -p "$work/$1"
  cp "$here/$2.toml" "$work/$1/config.toml"
  cp "$here/$3.cinq.toml" "$work/$1/show.cinq.toml"
  open -n --stdout "$OUT/$1.out" --stderr "$OUT/$1.err" "$app" --args --config-dir "$work/$1" "$work/$1/show.cinq.toml"
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

start vi vi show
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
# The file dialog is a window of a service of its own, which ui lists with the
# text on the screen, and whose Cancel button ui clicks where Vision reads it
"$work/ui" windows > "$OUT/vi-5-windows.txt" 2>&1 || true
"$work/ui" list >> "$OUT/vi-5-windows.txt" 2>&1 || true
mark 5-cancel
"$work/ui" click --contains Cancel 10 > "$OUT/vi-5-cancel.txt" 2>&1 || true
sleep 2
shot vi-5-after-cancel
"$work/ui" windows > "$OUT/vi-5-windows-after.txt" 2>&1 || true
"$work/ui" list >> "$OUT/vi-5-windows-after.txt" 2>&1 || true
# 6. Other buttons 3 and 4 of the mouse
key 6-gg g g
key 6-l l
key 6-G shift+g
button 6-button3-back 3
button 6-button4-forward 4
shot vi-6-buttons
stop

start cua cua show
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

searching=' message Searching('
accepted=' message SearchAccepted'
line_key=' message SearchKey('
cancelled=' message Cancel'
menu_key=' menu key='
opened=' message MenuOpen(true'

# 7. Operators: d 2 j cuts three cues, p pastes them below
start ops vi long
key 7-gg g g
key 7-d2j d 2 j
shot ops-7-cut
key 7-p p
shot ops-7-pasted
# 11. A stray key after d is dropped: d u does not undo
key 11-du d u
shot ops-11-du
# 11. Control+c after 2 d drops the typed keys and cuts nothing
key 11-2d 2 d
key 11-ctrl-c ctrl+c
shot ops-11-ctrl-c
stop

# 8. Search, forward and backward, with n, N, and Control+o
start search vi long
key 8-gg g g
key 8-slash slash
pattern=$searching key 8-text 1
pattern=$accepted key 8-enter return
key 8-n1 n
key 8-n2 n
key 8-qmark shift+slash
pattern=$searching key 8-text-back 1
pattern=$accepted key 8-enter-back return
key 8-n3 n
key 8-N shift+n
key 8-ctrl-o ctrl+o
key 8-9 9
key 8-9-ctrl-o ctrl+o
shot search-8-back
# 8. A search that wraps tells so: n goes up after ?, which wraps at the top
# of the list, and N goes down, which wraps at the bottom
key 8-gg-wrap g g
key 8-n-wrap n
key 8-N-wrap shift+n
# F1 shows the help page, which q closes
key h-f1 f1
shot search-h-help
key h-q q
stop

# 12. The search line: ArrowUp recalls an earlier search, Control+u clears the
# line, and Control+c closes it and gives the selection back
start line vi names
key 12-gg g g
key 12-slash-1 slash
pattern=$searching key 12-bl shift+b l
pattern=$accepted key 12-enter-bl return
key 12-slash-2 slash
pattern=$searching key 12-br shift+b r
pattern=$accepted key 12-enter-br return
key 12-gg-2 g g
key 12-slash-3 slash
pattern=$searching key 12-b shift+b
pattern=$line_key key 12-up up
pattern=$searching key 12-i i
pattern=$line_key key 12-ctrl-u ctrl+u
pattern=$searching key 12-bo shift+b o
shot line-12-typed
pattern=$cancelled key 12-ctrl-c ctrl+c
shot line-12-closed
stop

# 3. Whole pages: three Control+f in a row move three pages
# 9. View keys, on a list of sixty cues
start view vi sixty
key 3-gg g g
key 3-ctrl-f ctrl+f ctrl+f ctrl+f
shot view-3-ctrl-f
key 9-gg g g
key 9-ctrl-e-1 ctrl+e
key 9-ctrl-e-2 ctrl+e
key 9-ctrl-e-3 ctrl+e
shot view-9-scrolled
key 9-ctrl-y ctrl+y
key 9-H shift+h
key 9-L shift+l
key 9-zz z z
shot view-9-zz
key 9-zt z t
shot view-9-zt
key 9-H-after-zt shift+h
# A far move brings the cue into view at the middle
key 9-55G 5 5 shift+g
shot view-9-55G
stop

# 13. Tab from the cue list reaches the first field of the inspector first,
# whose focus the log names; the toolbar's list of cue types comes after the
# fields, so that nine Tabs reach it
# 10. A drop-down list: Tab to it, Enter opens it, Control+y picks, Control+e closes
start menu vi show
key 13-tab-1 tab
shot menu-13-first-tab
key 13-tab-2 tab
for n in 3 4 5 6 7 8 9; do key "10-tab-$n" tab; done
shot menu-10-focused
pattern=$opened key 10-enter return
shot menu-10-open
pattern=$menu_key key 10-j j
pattern=$menu_key key 10-ctrl-y ctrl+y
shot menu-10-picked
pattern=$opened key 10-enter-again return
pattern=$menu_key key 10-ctrl-e ctrl+e
shot menu-10-closed
stop

# 14. Enter in the Name or Channels field of an output applies the text and
# leaves the field, so that the key after it moves to another output
typed=' message SettingTyped('
left=' message Focus(Cues)'
start outs vi outputs
key 14-l l
key 14-cc c c
pattern=$typed key 14-name shift+f r o n t
shot outs-14-typed
pattern=$left key 14-enter return
key 14-j j
shot outs-14-after-name
for n in 1 2 3; do key "14-tab-$n" tab; done
pattern=$typed key 14-channels 1 minus 2
pattern=$left key 14-enter-channels return
key 14-k k
shot outs-14-after-channels
stop

# 16. Enter in an empty File field opens the file chooser, which Cancel closes
start file vi outputs
key 16-j j
for n in 1 2 3; do key "16-tab-$n" tab; done
shot file-16-focused
pattern=' message ChooseFile(' key 16-enter return
sleep 3
shot file-16-dialog
"$work/ui" windows > "$OUT/file-16-windows.txt" 2>&1 || true
"$work/ui" list >> "$OUT/file-16-windows.txt" 2>&1 || true
mark 16-cancel
"$work/ui" click --contains Cancel 10 > "$OUT/file-16-cancel.txt" 2>&1 || true
sleep 2
shot file-16-after-cancel
"$work/ui" windows > "$OUT/file-16-windows-after.txt" 2>&1 || true
"$work/ui" list >> "$OUT/file-16-windows-after.txt" 2>&1 || true
stop
