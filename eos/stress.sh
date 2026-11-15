#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Loads cinq's network and light output in a headless sway session of its
# own, with the show that stress.py writes: GO fires 50 network cues over
# TCP to fakeeos.py at once, then 50 over UDP; fakeeos.py goes down and comes
# back; GO fires 10 more over TCP, then a light cue on four universes, then
# 300 network cues a tenth of a second apart, while the memory of cinq is
# sampled.  listen.py records the sACN and Art-Net.
#
# Usage: stress.sh FONTS OUT, with cinq, sway, wtype, grim, and python3 on
# PATH, and FONTS the directory of the DejaVu fonts
set -euo pipefail

here=$(dirname "$(readlink -f "$0")")
fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)

export HOME=$work/home XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf '<?xml version="1.0"?><fontconfig><dir>%s</dir><cachedir>%s/fc</cachedir></fontconfig>\n' "$fonts" "$work" > "$FONTCONFIG_FILE"
printf 'output HEADLESS-1 resolution 1600x1000\n' > "$work/sway.config"
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"
python3 "$here/stress.py" "$work/stress.toml"

python3 "$here/listen.py" "$out/dmx.jsonl" 100 "$out/wire.pcap" &
listener=$!
python3 "$here/fakeeos.py" "$out/console-1.txt" 20 &
console=$!
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK
until swaymsg -t get_version > /dev/null 2>&1; do sleep 0.5; done

cinq "$work/stress.toml" > "$out/cinq.log" 2>&1 &
sleep 6
# Presses GO, for what LABEL says, and takes a screenshot after a while
go() {
  swaymsg -q '[app_id="cinq"] focus'
  wtype -s 300 ' '
  printf '%s GO %s\n' "$(date +%s.%N)" "$1" >> "$out/marks.txt"
  sleep 3
  grim "$out/$1.png"
}
go tcp
go udp
kill "$console"
wait "$console" || true
printf '%s console down\n' "$(date +%s.%N)" >> "$out/marks.txt"
python3 "$here/fakeeos.py" "$out/console-2.txt" 80 &
sleep 2
go again
go light
pid=$(pgrep -n -x cinq)
go soak
# Memory of cinq while the soak runs, in kilobytes
for _ in $(seq 10); do
  printf '%s %s\n' "$(date +%s.%N)" "$(ps -o rss= -p "$pid")" >> "$out/memory.txt"
  sleep 3
done
wait "$listener"
swaymsg exit > /dev/null 2>&1 || true
