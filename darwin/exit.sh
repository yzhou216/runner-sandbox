#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 6: how long the process takes to end when the window is closed (Command-W),
# then quit (Command-Q), then by SIGTERM, with the 500-cue show and the lights demo.
#
# Usage: exit.sh APPS OUT
set -uo pipefail
APPS=$1
OUT=$2
# shellcheck source=common2.sh
. darwin/common2.sh

template500 "$W/t500"
templatelights "$W/tlights"
APP=$APPS/cinq-new.app
# keys NAME LETTER: Command and the letter, with the time from the key to the end of the process
cmd() {
  focus
  t=$(now)
  osascript -e "tell application \"System Events\" to keystroke \"$2\" using command down" || true
  sent=$(awk -v a="$t" -v b="$(now)" 'BEGIN { print b - a }')
  echo "$1: Command-$2 took $sent s to post, then the process ended after $(waitgone "$PID" 6) s" | tee -a "$OUT/exit.txt"
}
for show in 500 lights; do
  launch "6-$show-w" "$(fresh "$show-w" "t$show")" "$W/empty"
  [ "$show" = 500 ] && sleep 20
  shot before
  cmd "$show" w
  if kill -0 "$PID" 2> /dev/null; then
    shot after-w
    cmd "$show" q
  fi
  if kill -0 "$PID" 2> /dev/null; then
    kill -TERM "$PID"
    echo "$show: still alive, SIGTERM: $(waitgone "$PID" 6) s" | tee -a "$OUT/exit.txt"
  fi
  kill -KILL "$PID" 2> /dev/null || true
  launch "6-$show-q" "$(fresh "$show-q" "t$show")" "$W/empty"
  [ "$show" = 500 ] && sleep 20
  cmd "$show" q
  launch "6-$show-term" "$(fresh "$show-term" "t$show")" "$W/empty"
  [ "$show" = 500 ] && sleep 20
  kill -TERM "$PID"
  echo "$show: SIGTERM, the process ended after $(waitgone "$PID" 6) s" | tee -a "$OUT/exit.txt"
  kill -KILL "$PID" 2> /dev/null || true
done
