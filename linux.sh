#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays the show of show.sh on jack:output, the device of jackd with its
# dummy backend of 64 playback ports, which Over on channels 65-66 overruns,
# in a headless sway of 1024x768, takes screenshots of the main window, the
# Levels tab, the outputs view, and the inspector of Stage L, and records
# the first two playback ports as house.wav, all in OUT.
#
# Usage: linux.sh CINQ OUT, with jackd, jack_capture, sway, wtype, grim,
# and ffmpeg on PATH
set -euo pipefail
cinq=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
here=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
sway='' pid='' server=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'pcm.!default { type null }\n' > "$HOME/.asoundrc"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$server" ] && kill "$server" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
}
trap cleanup EXIT

ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=60" \
  -af volume=0.5 -ac 2 -c:a pcm_f32le "$work/tone.wav"
bash "$here/show.sh" jack:output "JACK Output" 65-66 alsa:nothing > "$work/ui.cinq.toml"
cp "$work/ui.cinq.toml" "$out/"

jackd --no-realtime -d dummy -r 48000 -p 256 -P 64 -C 0 > "$out/jackd.log" 2>&1 &
server=$!
for _ in $(seq 50); do
  jack_lsp > /dev/null 2>&1 && break
  sleep 0.2
done

printf 'output HEADLESS-1 mode 1024x768\ndefault_border none\n' > "$work/sway.config"
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
for _ in $(seq 300); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
  [ -n "$socket" ] && [ -n "$SWAYSOCK" ] && SWAYSOCK=$SWAYSOCK swaymsg -t get_version > /dev/null 2>&1 && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket SWAYSOCK

(cd "$work" && exec "$cinq" ui.cinq.toml > "$out/cinq.log" 2>&1) &
pid=$!
sleep 6
wtype -s 300 -k space
sleep 1
jack_capture --daemon --recording-time 4 --channels 2 --port system:playback_1 --port system:playback_2 "$out/house.wav" \
  > "$out/capture.log" 2>&1 &
recording=$!
sleep 1
grim "$out/main.png"
wtype -s 300 gt
sleep 1
wtype -s 300 gt
sleep 1
grim "$out/levels.png"
wtype -s 300 l
sleep 1
grim "$out/outputs.png"
wtype -s 300 jjj
sleep 1
grim "$out/inspector.png"
wait "$recording" || true
