#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records cinq while it plays the show of make-show.sh and the keys of the
# steps below edit the levels of its cues: a headless sway session of its
# own shows the window, and a PulseAudio server of its own with a null sink
# takes the sound, whose monitor is recorded.  Every key goes through wtype,
# so that the pointer is never needed.
#
# Usage: record.sh CINQ SHOW OUT FONTS, with sway, swaymsg, wtype,
# wf-recorder, grim, pulseaudio, pactl, and parec on PATH, ALSA_PLUGIN_DIR
# naming the directory of the pulse plugin of ALSA, CINQ the program, SHOW
# the directory of levels.cinq.toml, OUT the directory for the recordings,
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
# argument is one key, as wtype joins several arguments with spaces, and a
# space is GO.  A text that begins with a hyphen follows two hyphens.
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

(cd "$show" && exec "$cinq" levels.cinq.toml >> "$out/cinq.log" 2>&1) &
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

# Key steps.  Cue 1 is on standby and selected as the show opens, GO fires
# the standby cue and moves standby down, and the selection follows standby.
# The inspector shows the Levels tab as the third, whose first field is the
# level of an audio cue and the level that a fade cue fades to.  Tab goes
# from the cue list to the first field of the inspector, with its text
# selected so that typing replaces it, and Enter leaves the field.  The
# number of Tabs to each field is a setting, in case a build puts other
# fields first.
level_tabs=${LEVEL_TABS:-1} fade_tabs=${FADE_TABS:-1}
tab() { for _ in $(seq "$1"); do keys -k Tab; done; }

caption "Cue 1, Tone, loops at -20 dB.  GO starts it."
keys -k space
event go-1
sleep 3
measure "Tone has played 3 s at -20 dB"

caption "Select cue 1, open the Levels tab, and Tab into the Level field."
keys k
sleep 1
keys 3gt
sleep 1
tab "$level_tabs"
sleep 1
grim "$out/levels-tab.png"
caption "Type -1, then wait 3 s without pressing Enter."
keys -- -1
event typed-minus-1
sleep 3
measure "-1 typed, 3 s later, Enter not pressed"

caption "Type 2, so that the field reads -12, then wait 3 s."
keys 2
event typed-2
sleep 3
measure "-12 typed, 3 s later, Enter not pressed"

caption "Press Enter to leave the field, then wait 3 s."
keys -k Return
event enter-12
sleep 3
measure "Enter pressed, 3 s later"

caption "GO fires cue 2, Duck: a fade to -30 dB in 2 s.  Wait 4 s."
keys -k space
event go-2
sleep 4
measure "Duck ended, 4 s later"

caption "Select cue 1, Tab into its Level field, type -9 (a raise of 3 dB), press Enter."
keys gg
sleep 1
tab "$level_tabs"
sleep 1
keys -- -9
sleep 1
keys -k Return
event edit-9
sleep 4
measure "Level edited to -9 after the duck, 4 s later"

caption "GO fires cue 3, Swell: a fade to ${SWELL:--6.0} dB in 12 s.  Wait 4 s."
keys -k space
event go-3
sleep 4
measure "Swell has run 4 s"

caption "Select cue 3, Tab into Fade target to, type -30, press Enter, then wait 10 s."
keys G
sleep 1
tab "$fade_tabs"
sleep 1
grim "$out/swell-field.png"
keys -- -30
sleep 1
keys -k Return
event edit-fade-30
sleep 10
measure "Fade target edited to -30 during the swell, 10 s later"
grim "$out/end.png"

caption "Escape is Panic: everything fades out in 1 s and stops."
keys -k Escape
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
