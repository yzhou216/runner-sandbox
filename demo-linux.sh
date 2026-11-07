#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records how cinq shows the problems of its config.toml on a screen of
# their own, in a headless sway session of its own: vim edits the file in
# one terminal, cinq runs from another, and cinq's window shows the
# warnings in its sidebar.  The terminal switches to the problems while any
# are left and back to what it showed before once none are, and problems
# left when cinq ends, by its window or by Control+c, stay in sight.
#
# Usage: demo-linux.sh FONTS OUT, with cinq, sway, foot, vim, wtype, and
# wf-recorder on PATH, FONTS the directory of the DejaVu fonts, and OUT the
# directory for the video.  The file goes in $HOME/.config/cinq.
set -euo pipefail

fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)
config=$HOME/.config/cinq/config.toml

export XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS XDG_CONFIG_HOME RUST_LOG
mkdir -m 700 "$XDG_RUNTIME_DIR"
cat > "$FONTCONFIG_FILE" << EOF
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <dir>$fonts</dir>
  <cachedir>$work/fontconfig</cachedir>
  <alias><family>monospace</family><prefer><family>DejaVu Sans Mono</family></prefer></alias>
  <alias><family>sans-serif</family><prefer><family>DejaVu Sans</family></prefer></alias>
</fontconfig>
EOF
cat > "$work/sway.config" << 'EOF'
output HEADLESS-1 resolution 1920x1080
font pango:DejaVu Sans 11
default_border normal 2
gaps right 960
for_window [app_id="cinq"] floating enable, resize set 940 1040, move position 970 20
EOF
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK

mkdir -p "$(dirname "$config")"
cat > "$config" << 'EOF'
# Key bindings of cinq, which the manual page cinq(5) describes
keymap = "booth"

[keymaps.booth]
from = "cua"
go = ["Space", "F5"]
fly = "f"

[keys]
next = ["Foo", "ArrowDown"]
EOF
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"
printf 'set nocompatible number ruler laststatus=2 shortmess+=I\nsyntax on\n' > "$work/vimrc"

terminal=(foot -o 'font=DejaVu Sans Mono:size=11' -o 'pad=8x8')
"${terminal[@]}" --app-id=editor --title='vim config.toml' \
  --working-directory="$(dirname "$config")" vim -u "$work/vimrc" config.toml > "$out/editor.log" 2>&1 &
sleep 2
swaymsg -q splitv
"${terminal[@]}" --app-id=terminal --title='Terminal' --working-directory="$HOME" \
  env PS1='$ ' bash --norc --noprofile > "$out/terminal.log" 2>&1 &
sleep 2
swaymsg -q '[app_id="editor"] resize set height 30 ppt'

# Keys for the window `app_id`, typed one by one so that they show
keys() {
  swaymsg -q "[app_id=\"$1\"] focus"
  shift
  wtype -s 300 -d 90 "$@"
  sleep 0.5
}
caption() {
  printf '%s\t%s\n' "$(awk -v now="$(date +%s.%N)" -v then="$started" 'BEGIN { printf "%.2f", now - then }')" "$1" >> "$out/captions.tsv"
}

: > "$out/captions.tsv"
wf-recorder -D -f "$out/demo.mp4" > "$out/recorder.log" 2>&1 &
recorder=$!
started=$(date +%s.%N)
sleep 1
caption 'config.toml has two mistakes; the terminal shows the file, then starts cinq'
sleep 2
keys terminal 'cat ~/.config/cinq/config.toml' -k Return
sleep 2
keys terminal 'cinq' -k Return
caption 'The problems take a screen of their own, in place of what the terminal showed'
sleep 8
caption 'Fixing line 7: the problem left takes the place of both'
keys editor '7Gcwhelp' -k Escape ':w' -k Return
sleep 7
caption 'Fixing line 10: none are left, so the terminal shows again what it showed before'
keys editor '10G' 'fFcwPageDown' -k Escape ':w' -k Return
sleep 7
caption 'Deleting a closing bracket: the file is no longer TOML, and the problems screen is back'
keys editor '10G$x:w' -k Return
sleep 6
caption 'Narrowing the terminal: the problems are drawn again at the new width'
swaymsg -q gaps right all set 1400
sleep 6
caption 'Undoing the deletion: the terminal shows again what it showed before'
keys editor 'u:w' -k Return
sleep 6
caption 'A mistake again, then closing cinq: the problem stays in sight below the command'
keys editor '7Gcwfly' -k Escape ':w' -k Return
sleep 5
swaymsg -q '[app_id="cinq"] kill' || true
sleep 6
caption 'Starting cinq again, with the mistake still in the file'
keys terminal 'cinq' -k Return
sleep 7
caption 'Control+c in the terminal: cinq ends, and the problem stays in sight'
keys terminal -M ctrl c -m ctrl
sleep 6
caption ''
kill -INT "$recorder"
wait "$recorder" || true
swaymsg -q exit || true
