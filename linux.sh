#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records cinq holding what is printed while the problems of its config.toml
# show on the alternate screen of the terminal, in a headless sway session of
# its own.  The top terminal runs cinq and the bottom one takes commands:
# other processes write to the standard error of cinq through
# /proc/PID/fd/2 while the problems show, config.toml is fixed, and the held
# lines come out on the main screen.  Then a problem comes back, one more
# line is written, and cinq is closed, which brings out the held line and
# then the problem that is left.
#
# Usage: linux.sh FONTS OUT, with cinq, sway, foot, aplay, script, wtype,
# wf-recorder, ffmpeg, and burn.sh next to this script, FONTS the directory
# of the DejaVu fonts, and OUT the directory for the video.  The file goes
# in $HOME/.config/cinq.
set -euo pipefail

fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
here=$(dirname "$(readlink -f "$0")")
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
for_window [app_id="cinq"] move right, resize set width 38 ppt
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

# A setting that cinq does not know, and no sound device to warn about
mkdir -p "$(dirname "$config")"
printf 'colour = "red"\n' > "$config"
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"

# The top terminal keeps a typescript of what it receives, which shows when
# the terminal switched screens and what came out after it
terminal=(foot -o 'font=DejaVu Sans Mono:size=13' -o 'pad=8x8')
shell="env PS1='\$ ' bash --norc --noprofile"
"${terminal[@]}" --app-id=run --title='cinq' --working-directory="$(dirname "$config")" \
  script -qf "$out/typescript" -c "$shell" > "$out/run.log" 2>&1 &
sleep 2
swaymsg -q splitv
"${terminal[@]}" --app-id=commands --title='Commands' --working-directory="$(dirname "$config")" \
  env PS1='$ ' bash --norc --noprofile > "$out/commands.log" 2>&1 &
sleep 2
swaymsg -q '[app_id="commands"] resize set height 26 ppt'

# Types a line into the terminal of `app_id`, one key at a time so that it
# shows, and presses Enter
typed() {
  swaymsg -q "[app_id=\"$1\"] focus"
  wtype -s 300 -d 55 "$2" -k Return
  sleep 0.5
}
# Captions, each from the time it is set
caption() {
  printf '%s\t%s\n' "$(awk -v now="$(date +%s.%N)" -v then="$started" 'BEGIN { printf "%.2f", now - then }')" "$1" >> "$captions"
}

captions=$out/linux.captions
: > "$captions"
wf-recorder -D -f "$work/linux.mp4" > "$out/recorder.log" 2>&1 &
recorder=$!
started=$(date +%s.%N)
sleep 1

caption 'cinq runs in the top terminal, and commands go into the bottom one'
sleep 2
caption 'config.toml sets something that cinq does not know'
typed commands 'cat config.toml'
sleep 2
caption 'cinq starts, and the problem takes the alternate screen of the terminal'
typed run 'cinq'
sleep 5
caption 'Another process writes a line to the standard error of cinq'
typed commands 'echo "a line from another process" > /proc/$(pidof cinq)/fd/2'
sleep 3
caption 'So does aplay, a C program, with an ALSA error: the problems screen stays clean'
typed commands 'aplay -D missing /dev/zero 2> /proc/$(pidof cinq)/fd/2'
sleep 4
caption 'Fixing config.toml: the terminal switches back, and the held lines come out'
typed commands 'sed -i /colour/d config.toml'
sleep 6
caption 'A problem comes back, and one more line is written meanwhile'
typed commands 'echo '"'"'colour = "red"'"'"' > config.toml'
sleep 3
typed commands 'echo "one more line" > /proc/$(pidof cinq)/fd/2'
sleep 2
caption 'Closing cinq with the problem left: the held line comes out first, then the problem'
typed commands "swaymsg '[app_id=\"cinq\"] kill'"
sleep 6
caption ''
kill -INT "$recorder"
wait "$recorder" || true

bash "$here/burn.sh" "$fonts/truetype/DejaVuSans.ttf" "$work/linux.mp4" "$captions" "$out/linux.mp4"
swaymsg -q exit || true
