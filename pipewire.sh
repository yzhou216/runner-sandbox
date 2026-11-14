#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays shows whose outputs need 3, 4, 6, and 8 channels, one output on
# each channel, each 3 dB below the one before, on a null sink of eight
# channels in a PipeWire server of its own, which is its default sink, and
# records the sink each time.  The sink takes the positions AUX0 to AUX7,
# as a desk's channels do, then those of 7.1, as a sound card's do.  Each
# show plays on jack:output, through PipeWire's JACK, whose ports should
# connect in order to those of the sink, and on alsa:pipewire, through
# PipeWire's plugin of ALSA, which gives channels positions of its own.
#
# Usage: pipewire.sh CINQ SOUND OUT, with pipewire, wireplumber, pw-cli,
# wpctl, pw-record, pw-link, pw-dump, sway, wtype, grim, and ffmpeg on
# PATH, ALSA_PLUGIN_DIR naming the plugins of PipeWire for ALSA, PW_JACK
# the directory of PipeWire's libjack, CINQ the program, SOUND the sound of
# the cue, and OUT the directory for what is recorded, as
# SINK-DEVICE-CHANNELS.wav
set -euo pipefail
cinq=$(readlink -f "$1")
sound=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
work=$(mktemp -d)
server=() sway='' pid=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'pcm.!default { type pipewire }\nctl.!default { type pipewire }\npcm.pipewire {\n  type pipewire\n  hint {\n    show on\n    description "PipeWire"\n  }\n}\n' > "$HOME/.asoundrc"
stop_server() {
  [ "${#server[@]}" -gt 0 ] && kill "${server[@]}" 2> /dev/null || true
  [ "${#server[@]}" -gt 0 ] && wait "${server[@]}" 2> /dev/null || true
  server=()
}
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  stop_server
}
trap cleanup EXIT

# Starts PipeWire with the sink desk on the positions $2, as the default
# sink, logging under the name $1
start_server() {
  pipewire > "$out/pipewire-$1.log" 2>&1 &
  server+=("$!")
  sleep 2
  wireplumber > "$out/wireplumber-$1.log" 2>&1 &
  server+=("$!")
  sleep 3
  pw-cli create-node adapter "{ factory.name=support.null-audio-sink node.name=desk node.description=Desk media.class=Audio/Sink object.linger=true audio.rate=48000 audio.channels=8 audio.position=[ $2 ] monitor.channel-volumes=true }" \
    > "$out/node-$1.log" 2>&1
  sleep 2
  local id
  id=$(pw-cli ls Node | awk '/^\s*id / { id = $2 } /node.name = "desk"/ { sub(",", "", id); print id }' | head -1)
  wpctl set-default "$id" || true
  wpctl status > "$out/status-$1.txt" 2>&1 || true
}

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

cp "$sound" "$work/music.flac"
# Writes the show of $1 channels on device $2: output Ch K on channel K,
# to which the cue sends 3 (K - 1) dB down
show() {
  local channel
  for channel in $(seq "$1"); do
    printf '[[output]]\nname = "Ch %s"\ndevice = "%s"\nchannels = "%s"\n\n' "$channel" "$2" "$channel"
  done
  printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\n'
  printf 'kind = "audio"\nname = "Music"\nfile = "music.flac"\n\n[list.cue.sends]\n'
  for channel in $(seq "$1"); do
    printf '"Ch %s" = %s.0\n' "$channel" "$((-3 * (channel - 1)))"
  done
}

for sink in aux surround; do
  case $sink in
    aux) positions='AUX0 AUX1 AUX2 AUX3 AUX4 AUX5 AUX6 AUX7' ;;
    surround) positions='FL FR FC LFE RL RR SL SR' ;;
  esac
  start_server "$sink" "$positions"
  LD_LIBRARY_PATH=$PW_JACK "$cinq" --audio-devices > "$out/devices-$sink.tsv" 2> "$out/devices-$sink.log" || true
  for device in jack:output alsa:pipewire; do
    for channels in 3 4 6 8; do
      take=$sink-${device%%:*}-$channels
      show "$channels" "$device" > "$work/$take.cinq.toml"
      pw-record --target desk -P '{ stream.capture.sink=true }' --channels 8 --format f32 \
        --channel-map "${positions// /,}" "$out/$take.wav" > "$out/record-$take.log" 2>&1 &
      recorder=$!
      (cd "$work" && LD_LIBRARY_PATH=$PW_JACK exec "$cinq" "$take.cinq.toml" > "$out/cinq-$take.log" 2>&1) &
      pid=$!
      sleep 6
      pw-link -l > "$out/links-$take.txt" 2>&1 || true
      pw-dump > "$out/dump-$take.json" 2>&1 || true
      wtype -s 300 -k space
      sleep 5
      grim "$out/$take.png" || true
      wtype -s 300 -k Escape
      sleep 2
      kill -INT "$recorder" 2> /dev/null || true
      wait "$recorder" 2> /dev/null || true
      kill "$pid" 2> /dev/null || true
      wait "$pid" 2> /dev/null || true
      pid=''
      sleep 1
    done
  done
  stop_server
  sleep 2
done
