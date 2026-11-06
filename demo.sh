#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records how cinq reports the problems of its config.toml, in a headless
# sway session of its own: vim edits the file in one terminal, cinq runs
# from another, which shows what cinq writes to standard error, and cinq's
# window shows the warnings in its sidebar and the keys in effect in its
# status bar.  The first video edits the file while cinq runs; the second
# pauses the terminal with Control+s while saves still reach the window.
#
# Usage: demo.sh FONTS OUT, with cinq, sway, foot, vim, wtype, wf-recorder,
# and ffmpeg on PATH, FONTS the directory of the DejaVu fonts, and OUT the
# directory for the videos.  The file goes in $HOME/.config/cinq.
set -euo pipefail

fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)
config=$HOME/.config/cinq/config.toml

# Session: one 1920x1080 output, the DejaVu fonts alone, and cinq's window
# moved to the right of the terminals as it opens
export XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS XDG_CONFIG_HOME
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
for_window [app_id="cinq"] move right, resize set width 56 ppt
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

# Configuration with two mistakes, and no sound device to warn about
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

# Keys for the window `app_id`, typed one by one so that they show
keys() {
  swaymsg -q "[app_id=\"$1\"] focus"
  shift
  wtype -s 300 -d 90 "$@"
  sleep 0.5
}
# Captions of the video being recorded, each from the time it is set
caption() {
  printf '%s\t%s\n' "$(awk -v now="$(date +%s.%N)" -v then="$started" 'BEGIN { printf "%.2f", now - then }')" "$1" >> "$captions"
}
record() {
  captions=$out/$1.captions
  : > "$captions"
  wf-recorder -D -f "$work/$1.mp4" > "$out/$1.recorder.log" 2>&1 &
  recorder=$!
  started=$(date +%s.%N)
  sleep 1
}
stop() {
  sleep 3
  kill -INT "$recorder"
  wait "$recorder" || true
  caption ''
  burn "$1"
}
# Burns the captions into a band under the picture
burn() {
  local filters=() at text next
  mapfile -t lines < "$out/$1.captions"
  for ((i = 0; i + 1 < ${#lines[@]}; i++)); do
    IFS=$'\t' read -r at text <<< "${lines[i]}"
    IFS=$'\t' read -r next _ <<< "${lines[i + 1]}"
    printf '%s' "$text" > "$work/$1-$i.txt"
    filters+=("drawtext=fontfile=$fonts/truetype/DejaVuSans.ttf:textfile=$work/$1-$i.txt:expansion=none:fontsize=30:fontcolor=white:x=(w-text_w)/2:y=1080+(80-text_h)/2:enable='between(t,$at,$next)'")
  done
  local IFS=,
  ffmpeg -loglevel error -y -i "$work/$1.mp4" \
    -vf "pad=iw:ih+80:0:0:black,${filters[*]}" \
    -c:v libx264 -crf 22 -pix_fmt yuv420p "$out/$1.mp4"
}

record 1-editing
caption 'config.toml has two mistakes in it, and cinq is about to start from the terminal'
sleep 4
keys terminal 'cinq' -k Return
caption 'cinq draws each problem on standard error with miette, and lists it in its sidebar with its line'
sleep 8
caption 'Fixing the action on line 7 and saving: cinq reads the file again, binds help to f, and reports what is left'
keys editor '7Gcwhelp' -k Escape ':w' -k Return
sleep 7
caption 'Fixing the key on line 10: nothing is left to report'
keys editor '10G' 'fFcwPageDown' -k Escape ':w' -k Return
sleep 7
caption 'Deleting a closing bracket: the file is no longer TOML, so cinq falls back to the vi keys (?: help)'
keys editor '10G$x:w' -k Return
sleep 8
caption 'Undoing the deletion: the keys of the file apply again (f: help)'
keys editor 'u:w' -k Return
sleep 5
stop 1-editing

record 2-paused
caption 'Control+s pauses the output of the terminal that cinq runs in'
sleep 2
keys terminal -M ctrl s -m ctrl
sleep 3
caption 'A save with a mistake reaches the window at once, while its report waits for the terminal'
keys editor '7Gcwhepl' -k Escape ':w' -k Return
sleep 6
caption 'Fixing it reaches the window too, while the terminal stays paused'
keys editor 'u:w' -k Return
sleep 5
caption 'Control+q resumes the terminal, which shows the report that waited'
keys terminal -M ctrl q -m ctrl
sleep 6
stop 2-paused

swaymsg -q exit || true
