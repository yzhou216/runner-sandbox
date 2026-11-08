#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Types into cinq's window once it is up: adds a cue, saves the show as
# "show" through the save dialog, then opens the open dialog and closes it.
#
#   keys.sh <COM1 log> <QEMU monitor socket>
set -euo pipefail

log=$1
monitor=$2
send() {
  echo "sendkey $1" | socat - UNIX-CONNECT:"$monitor" > /dev/null
  echo "$(date +%T) sent $1" >&2
  sleep "${2:-1}"
}
until grep -aq 'ROSTEST CINQ start' "$log" 2>/dev/null; do sleep 1; done
sleep 30
send o 5
send ctrl-s 10
for key in s h o w; do send $key; done
send ret 8
send ctrl-o 10
send esc 5
