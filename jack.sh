#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays two shows on jack:output, the device of a JACK server, here jackd
# with its dummy backend of eight playback ports: partial.cinq.toml, whose
# outputs need three channels, and full.cinq.toml, whose outputs fill all
# eight.  For each, jack_lsp tells where the ports of cinq connect, which
# should be the playback port of each one's number, and jack_capture
# records what the ports carry.
#
# Usage: jack.sh CINQ SHOW OUT, with jackd, jack_lsp, jack_capture, sway,
# wtype, and ffmpeg on PATH, CINQ the program, SHOW the directory of the
# shows and their sound, and OUT the directory for what is recorded
set -euo pipefail
cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
work=$(mktemp -d)
pids=() sway='' pid=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
# No ALSA device, so that sound has only JACK to go to
printf 'pcm.!default { type null }\nctl.!default { type hw card 0 }\n' > "$HOME/.asoundrc"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  [ "${#pids[@]}" -gt 0 ] && kill "${pids[@]}" 2> /dev/null || true
}
trap cleanup EXIT

jackd --no-realtime -d dummy -r 48000 -p 1024 -P 8 -C 0 > "$out/jackd.log" 2>&1 &
pids+=("$!")
for _ in $(seq 50); do
  jack_lsp > /dev/null 2>&1 && break
  sleep 0.2
done
jack_lsp > "$out/ports.txt"
"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log" || true
cat "$out/devices.tsv"

printf 'output HEADLESS-1 mode 1280x900\ndefault_border none\n' > "$work/sway.config"
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
for _ in $(seq 300); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
  [ -n "$socket" ] && [ -n "$SWAYSOCK" ] && SWAYSOCK=$SWAYSOCK swaymsg -t get_version > /dev/null 2>&1 && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket SWAYSOCK

for name in partial full; do
  (cd "$show" && exec "$cinq" "$name.cinq.toml" > "$out/cinq-$name.log" 2>&1) &
  pid=$!
  sleep 6
  jack_lsp -c > "$out/connections-$name.txt"
  ports=$(jack_lsp | command grep -c '^cpal_client' || true)
  echo "$name: cinq has $ports ports"
  wtype -s 300 -k space
  sleep 1
  # Ports of cinq, in order, each into a channel of its own
  capture=()
  while read -r port; do
    capture+=(--port "$port")
  done < <(jack_lsp | command grep '^cpal_client' | sort -V)
  jack_capture --daemon --format wav --duration 4 "${capture[@]}" "$out/$name.wav" \
    > "$out/capture-$name.log" 2>&1 || true
  wtype -s 300 -k Escape
  sleep 2
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 2
done
