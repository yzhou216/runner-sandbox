# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Helpers that checks.sh uses, sourced with OUT set.  ui, built from
# screen.swift, drives the screen by the text that it shows.

now() { perl -MTime::HiRes=time -e 'printf "%.3f", time'; }
mark() { printf '%s %s\n' "$(now)" "$1" >> "$OUT/marks.txt"; }
# shot NAME: a screenshot and the text that Vision reads from it
shot() {
  screencapture -x "$D/$1.png" || true
  ./ui list > "$D/$1.txt" 2>&1 || true
}
focus() {
  for _ in 1 2 3 4 5; do
    osascript -e "tell application \"System Events\" to set frontmost of (first process whose unix id is $PID) to true" && return
    sleep 1
  done
  return 0
}
# typ LETTERS: types them as System Events keystrokes
typ() { focus; osascript -e "tell application \"System Events\" to keystroke \"$1\"" || true; sleep 0.5; }
# code N: presses the key of virtual key code N (Escape 53, F9 101, Backspace 51)
code() { focus; osascript -e "tell application \"System Events\" to key code $1" || true; sleep 0.5; }
ctrl() { focus; osascript -e "tell application \"System Events\" to keystroke \"$1\" using control down" || true; sleep 0.5; }
# allow SECONDS LABEL: presses Allow on Darwin's Local Network prompt, as long as one comes up
allow() {
  for press in 1 2 3; do
    ./ui click Allow "$1" > "$D/allow-$2-$press.txt" 2>&1 || return 0
    mark "pressed Allow ($2, press $press)"
    sleep 1
  done
}
# launch NAME SHOW CONFIGDIR: opens cinq.app on SHOW, with its standard error in NAME/cinq.log
launch() {
  D=$OUT/$1
  [ -f "$3/config.toml" ] || { mkdir -p "$W/cfg-$1"; set -- "$1" "$2" "$W/cfg-$1"; }
  mkdir -p "$D"
  mark "launch $1"
  open -n --stdout "$D/cinq.log" --stderr "$D/cinq.log" "$APP" --args --config-dir "$3" "$2"
  sleep 8
  PID=$(pgrep -n -x cinq)
  allow 3 "$1"
  sleep 1
}
quit() {
  kill "$PID" || true
  sleep 2
}
