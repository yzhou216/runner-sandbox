# shellcheck shell=bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Helpers that the Darwin tests share, sourced with OUT set.  ui, built from
# screen.swift, drives the screen by the text that it shows.

now() { perl -MTime::HiRes=time -e 'printf "%.3f", time'; }
# Times of what happens go to marks.txt in seconds since the epoch
mark() { printf '%s %s\n' "$(now)" "$1" >> "$OUT/marks.txt"; }
shot() { screencapture -x "$OUT/$1.png" || true; ./ui list > "$OUT/$1.txt" 2>&1 || true; }
# Brings the program of process PID to the front, which System Events
# sometimes cannot find for a moment (-1728)
focus() {
  for _ in 1 2 3 4 5; do
    osascript -e "tell application \"System Events\" to set frontmost of (first process whose unix id is $1) to true" && return
    sleep 1
  done
  return 0
}
# go PID LABEL [COUNT [GAP]]: presses GO COUNT times, GAP seconds apart,
# and marks the time of each press as GO LABEL, or GO LABEL-N
go() {
  local count=${3:-1} gap=${4:-0.2} n=0
  focus "$1"
  for stamp in $(osascript darwin/go.applescript "$count" "$gap" || true); do
    n=$((n + 1))
    if [ "$count" = 1 ]; then
      printf '%s GO %s\n' "$stamp" "$2" >> "$OUT/marks.txt"
    else
      printf '%s GO %s-%s\n' "$stamp" "$2" "$n" >> "$OUT/marks.txt"
    fi
  done
}
# allow SECONDS LABEL: presses Allow on Darwin's Local Network prompt if one
# comes up within SECONDS, as the exact text, since Don't Allow holds Allow
# too and comes first; returns whether it pressed
allow() {
  if ./ui click Allow "$1" > "$OUT/allow-$2.txt" 2>&1; then
    mark "pressed Allow ($2)"
    return 0
  fi
  mark "no Allow prompt ($2)"
  return 1
}
# stamped FILE: prepends the time of day to each line that FILE gains
stamped() {
  { tail -n 0 -F "$1" | perl -MTime::HiRes=time -ne 'BEGIN { $| = 1 } printf "%.3f %s", time, $_'; } > "$1.stamped" 2>&1 < /dev/null &
  # shellcheck disable=SC2034 # Read by the scripts that source this
  watcher=$!
}
system() {
  { sw_vers; uname -a; sysctl -n hw.model machdep.cpu.brand_string; python3 --version; ifconfig; netstat -rn -f inet; } > "$OUT/system.txt" 2>&1 || true
}
