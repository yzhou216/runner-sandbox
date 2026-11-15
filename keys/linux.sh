#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Drives cinq's vi and cua keys in a headless sway session with wtype, and
# the back and forward buttons of the mouse with vptr, which makes a
# virtual pointer.  cinq's standard error goes to OUT/vi.err and OUT/cua.err,
# and each step writes the time and its name to vi.marks or cua.marks, so that
# verdict.sh can put the lines of cinq between the marks of the keys that
# caused them.
#
# Usage: linux.sh FONTS OUT, with cinq, sway, wtype, vptr (from vptr.sh), and grim on PATH
set -uo pipefail

# The file dialog comes from a portal, which reaches cinq over the session bus
if [ -z "${KEYS_BUS:-}" ]; then
  KEYS_BUS=1 exec dbus-run-session -- bash "$0" "$@"
fi

here=$(dirname "$(readlink -f "$0")")
fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)

export HOME=$work/home XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf '<?xml version="1.0"?><fontconfig><dir>%s</dir><cachedir>%s/fc</cachedir></fontconfig>\n' "$fonts" "$work" > "$FONTCONFIG_FILE"
cat > "$work/sway.config" <<'EOF'
output HEADLESS-1 resolution 1280x800
for_window [app_id="cinq"] floating enable, resize set 900 640, move position center
EOF
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"

# GTK's file chooser portal, where PORTAL_SHARE names the share directories of
# xdg-desktop-portal and xdg-desktop-portal-gtk
export XDG_DATA_DIRS=${PORTAL_SHARE:-}:${XDG_DATA_DIRS:-/usr/share} XDG_CURRENT_DESKTOP=sway XDG_CONFIG_HOME=$HOME/.config
mkdir -p "$XDG_CONFIG_HOME/xdg-desktop-portal"
printf '[preferred]\ndefault=gtk\n' > "$XDG_CONFIG_HOME/xdg-desktop-portal/portals.conf"

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
{ swaymsg -t get_outputs; swaymsg -t get_inputs; } > "$out/sway-devices.json" 2>&1
dbus-update-activation-environment WAYLAND_DISPLAY XDG_CURRENT_DESKTOP XDG_DATA_DIRS XDG_CONFIG_HOME XDG_RUNTIME_DIR FONTCONFIG_FILE HOME

err=
marks=
mark() { printf '%s MARK %s\n' "$(date +%s.%N)" "$1" >> "$marks"; }
shot() { grim "$out/$1.png" || true; swaymsg -t get_tree > "$out/$1.json" 2>&1 || true; }
focus() { swaymsg -q '[app_id="cinq"] focus'; sleep 0.5; }
# key LABEL WTYPE-ARGUMENT...: marks the step, then types
key() {
  local label=$1
  shift
  mark "$label"
  focus
  wtype -s 300 -d 100 "$@"
  sleep 1
}
ctrl() { key "$1" -M ctrl "$2" -m ctrl; }
alt() { key "$1" -M alt -k "$2" -m alt; }
button() {
  mark "$1"
  vptr 640 400 1280 800 "$2" >> "$out/vptr.txt" 2>&1
  printf 'vptr %s exit %s\n' "$2" "$?" >> "$out/vptr.txt"
  sleep 1
}

start() {
  err=$out/$1.err
  marks=$out/$1.marks
  : > "$err"
  mkdir -p "$work/$1"
  cp "$here/$1.toml" "$work/$1/config.toml"
  cinq --config-dir "$work/$1" "$here/show.cinq.toml" 2>> "$err" &
  cinq=$!
  sleep 8
  shot "$1-0-start"
}
stop() {
  swaymsg -q '[app_id="cinq"] kill' || true
  wait "$cinq" || true
}

start vi
# 1. Control+i after Control+o
key 1-l l
key 1-G G
ctrl 1-ctrl-o o
ctrl 1-ctrl-i i
shot vi-1-back-forward
# 3. Half pages
key 3-gg gg
ctrl 3-ctrl-d d
shot vi-3-ctrl-d
ctrl 3-ctrl-u u
# 4. Copy, paste, cut, undo
key 4-gg gg
key 4-yy yy
key 4-p p
key 4-p p
key 4-p p
shot vi-4-pasted
key 4-dd dd
key 4-u u
key 4-3u 3u
shot vi-4-undone
# 5. Control+o opens nothing, Control+l opens the file dialog
ctrl 5-ctrl-o o
shot vi-5-after-ctrl-o
ctrl 5-ctrl-l l
sleep 5
shot vi-5-after-ctrl-l
swaymsg -t get_tree | grep -E '"(app_id|name)"' > "$out/vi-5-windows.txt" || true
# The platform's cancel: closing the dialog's window, not a key to cinq
mark 5-cancel
swaymsg -q '[app_id="xdg-desktop-portal-gtk"] kill' || true
sleep 2
shot vi-5-after-cancel
swaymsg -t get_tree | grep -E '"(app_id|name)"' > "$out/vi-5-windows-after.txt" || true
# 6. Buttons of the mouse
key 6-gg gg
key 6-l l
key 6-G G
button 6-back back
button 6-forward forward
button 6-side side
button 6-extra extra
shot vi-6-buttons
stop

start cua
# 2. Alt with the arrows
key 2-ctrl-pagedown -M ctrl -k Next -m ctrl
key 2-end -k End
alt 2-alt-left Left
alt 2-alt-right Right
shot cua-2-alt-arrows
stop

swaymsg exit > /dev/null 2>&1 || true
