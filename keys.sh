#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Sends Option+j and Option+k to cinq with its vi keymap, through System
# Events, and records what cinq's log says about each key event and how the
# order of the cues changes.  The vi keymap binds Alt+j to move_down and
# Alt+k to move_up, and on Darwin Option+j types another character, the
# increment sign, which may be what the event's modified key holds, so that
# the binding never matches.
#
# Usage: keys.sh CINQ OUT, with OUT the directory for the results.
set -uo pipefail

cinq=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
OUT=$(mkdir -p "$2" && cd "$2" && pwd)
source "$(dirname "$0")/common.sh"

work=$(mktemp -d)
mkdir -p "$work/config"
printf 'keymap = "vi"\n' > "$work/config/config.toml"
show="$work/show.cinq.toml"
{
  printf '[[list]]\nname = "Main"\n'
  id=0
  for name in alpha beta gamma delta; do
    id=$((id + 1))
    printf '\n[[list.cue]]\nid = %s\nnumber = "%s"\nkind = "memo"\nname = "%s"\n' "$id" "$id" "$name"
  done
} > "$show"

order() { grep -E '^name = ' "$show" | sed 's/name = //' | tr -d '"' | tr '\n' ' ' | sed 's/^Main //'; }

: > "$OUT/steps.txt"
step "cinq starts with the show, in the order $(order)"
"$cinq" --config-dir "$work/config" "$show" 2> "$OUT/err.txt" &
pid=$!
wait_windows cinq 1 90 || { cat "$OUT/err.txt"; exit 1; }
sleep 3
events 'set frontmost of process "cinq" to true'
sleep 1
shot start

# Presses Option with a key through System Events, and saves the show with
# Control+s, so that the file shows what the key did
press() {
  step "$1"
  events "$2"
  sleep 2
}
save() {
  events 'keystroke "s" using control down'
  sleep 2
  step "saved, the file's order is: $(order)"
}

press 'keystroke "j" using option down, first time' 'keystroke "j" using option down'
press 'keystroke "j" using option down, second time' 'keystroke "j" using option down'
shot after-option-j
save

press 'key code 40 (the k key) using option down' 'key code 40 using option down'
shot after-option-k
save

press 'keystroke "k" using option down' 'keystroke "k" using option down'
save

press 'control: keystroke "j" alone, which selects the next cue' 'keystroke "j"'
press 'control: key code 125 (Down arrow) using option down, Alt+ArrowDown' 'key code 125 using option down'
shot after-option-down
save

step "end"
kill "$pid" 2> /dev/null
wait "$pid" 2> /dev/null
cp "$show" "$OUT/show.cinq.toml"
echo "--- err.txt"
cat "$OUT/err.txt"
echo "--- steps.txt"
cat "$OUT/steps.txt"
