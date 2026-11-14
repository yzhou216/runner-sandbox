#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Cuts the power of a console under cinq: GO sends a network cue to
# fakeeos.py, then iptables drops every packet of port 3037, as a pulled
# cable or a lost power supply does, so that cinq never hears that
# fakeeos.py has gone; after 20 seconds the network comes back with a new
# fakeeos.py, and GO sends two more cues.  Whether the first of them arrives
# tells whether cinq found the old connection dead.
#
# Usage: powercut.sh CINQ FONTS OUT, with sway, wtype, python3, and sudo
# iptables at hand, and FONTS the directory of the DejaVu fonts
set -euo pipefail

here=$(dirname "$(readlink -f "$0")")
cinq=$(readlink -f "$1")
fonts=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
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

mark() { printf '%s %s\n' "$(date +%s.%N)" "$1" >> "$out/marks.txt"; }
cut() { sudo iptables "$1" OUTPUT -o lo -p tcp --dport 3037 -j DROP; sudo iptables "$1" OUTPUT -o lo -p tcp --sport 3037 -j DROP; }
# The network comes back however the script ends
trap 'cut -D 2> /dev/null || true' EXIT
# Until no earlier console holds the port
while ss -ltn | grep -q ':3037 '; do sleep 1; done

python3 "$here/fakeeos.py" "$out/console-1.txt" 15 &
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
"$cinq" "$here/powercut.toml" > "$out/cinq.log" 2>&1 &
sleep 6
go() {
  swaymsg -q '[app_id="cinq"] focus'
  wtype -s 300 ' '
  mark "GO $1"
  sleep 2
}

go before
cut -I
mark "cable out"
kill "$console" || true
wait "$console" || true
sleep 20
ss -tn > "$out/sockets.txt" 2>&1 || true
cut -D
mark "cable back"
python3 "$here/fakeeos.py" "$out/console-2.txt" 15 &
console=$!
sleep 2
go first
go second
sleep 3
kill "$console" || true
wait "$console" || true
swaymsg exit > /dev/null 2>&1 || true
