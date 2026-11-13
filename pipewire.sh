#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays partial.cinq.toml, whose outputs need three channels, on a null sink
# of eight channels in a PipeWire server of its own, which is its default
# sink, twice: once on jack:output, through PipeWire's JACK, whose ports
# should connect in order to those of the sink, and once on alsa:pipewire,
# through PipeWire's plugin of ALSA, which may spread three channels over
# eight as PulseAudio does.  pw-record records the sink each time.
#
# Usage: pipewire.sh CINQ SHOW OUT, with pipewire, wireplumber, pw-cli,
# wpctl, pw-record, pw-jack, sway, wtype, dbus-run-session, and ffmpeg on
# PATH, ALSA_PLUGIN_DIR naming the plugins of PipeWire for ALSA, PW_JACK
# the directory of PipeWire's libjack, CINQ the program, SHOW the directory
# of the show and its sound, and OUT the directory for what is recorded
set -euo pipefail
cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
work=$(mktemp -d)
pids=() sway='' pid=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'pcm.!default { type pipewire }\nctl.!default { type pipewire }\npcm.pipewire {\n  type pipewire\n  hint {\n    show on\n    description "PipeWire"\n  }\n}\n' > "$HOME/.asoundrc"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  [ "${#pids[@]}" -gt 0 ] && kill "${pids[@]}" 2> /dev/null || true
}
trap cleanup EXIT

pipewire > "$out/pipewire.log" 2>&1 &
pids+=("$!")
sleep 2
wireplumber > "$out/wireplumber.log" 2>&1 &
pids+=("$!")
sleep 3
pw-cli create-node adapter '{ factory.name=support.null-audio-sink node.name=desk node.description=Desk media.class=Audio/Sink object.linger=true audio.rate=48000 audio.channels=8 audio.position=[ AUX0 AUX1 AUX2 AUX3 AUX4 AUX5 AUX6 AUX7 ] monitor.channel-volumes=true }' \
  > "$out/node.log" 2>&1
sleep 2
id=$(pw-cli ls Node | awk '/^\s*id / { id = $2 } /node.name = "desk"/ { sub(",", "", id); print id }' | head -1)
wpctl set-default "$id" || true
wpctl status > "$out/status.txt" 2>&1 || true

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

LD_LIBRARY_PATH=$PW_JACK "$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log" || true
cat "$out/devices.tsv"

for device in jack:output alsa:pipewire; do
  name=${device%%:*}
  sed "s|^device = .*|device = \"$device\"|; /^device_name = /d" "$show/partial.cinq.toml" > "$show/$name.cinq.toml"
  pw-record --target desk -P '{ stream.capture.sink=true }' --channels 8 --format f32 \
    "$out/$name.wav" > "$out/record-$name.log" 2>&1 &
  recorder=$!
  (cd "$show" && LD_LIBRARY_PATH=$PW_JACK exec "$cinq" "$name.cinq.toml" > "$out/cinq-$name.log" 2>&1) &
  pid=$!
  sleep 6
  pw-link -l > "$out/links-$name.txt" 2>&1 || true
  wtype -s 300 -k space
  sleep 5
  wtype -s 300 -k Escape
  sleep 2
  kill -INT "$recorder" 2> /dev/null || true
  wait "$recorder" 2> /dev/null || true
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 2
done
