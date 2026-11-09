#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Helpers of keys.sh and dialog.sh, which source this file with OUT set to the
# directory that collects their screenshots and logs

now() { perl -MTime::HiRes=time -e 'printf "%.3f", time'; }

# Line in steps.txt with the time that the logs of cinq stamp, since both
# are seconds since the epoch
step() { printf '[%s] STEP %s\n' "$(now)" "$1" | tee -a "$OUT/steps.txt"; }

# System Events can lose a window for a moment, so a failure only logs
events() {
  osascript -e "tell application \"System Events\" to $1" 2>&1 || echo "System Events failed: $1"
}

shot() { screencapture -x "$OUT/$1.png" && step "screenshot $1"; }

# Waits until process $1 has at least $2 windows, up to $3 seconds
wait_windows() {
  for _ in $(seq "$(($3 * 2))"); do
    count=$(osascript -e "tell application \"System Events\" to count windows of process \"$1\"" 2> /dev/null)
    [ "${count:-0}" -ge "$2" ] 2> /dev/null && return 0
    sleep 0.5
  done
  step "process $1 never had $2 windows"
  return 1
}

# Lists the windows of process $1 and their interface elements into $OUT/$2.txt
dump() {
  step "dump $2"
  timeout 180 osascript "$(dirname "${BASH_SOURCE[0]}")/dump.applescript" "$1" > "$OUT/$2.txt" 2>&1 || echo "dump failed: $?" >> "$OUT/$2.txt"
}
