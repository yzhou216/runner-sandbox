#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays tones of 10 and 15 kHz from WAV files of 44.1 kHz, and one of 10 kHz
# from a file of 48 kHz, on jack:output, the device of jackd with its dummy
# backend at 48 kHz, with each cinq that CINQ names, and records the two
# playback ports with jack_capture, as BUILD-TONE.wav in OUT.
#
# Usage: resample.sh OUT CINQ..., with jackd, jack_capture, sway, wtype,
# and ffmpeg on PATH, OUT the directory for what is recorded, and each CINQ
# a directory whose bin/cinq is a build of cinq, named after the directory
set -euo pipefail
out=$(mkdir -p "$1" && readlink -f "$1")
shift
work=$(mktemp -d)
pids=() sway='' pid=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'pcm.!default { type null }\n' > "$HOME/.asoundrc"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  [ "${#pids[@]}" -gt 0 ] && kill "${pids[@]}" 2> /dev/null || true
}
trap cleanup EXIT

# Tones at half of full scale in 32-bit floating point, so that the file
# adds no noise of its own
for tone in 44100:10000 44100:15000 48000:10000; do
  ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=${tone#*:}:sample_rate=${tone%:*}:duration=20" \
    -af volume=0.5 -ac 2 -c:a pcm_f32le "$work/${tone%:*}-${tone#*:}.wav"
done

jackd --no-realtime -d dummy -r 48000 -p 1024 -P 2 -C 0 > "$out/jackd.log" 2>&1 &
pids+=("$!")
for _ in $(seq 50); do
  jack_lsp > /dev/null 2>&1 && break
  sleep 0.2
done

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

for build in "$@"; do
  cinq=$(readlink -f "$build")/bin/cinq
  name=$(basename "$build")
  for tone in 44100-10000 44100-15000 48000-10000; do
    printf '[[output]]\nname = "Main"\ndevice = "jack:output"\nchannels = "1-2"\n\n' > "$work/$tone.cinq.toml"
    printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "audio"\nname = "Tone"\nfile = "%s.wav"\n' \
      "$tone" >> "$work/$tone.cinq.toml"
    (cd "$work" && exec "$cinq" "$tone.cinq.toml" > "$out/cinq-$name-$tone.log" 2>&1) &
    pid=$!
    sleep 6
    wtype -s 300 -k space
    sleep 1
    jack_capture --daemon --recording-time 3 --channels 2 --port 'system:playback*' "$out/$name-$tone.wav" \
      > "$out/capture-$name-$tone.log" 2>&1 || true
    wtype -s 300 -k Escape
    sleep 1
    kill "$pid" 2> /dev/null || true
    wait "$pid" 2> /dev/null || true
    pid=''
    sleep 1
  done
done
