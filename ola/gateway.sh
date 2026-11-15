#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs gateway.py, which plays gateway.toml in cinq into OLA's sACN and
# Art-Net inputs, in a headless sway session of its own, or another script
# with its show, such as refuse.py with refuse.toml.
#
# Usage: gateway.sh FONTS OUT [SCRIPT SHOW], with cinq, sway, wtype, grim,
# python3, and OLA's tools on PATH, and FONTS the directory of the DejaVu
# fonts
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
ip -4 route >> "$out/ip.txt"

sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK
until swaymsg -t get_version > /dev/null 2>&1; do sleep 0.5; done

status=0
python3 "$here/${3:-gateway.py}" "$out" "$here/${4:-gateway.toml}" "$work" || status=$?
swaymsg exit > /dev/null 2>&1 || true
wait "$sway" || true
exit "$status"
