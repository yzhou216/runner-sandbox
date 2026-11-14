#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs linux.toml in cinq in a headless sway session of its own and records
# what goes out: sACN and Art-Net of the light cue, which listen.py hears
# on the runner's own networks, and the messages of the network cues, which
# fakeeos.py decodes as a console would.
#
# Usage: linux.sh FONTS OUT, with cinq, sway, wtype, and python3 on PATH,
# and FONTS the directory of the DejaVu fonts
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
ip -4 addr > "$out/ip.txt"

python3 "$here/listen.py" "$out/dmx.jsonl" 60 "$out/wire.pcap" &
listener=$!
python3 "$here/fakeeos.py" "$out/console.txt" 60 &
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

cinq "$here/linux.toml" > "$out/cinq.log" 2>&1 &
sleep 6
for cue in 1 2 3; do
  swaymsg -q '[app_id="cinq"] focus'
  wtype -s 300 ' '
  printf '%s GO %s\n' "$(date +%s.%N)" "$cue" >> "$out/marks.txt"
  sleep 5
done
# The console too, which holds its ports until it ends
wait "$listener" "$console"
swaymsg exit > /dev/null 2>&1 || true
