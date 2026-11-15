#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays lights.toml in cinq in headless sway with one GO, while listen.py
# records the sACN and Art-Net that reach the machine and fakeeos.py stands
# in for a console, which goes down for a while and comes back.  Runs on
# GNU/Linux and FreeBSD.
#
# Usage: play.sh CINQ SHOW OUT, with sway, wtype, python3, and (if there)
# grim on PATH
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
cinq=$1
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && cd "$3" && pwd)
work=$(mktemp -d)
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }
mark() { printf '%s %s\n' "$(now)" "$*" >> "$out/marks.txt"; }

export HOME=$work/home XDG_RUNTIME_DIR=$work/run
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'output HEADLESS-1 resolution 1600x1000\n' > "$work/sway.config"
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"
{ uname -a; ip -4 addr 2> /dev/null || ifconfig -a; } > "$out/ip.txt"
mark start

python3 "$here/listen.py" "$out/dmx.jsonl" 600 "$out/wire.pcap" &
listener=$!
python3 "$here/fakeeos.py" "$out/console-a.txt" 600 &
console=$!
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
trap 'kill "$listener" "$console" "$sway" 2> /dev/null || true' EXIT
socket='' ipc=''
for _ in $(seq 300); do
  for path in "$XDG_RUNTIME_DIR"/wayland-[0-9]; do [ -S "$path" ] && socket=${path##*/}; done
  for path in "$XDG_RUNTIME_DIR"/sway-ipc.*.sock; do [ -S "$path" ] && ipc=$path; done
  [ -n "$socket" ] && [ -n "$ipc" ] && break
  sleep 0.2
done
export WAYLAND_DISPLAY=$socket SWAYSOCK=$ipc
until swaymsg -t get_version > /dev/null 2>&1; do sleep 0.5; done
mark sway up

"$cinq" "$show" > >(python3 "$here/stamp.py" > "$out/cinq.log") 2>&1 &
cinq_pid=$!
for _ in $(seq 600); do
  swaymsg -t get_tree | grep -q '"app_id": "cinq"' && break
  kill -0 "$cinq_pid" 2> /dev/null || { mark "cinq ended before its window"; break; }
  sleep 0.5
done
kill -0 "$cinq_pid" 2> /dev/null || exit 1
mark window up
sleep 5
swaymsg -q '[app_id="cinq"] focus'
mark GO
wtype -s 300 ' '
sleep 30.3
kill "$console"
mark console down
sleep 2.7
python3 "$here/fakeeos.py" "$out/console-b.txt" 600 &
console_b=$!
mark console back
sleep 5
if command -v grim > /dev/null; then grim "$out/screen.png" || true; fi
sleep 8
mark end
swaymsg -t get_tree > "$out/tree.json" || true
swaymsg exit > /dev/null 2>&1 || true
sleep 2
kill "$listener" "$console_b" "$sway" 2> /dev/null || true
wait 2> /dev/null || true
