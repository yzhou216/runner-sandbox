#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Takes the screenshots of cinq's tutorial in a headless sway session of
# its own, with two 4:3 outputs: the main window fills the first, the
# stage window the second, and neither has a border, so that each shot
# holds one whole window and nothing of the platform around it.  Sound goes
# to a PulseAudio null sink, which plays it in real time.
#
# Usage: shots.sh CINQ SHOW OUT, with sway, swaymsg, grim, wtype,
# pulseaudio, and pactl on PATH, ALSA_PLUGIN_DIR naming the directory of
# the pulse plugin of ALSA, CINQ the program, SHOW the directory of
# storm.cinq.toml and its files, and OUT the directory for the shots.
# SIZE sets the logical size of each output, 1280x960 by default, SCALE its
# scale, 2 by default, and SHOTS_HOME the empty home directory of the
# session, whose path the help page shows, a temporary one by default.
set -euo pipefail

cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
size=${SIZE:-1280x960} scale=${SCALE:-2}
work=$(mktemp -d)
pid='' sway=''

export HOME=${SHOTS_HOME:-$work/home} XDG_RUNTIME_DIR=$work/run
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
unset XDG_CONFIG_HOME XDG_DATA_HOME XDG_STATE_HOME XDG_CACHE_HOME
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  pulseaudio --kill 2> /dev/null || true
}
trap cleanup EXIT

pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix \
  -L 'module-null-sink sink_name=out rate=48000 channels=2'
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
printf 'pcm.!default { type pulse }\nctl.!default { type pulse }\n' > "$HOME/.asoundrc"

# Second output comes from create_output, so the rule that moves the stage
# window there applies once it exists
width=${size%x*} height=${size#*x}
cat > "$work/sway.config" << EOF
output HEADLESS-1 mode $((width * scale))x$((height * scale)) position 0 0 scale $scale
output HEADLESS-2 mode $((width * scale))x$((height * scale)) position $width 0 scale $scale
default_border none
default_floating_border none
focus_on_window_activation none
for_window [title=" stage - cinq\$"] move container to output HEADLESS-2
EOF
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
swaymsg -q create_output
sleep 1

# Keys go one wtype call each, which waits before typing, since sway drops
# keys sent right after wtype uploads its keymap.  Each character of the
# argument is one key, as wtype joins several arguments with spaces, and a
# space is GO.
keys() { wtype -s 300 "$@"; }
main() { swaymsg -q "[title=\"^$1 - cinq\$\"] focus"; }
shot() { grim -o "${2:-HEADLESS-1}" "$out/$1.png"; }
launch() {
  (cd "$show" && exec "$cinq" "$1" >> "$out/cinq.log" 2>&1) &
  pid=$!
  sleep 5
  main "$1"
  sleep 1
}
quit() {
  kill "$pid"
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 1
}

# Show as it opens, with cue 1 on standby and selected, then the first GO,
# after which the preshow music plays and the selection follows standby to
# cue 3
launch storm.cinq.toml
shot window
keys -k space
sleep 2
shot playing

# Trimmed sound on the Time & Loops tab, and a fade on the Levels tab
keys jjjj
keys gt
sleep 1
shot time
keys k
keys gt
sleep 1
shot levels
keys gt

# Fade cue on standby, which shows its chain, then that chain five seconds
# after its GO
keys gg
keys jj
sleep 1
shot chain
keys -k space
sleep 5
shot running

# Box of the keys that can follow g, then the help page
keys g
sleep 1
shot keys
keys g
keys '?'
sleep 1
shot help
keys q
keys -k Escape
sleep 1
keys -k Escape
sleep 2

# Title card on the stage, and the main window while it shows
keys l
sleep 1
keys -k space
sleep 3
shot stage HEADLESS-2
shot showing
quit

# New show of the reader's own, with one audio cue added, which has no file
# yet
launch my-show.cinq.toml
keys o
sleep 1
shot added
quit
