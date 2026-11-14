#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a tone of 1 kHz on the hw device of the kernel's loopback card with
# each cinq that CINQ names, for ten seconds, while four loops keep the
# processors busy and grim takes a screenshot every half second, as a show
# machine that decodes video and draws its windows meanwhile.  arecord
# records the other end of the loopback as NAME.wav in OUT, and the
# scheduling of every thread of cinq goes to NAME-threads.txt.
#
# Usage: rt.sh OUT NAME=CINQ..., with the snd-aloop module loaded, and
# sway, wtype, grim, arecord, ps, and ffmpeg on PATH
set -euo pipefail
out=$(mkdir -p "$1" && readlink -f "$1")
shift
work=$(mktemp -d)
sway='' pid='' busy=()

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'pcm.!default { type null }\n' > "$HOME/.asoundrc"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ "${#busy[@]}" -gt 0 ] && kill "${busy[@]}" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
}
trap cleanup EXIT

ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=60" \
  -af volume=0.5 -ac 2 -c:a pcm_f32le "$work/tone.wav"
{
  for channel in 1 2 3; do
    printf '[[output]]\nname = "Ch %s"\ndevice = "alsa:hw:CARD=0,DEV=0"\nchannels = "%s"\n\n' "$channel" "$channel"
  done
  printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "audio"\nname = "Tone"\nfile = "tone.wav"\n\n'
  printf '[list.cue.sends]\n"Ch 1" = 0.0\n"Ch 2" = -3.0\n"Ch 3" = -6.0\n'
} > "$work/hw.cinq.toml"

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
  name=${build%%=*}
  cinq=${build#*=}
  (cd "$work" && exec "$cinq" hw.cinq.toml > "$out/cinq-$name.log" 2>&1) &
  pid=$!
  sleep 5
  wtype -s 300 -k space
  sleep 1
  ps -L -o tid,cls,rtprio,ni,comm -p "$pid" > "$out/$name-threads.txt" || true
  for _ in 1 2 3 4; do
    sh -c 'while :; do :; done' &
    busy+=("$!")
  done
  timeout -k 2 14 arecord -D plughw:Loopback,1,0 -c 3 -r 48000 -f FLOAT_LE -t wav -d 10 "$out/$name.wav" \
    > "$out/arecord-$name.log" 2>&1 &
  recorder=$!
  for _ in $(seq 20); do
    grim "$out/$name.png" 2> /dev/null || true
    sleep 0.5
  done
  wait "$recorder" 2> /dev/null || true
  kill "${busy[@]}" 2> /dev/null || true
  busy=()
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 2
done
