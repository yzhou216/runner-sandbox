#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Checks which files an open panel lets the operator choose.  cinq's open
# dialog hands rfd the extension "cinq.toml", which rfd gives to NSOpenPanel
# as an allowed file type, so that show.cinq.toml may be greyed out.
#
# Usage:
#   dialog.sh cinq CINQ OUT
#     opens cinq's own dialog with Control+o in a directory that holds
#     show.cinq.toml, other.toml, and note.txt, and tries to choose each
#   dialog.sh panel PANEL OUT TYPE...
#     opens a bare NSOpenPanel whose allowed file types are the TYPEs, which
#     panel.swift builds, in the same directory, and tries to choose the show
set -uo pipefail

mode=$1
program=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
OUT=$(mkdir -p "$3" && cd "$3" && pwd)
[ -x "$program" ] || { echo "no program at $program"; exit 1; }
shift 3
source "$(dirname "$0")/common.sh"

work=$(mktemp -d)
pick="$work/pick"
mkdir -p "$pick" "$work/config"
printf 'keymap = "vi"\n' > "$work/config/config.toml"
printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "memo"\nname = "from show"\n' > "$pick/show.cinq.toml"
printf '[[list]]\nname = "Other"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "memo"\nname = "from other"\n' > "$pick/other.toml"
printf 'not a show\n' > "$pick/note.txt"

: > "$OUT/steps.txt"
step "$(sw_vers -productVersion), $(uname -m), mode $mode, types: $*"

# Types a name into the list of the open panel, which selects the file whose
# name starts with it if the panel lets that file be chosen, and records the
# panel
choose() {
  sleep 2
  events "keystroke \"$1\""
  sleep 1
  shot "$1-typed"
  dump "$proc" "$1-typed"
}

case $mode in
  cinq)
    proc=cinq
    "$program" --config-dir "$work/config" "$pick/show.cinq.toml" 2> "$OUT/err.txt" &
    pid=$!
    wait_windows cinq 1 90 || { cat "$OUT/err.txt"; exit 1; }
    sleep 3
    windows=2
    first=true
    for name in other note show; do
      events 'set frontmost of process "cinq" to true'
      sleep 1
      step "Control+o opens the dialog, then $name is typed"
      events 'keystroke "o" using control down'
      wait_windows cinq 2 30
      sleep 3
      if $first; then
        shot open
        dump cinq open
        first=false
      fi
      # List view, whose rows show a greyed out name plainly
      events 'keystroke "2" using command down'
      choose "$name"
      step "Return after $name"
      events 'key code 36'
      sleep 3
      shot "$name-returned"
      if [ "$(osascript -e 'tell application "System Events" to count windows of process "cinq"' 2> /dev/null)" -ge "$windows" ] 2> /dev/null; then
        step "the dialog is still open after $name, Escape closes it"
        events 'key code 53'
        sleep 2
      fi
    done
    step "end"
    kill "$pid" 2> /dev/null
    wait "$pid" 2> /dev/null
    ;;
  panel)
    proc=panel
    "$program" "$pick" "$@" > "$OUT/panel-result.txt" 2> "$OUT/err.txt" &
    pid=$!
    wait_windows panel 1 60 || { cat "$OUT/err.txt"; exit 1; }
    sleep 3
    shot open
    dump panel open
    events 'keystroke "2" using command down'
    choose show
    step "Return after show"
    events 'key code 36'
    sleep 3
    shot show-returned
    if kill -0 "$pid" 2> /dev/null; then
      step "the panel is still open after show, Escape closes it"
      events 'key code 53'
      sleep 2
    fi
    kill "$pid" 2> /dev/null
    wait "$pid" 2> /dev/null
    step "panel result: $(cat "$OUT/panel-result.txt")"
    ;;
esac

echo "--- err.txt"
cat "$OUT/err.txt"
echo "--- steps.txt"
cat "$OUT/steps.txt"
