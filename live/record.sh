#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0
# Records cinq while it plays the show of make-show.sh and the keys of the
# steps below fire its start, pause, and GoTo cues: a headless sway session of its
# own shows the window, and a PulseAudio server of its own with a null sink
# own shows the window, and a PulseAudio server of its own with a null sink
# takes the sound, whose monitor is recorded.  Every key goes through wtype,
# so that the pointer is never needed.
#
# Usage: record.sh CINQ SHOW OUT FONTS, with sway, swaymsg, wtype,
# wf-recorder, grim, pulseaudio, pactl, and parec on PATH, ALSA_PLUGIN_DIR
# naming the directory of the pulse plugin of ALSA, CINQ the program, SHOW
# the directory of control.cinq.toml, OUT the directory for the recordings,
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

(cd "$show" && exec "$cinq" control.cinq.toml >> "$out/cinq.log" 2>&1) &
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
# key is Space or Escape.  The wait after each GO says how long its sound is
# heard, and a measure mark before the end of a wait gives the level of the
# first second after it.
caption "Cue 1 plays a tone whose level rises steadily.  GO fires it."
keys -k F5
event go-1
sleep 4
measure "GO on cue 1: the tone has risen for 4 s"

caption "GO fires cue 2, a pause cue on the tone.  The tone holds, and its State reads paused."
keys -k F5
event go-2
sleep 1
grim "$out/paused.png"
sleep 2
measure "GO on cue 2: the tone is paused, 3 s later"

caption "GO fires cue 3, a start cue on the tone.  It resumes from the level at which it paused."
keys -k F5
event go-3
sleep 1.3
measure "GO on cue 3: first second after the resume"
sleep 2.7
measure "GO on cue 3: the tone has run 4 s since the resume"

caption "GO fires cue 4, a GoTo cue on the tone.  Standby returns to cue 1, and nothing else happens."
keys -k F5
event go-4
sleep 2
grim "$out/goto.png"
measure "GO on cue 4: standby is on cue 1 and the tone goes on"

caption "GO fires cue 1 again, and the running tone restarts from the bottom."
keys -k F5
event go-1-again
sleep 1.3
measure "GO on cue 1 again: first second after the restart"
sleep 1.7
measure "GO on cue 1 again: the tone has run 3 s since the restart"

caption "In the list Effects, a start cue fires the tone of Main.  It leaves the standby cue of Main alone."
keys l
sleep 1
grim "$out/effects.png"
keys -k F5
event go-effects
sleep 1.3
measure "Start cue of Effects: first second after the restart"
sleep 1.7
measure "Start cue of Effects: the tone has run 3 s since the restart"

caption "Back in Main, standby is still on cue 2, where the GO on cue 1 left it."
keys h
sleep 2
grim "$out/main-again.png"

caption "F12 is Panic: everything fades out in 1 s and stops."
keys -k F12
event panic
sleep 3
event end

kill "$pid" 2> /dev/null || true
wait "$pid" 2> /dev/null || true
pid=''
sleep 1
kill -INT "$sound" "$screen"
wait "$sound" "$screen" 2> /dev/null || true
sound='' screen=''
