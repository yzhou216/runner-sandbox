#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
# Records cinq while it plays the show of make-show.sh and the keys of the
# steps below show its loop and fire its devamp cue: a headless sway session of
# its own shows the window, and a PulseAudio server of its own with a null sink
# takes the sound, whose monitor is recorded.  Every key goes through wtype,
# so that the pointer is never needed.
#
# Usage: record.sh CINQ SHOW OUT FONTS, with sway, swaymsg, wtype,
# wf-recorder, grim, pulseaudio, pactl, and parec on PATH, ALSA_PLUGIN_DIR
# naming the directory of the pulse plugin of ALSA, CINQ the program, SHOW
# the directory of devamp.cinq.toml, OUT the directory for the recordings,
# and FONTS a directory of fonts that cinq draws in.  events.tsv in OUT
# holds, in seconds since the epoch, the moments that compose.sh reads: the
# start of each recorder, the flash that ties the screen's clock to the
# wall clock, each caption, and each moment at which the sound is measured.
set -euo pipefail

cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
fonts=$(readlink -f "$4")
size=1024x768
work=$(mktemp -d)
pid='' sway='' screen='' sound=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export XDG_STATE_HOME=$work/home/.state
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
export FONTCONFIG_FILE=$work/fonts.conf
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
unset XDG_DATA_HOME XDG_CACHE_HOME
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
# Keys of the show: GO on F5 and Panic on F12, so that wtype sends neither
# Space nor Escape
mkdir -p "$XDG_CONFIG_HOME/cinq"
printf '[keys]\ngo = ["F5"]\npanic = ["F12"]\n' > "$XDG_CONFIG_HOME/cinq/config.toml"
cat > "$FONTCONFIG_FILE" << EOF
<?xml version="1.0"?>
<fontconfig><dir>$fonts</dir><cachedir>$work/fontcache</cachedir></fontconfig>
EOF
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$screen" ] && kill -INT "$screen" 2> /dev/null || true
  [ -n "$sound" ] && kill -INT "$sound" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  pulseaudio --kill 2> /dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

: > "$out/events.tsv"
event() { printf '%s\t%s\t%s\n' "$(date +%s.%N)" "$1" "${2:-}" >> "$out/events.tsv"; }
# Caption shown from now until the next one, and a mark at which compose.sh
# measures the sound of the wait that ends here
caption() { event caption "$1"; }
measure() { event measure "$1"; }

pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix \
  -L 'module-null-sink sink_name=out rate=48000 channels=2'
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
printf 'pcm.!default { type pulse }\nctl.!default { type pulse }\n' > "$HOME/.asoundrc"

width=${size%x*} height=${size#*x}
cat > "$work/sway.config" << EOF
output HEADLESS-1 mode ${width}x${height} position 0 0 bg #000000 solid_color
default_border none
default_floating_border none
focus_on_window_activation none
EOF
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
# Sway's sockets appear before it answers on them, which can take seconds
for _ in $(seq 300); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
  [ -n "$socket" ] && [ -n "$SWAYSOCK" ] && SWAYSOCK=$SWAYSOCK swaymsg -t get_version > /dev/null 2>&1 && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket SWAYSOCK
sleep 1

# Keys go one wtype call each, which waits before typing, since sway drops
# keys sent right after wtype uploads its keymap.  Each character of the
# argument is one key, as wtype joins several arguments with spaces.  GO and
# Panic are F5 and F12, which config.toml binds, and never Space or Escape.
keys() { wtype -s 300 "$@"; }
focus() { swaymsg -q '[app_id="cinq"] focus' || true; }

wf-recorder -D -r 15 -x yuv420p -f "$out/screen.mp4" > "$out/wf-recorder.log" 2>&1 &
screen=$!
until [ -s "$out/screen.mp4" ]; do sleep 0.1; done
event screen-start
# One white frame, whose place in the video ties the video to the clock
sleep 1
swaymsg -q output HEADLESS-1 bg '#ffffff' solid_color
event flash
sleep 1
swaymsg -q output HEADLESS-1 bg '#000000' solid_color
sleep 1
parec -d out.monitor --rate=48000 --channels=2 --file-format=wav "$out/sound.wav" &
sound=$!
event sound-start
sleep 1

(cd "$show" && exec "$cinq" devamp.cinq.toml >> "$out/cinq.log" 2>&1) &
pid=$!
for _ in $(seq 300); do
  swaymsg -t get_tree 2> /dev/null | grep -q '"app_id": "cinq"' && break
  sleep 0.1
done
event window
sleep 4
focus
sleep 1
grim "$out/start.png"

# Key steps.  Cue 1 of Main is on standby and selected as the show opens, GO
# fires the standby cue and moves standby down, and the selection follows
# standby.  GO and Panic are bound to F5 and F12 in config.toml, so that no
# key is Space or Escape.  Times after the first GO are counted from it, so
# that the steps keep their place in the sound whatever a key takes.
# Waits until SECONDS after the first GO
go=0
at() {
  sleep "$(awk -v g="$go" -v d="$1" -v n="$(date +%s.%N)" 'BEGIN { s = g + d - n; print (s > 0 ? s : 0) }')"
}

# 2gt shows the second tab of the inspector, Time & Loops
caption "Cue 1 loops the section from 2 s to 6 s of its file.  GO fires it."
keys 2gt
sleep 1
grim "$out/tabs.png"
sleep 2

caption "GO fires cue 1: the introduction plays, then the vamp goes round its loop."
keys -k F5
go=$(date +%s.%N)
event go-1
at 5
grim "$out/vamp.png"
at 10.8
measure "Second pass of the vamp: first second after the loop end"

# The third pass starts 10 s after GO, and this key is typed 0.3 s after the
# call, so that it falls about 2 s into the pass
at 11.5
caption "GO fires cue 2, a devamp cue.  Cue 1 finishes the pass under way, then goes on."
keys -k F5
event devamp
at 13
grim "$out/devamp.png"

# The pass ends 14 s after GO
at 14.3
caption "The pass has ended: the sound goes on to the ending, not to the vamp."
event pass-end
at 17
measure "The ending: 3 s after the loop end"

at 18.5
grim "$out/ending.png"
caption "The ending is over: cue 1 has left the Active list and the State column."
at 20.5
grim "$out/end.png"
event end

kill "$pid" 2> /dev/null || true
wait "$pid" 2> /dev/null || true
pid=''
sleep 1
kill -INT "$sound" "$screen"
wait "$sound" "$screen" 2> /dev/null || true
sound='' screen=''
