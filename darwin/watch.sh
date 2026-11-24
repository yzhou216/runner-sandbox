#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 4: the file watch of NEW in the 500-cue show.  A burst of screenshots runs
# while a file goes, comes back, and another is replaced by a longer one; the text
# of each frame says when each change showed.
#
# Usage: watch.sh APPS OUT
set -uo pipefail
APPS=$1
OUT=$2
# shellcheck source=common2.sh
. darwin/common2.sh

template500 "$W/t500"
APP=$APPS/cinq-new.app
launch 4-watch "$(fresh watch t500)" "$W/empty"
sleep 20
shot loaded
S=$W/run-watch/sounds
burst "$D/frames" &
burster=$!
sleep 4
mv "$S/s001.wav" "$S/s001.wav.away"
t_delete=$(now)
sleep 10
mv "$S/s001.wav.away" "$S/s001.wav"
t_back=$(now)
sleep 10
cp "$S/long.wav" "$S/s002.new"
mv "$S/s002.new" "$S/s002.wav"
t_replace=$(now)
sleep 10
touch "$D/frames/stop"
wait "$burster"
shot end
quit
readframes "$D/frames"
{
  echo "delete at $t_delete"
  python3 darwin/first.py "$D/frames" 'broken|not found|file not' present "$t_delete"
  echo "restore at $t_back"
  python3 darwin/first.py "$D/frames" 'broken|not found|file not' absent "$t_back"
  echo "replace at $t_replace"
  python3 darwin/first.py "$D/frames" '0:05' present "$t_replace"
} | tee "$OUT/watch.txt"
