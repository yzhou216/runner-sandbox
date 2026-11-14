#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays sixteen looping cues at once on all 64 channels of jack:output, the
# device of jackd with its dummy backend at 48 kHz and periods of 256
# frames, as many channels as the Dante Virtual Soundcard offers, for
# SECONDS with each cinq that CINQ names.  The sound is a file of 44.1 kHz,
# which each build converts to 48 kHz its own way.  For each build it
# prints the share of a processor that cinq took and the xruns that jackd
# logged meanwhile, and records the 64 playback ports at the end as
# BUILD.wav in OUT.
#
# Usage: soak.sh OUT SECONDS CINQ..., with jackd, jack_capture, sway,
# wtype, and ffmpeg on PATH, and each CINQ a directory whose bin/cinq is a
# build of cinq, named after the directory
set -euo pipefail
out=$(mkdir -p "$1" && readlink -f "$1")
seconds=$2
shift 2
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

ffmpeg -loglevel error -y \
  -f lavfi -i "aevalsrc='0.3*sin(2*PI*t*440*pow(2,mod(floor(t*4),4)*4/12))':s=44100:d=60" \
  -ac 2 "$work/music.flac"
{
  for output in $(seq 32); do
    printf '[[output]]\nname = "O%s"\ndevice = "jack:output"\nchannels = "%s-%s"\n\n' \
      "$output" "$((2 * output - 1))" "$((2 * output))"
  done
  printf '[[list]]\nname = "Main"\n'
  for cue in $(seq 16); do
    advance=$([ "$cue" -lt 16 ] && echo continue || echo manual)
    printf '\n[[list.cue]]\nid = %s\nnumber = "%s"\nkind = "audio"\nname = "Loop %s"\n' "$cue" "$cue" "$cue"
    printf 'file = "music.flac"\nloop = true\nadvance = "%s"\n\n[list.cue.sends]\n' "$advance"
    printf 'O%s = 0.0\nO%s = -6.0\n' "$((2 * cue - 1))" "$((2 * cue))"
  done
} > "$work/soak.cinq.toml"
cp "$work/soak.cinq.toml" "$out/"

jackd --no-realtime -d dummy -r 48000 -p 256 -P 64 -C 0 > "$out/jackd.log" 2>&1 &
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

# Processor time of process $1 so far, in ticks of the clock
ticks() { awk '{ print $14 + $15 }' "/proc/$1/stat"; }
xruns() { command grep -ci xrun "$out/jackd.log" || true; }

printf 'build\tprocessor\txruns\n'
for build in "$@"; do
  cinq=$(readlink -f "$build")/bin/cinq
  name=$(basename "$build")
  (cd "$work" && exec "$cinq" soak.cinq.toml > "$out/cinq-$name.log" 2>&1) &
  pid=$!
  sleep 6
  wtype -s 300 -k space
  sleep 5
  grim "$out/$name.png" 2> /dev/null || true
  before=$(ticks "$pid") xruns_before=$(xruns)
  sleep "$seconds"
  after=$(ticks "$pid") xruns_after=$(xruns)
  jack_capture --daemon --recording-time 5 --channels 64 --port 'system:playback*' "$out/$name.wav" \
    > "$out/capture-$name.log" 2>&1 || true
  awk -v ticks="$((after - before))" -v seconds="$seconds" -v hz="$(getconf CLK_TCK)" -v name="$name" \
    -v xruns="$((xruns_after - xruns_before))" \
    'BEGIN { printf "%s\t%.1f%%\t%d\n", name, 100 * ticks / hz / seconds, xruns }'
  wtype -s 300 -k Escape
  sleep 1
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 2
done
