#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records demo.mov of how cinq, typed into Terminal on Darwin, shows the
# problems of its config.toml on a screen of their own while any are left,
# and switches back to what Terminal showed before once none are.  The
# script rewrites the file at each step, narrows Terminal's window while
# problems show, and ends cinq once by its window and once by Control+c,
# each time with a problem left.  Writes the time of each step since the
# recording started to captions.tsv.
#
# Usage: demo-darwin.sh OUT, with cinq on the PATH of Terminal's shell.
set -uo pipefail

out=$(mkdir -p "$1" && cd "$1" && pwd)
config="$HOME/Library/Application Support/cinq/config.toml"
mkdir -p "$(dirname "$config")"
broken=$(cat << 'EOF'
keymap = "booth"

[keymaps.booth]
from = "cua"
go = ["Space", "F5"]
fly = "f"

[keys]
next = ["Foo", "ArrowDown"]
EOF
)
one=${broken/fly/help}
fixed=${one/Foo/PageDown}
write() { printf '%s\n' "$1" > "$config"; }
write "$broken"

now() { perl -MTime::HiRes=time -e 'printf "%.2f", time'; }
caption() { printf '%s\t%s\n' "$(perl -e "printf '%.2f', $(now) - $started")" "$1" >> "$out/captions.tsv"; }
# System Events can lose a window for a moment, so a failure only logs
events() { osascript -e "tell application \"System Events\" to $1" || echo "System Events failed: $1"; }
place() {
  events "tell process \"$1\" to set position of window 1 to {$2, 25}"
  events "tell process \"$1\" to set size of window 1 to {$3, 720}"
}
# Line typed into Terminal, brought to the front first
type_line() {
  open -a Terminal
  sleep 1
  events "keystroke \"$1\""
  events 'key code 36'
}
start_cinq() {
  type_line 'cinq'
  for _ in $(seq 60); do
    [ "$(osascript -e 'tell application "System Events" to count windows of process "cinq"' 2> /dev/null)" -ge 1 ] 2> /dev/null && break
    sleep 1
  done
  sleep 1
  place cinq 585 439
}
close_cinq() {
  events 'tell process "cinq" to click (first button of window 1 whose subrole is "AXCloseButton")'
  for _ in $(seq 20); do
    pgrep -x cinq > /dev/null || return 0
    sleep 0.5
  done
  echo "cinq still runs"
}

system_profiler SPDisplaysDataType | grep -i resolution
open -a Terminal
sleep 6
place Terminal 0 580
type_line 'clear'
sleep 1
: > "$out/captions.tsv"
screencapture -x -V 150 "$out/demo.mov" &
recorder=$!
started=$(now)
sleep 2

caption 'config.toml has two mistakes; Terminal shows the file, then starts cinq'
type_line 'cat ~/Library/Application*/cinq/config.toml'
sleep 2
start_cinq
caption 'The problems take a screen of their own, in place of what Terminal showed'
sleep 8
caption 'Fixing the action: the problem left takes the place of both'
write "$one"
sleep 7
caption 'Fixing the key: none are left, so Terminal shows again what it showed before'
write "$fixed"
sleep 7
caption 'Deleting a closing bracket: the file is no longer TOML, and the problems screen is back'
write "${fixed%]}"
sleep 6
caption 'Narrowing Terminal: the problems are drawn again at the new width'
place Terminal 0 360
sleep 6
caption 'Putting the bracket back: Terminal shows again what it showed before'
write "$fixed"
sleep 6
caption 'A mistake again, then closing cinq: the problem stays in sight below the command'
write "${fixed/help/fly}"
sleep 5
close_cinq
sleep 5
caption 'Starting cinq again, with the mistake still in the file'
start_cinq
sleep 7
caption 'Control+c in Terminal: cinq ends, and the problem stays in sight'
open -a Terminal
sleep 1
events 'keystroke "c" using control down'
sleep 1
pgrep -x cinq > /dev/null && echo "cinq still runs after Control+c"
sleep 5
caption 'end'
wait "$recorder"
