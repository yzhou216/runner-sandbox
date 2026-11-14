#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a tone of 1 kHz on three outputs, on channels 1 to 3 at 0, -3, and
# -6 dB, on devices that stand in for the quirks of sound cards: ALSA's hw
# plugin over the kernel's loopback card, held to one format of samples,
# one count of channels, or one rate, as the hw devices of cards are, under
# ALSA's copy plugin, which passes that on and lists the device, and
# plug over the one of packed 24-bit samples, and last outputs on channels
# 1 to 3 and 7 of the device of six channels.  arecord records the other
# end of the loopback through plughw, and each take is NAME.wav in OUT,
# with a screenshot of cinq as NAME.png.
#
# Usage: alsa.sh CINQ OUT, with the snd-aloop module loaded, and sway,
# wtype, grim, arecord, and ffmpeg on PATH
set -euo pipefail
cinq=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)
sway='' pid='' recorder=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$recorder" ] && kill -INT "$recorder" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
}
trap cleanup EXIT

# Devices, by name, and what holds each to one way of taking samples
devices=(
  "loop:"
  "s16:format S16_LE"
  "s32:format S32_LE"
  "s24p:format S24_3LE"
  "float:format FLOAT_LE"
  "ch6:channels 6"
  "r44:rate 44100"
  "r96:rate 96000"
)
{
  printf 'pcm.!default { type null }\n'
  for device in "${devices[@]}"; do
    name=${device%%:*}
    # hw device held to the quirk, under copy, which passes on what the hw
    # device takes and, unlike a hw device of the configuration, lists
    printf 'pcm.held_%s {\n  type hw\n  card Loopback\n  device 0\n  subdevice 0\n  %s\n}\n' \
      "$name" "${device#*:}"
    printf 'pcm.%s {\n  type copy\n  slave.pcm held_%s\n  hint {\n    show on\n    description "%s"\n  }\n}\n' \
      "$name" "$name" "Quirk $name"
  done
  printf 'pcm.s24pplug {\n  type plug\n  slave.pcm held_s24p\n  hint {\n    show on\n    description "Quirk s24p through plug"\n  }\n}\n'
} > "$HOME/.asoundrc"
cp "$HOME/.asoundrc" "$out/asoundrc"
cat /proc/asound/cards > "$out/cards.txt"
"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log" || true

ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=40" \
  -af volume=0.5 -ac 2 -c:a pcm_f32le "$work/tone.wav"

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

for name in loop s16 s32 s24p float ch6 r44 r96 s24pplug; do
  {
    for channel in 1 2 3; do
      printf '[[output]]\nname = "Ch %s"\ndevice = "alsa:%s"\nchannels = "%s"\n\n' "$channel" "$name" "$channel"
    done
    printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "audio"\nname = "Tone"\nfile = "tone.wav"\n\n'
    printf '[list.cue.sends]\n"Ch 1" = 0.0\n"Ch 2" = -3.0\n"Ch 3" = -6.0\n'
  } > "$work/$name.cinq.toml"
  (cd "$work" && exec "$cinq" "$name.cinq.toml" > "$out/cinq-$name.log" 2>&1) &
  pid=$!
  sleep 5
  wtype -s 300 -k space
  sleep 1
  timeout -k 2 9 arecord -D plughw:Loopback,1,0 -c 3 -r 48000 -f FLOAT_LE -t wav -d 5 "$out/$name.wav" \
    > "$out/arecord-$name.log" 2>&1 &
  recorder=$!
  sleep 3
  grim "$out/$name.png" || true
  wait "$recorder" 2> /dev/null || true
  recorder=''
  wtype -s 300 -k Escape
  sleep 1
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 1
done

# Outputs on channels 1 to 3 of the device of six channels, and one on
# channel 7, past them, which should warn while the others play
{
  for channel in 1 2 3; do
    printf '[[output]]\nname = "Ch %s"\ndevice = "alsa:ch6"\nchannels = "%s"\n\n' "$channel" "$channel"
  done
  printf '[[output]]\nname = "Over"\ndevice = "alsa:ch6"\nchannels = "7"\n\n'
  printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "audio"\nname = "Tone"\nfile = "tone.wav"\n\n'
  printf '[list.cue.sends]\n"Ch 1" = 0.0\n"Ch 2" = -3.0\n"Ch 3" = -6.0\nOver = -6.0\n'
} > "$work/ch6over.cinq.toml"
(cd "$work" && exec "$cinq" ch6over.cinq.toml > "$out/cinq-ch6over.log" 2>&1) &
pid=$!
sleep 5
wtype -s 300 -k space
sleep 1
timeout -k 2 9 arecord -D plughw:Loopback,1,0 -c 3 -r 48000 -f FLOAT_LE -t wav -d 5 "$out/ch6over.wav" \
  > "$out/arecord-ch6over.log" 2>&1 &
recorder=$!
sleep 3
grim "$out/ch6over.png" || true
wait "$recorder" 2> /dev/null || true
recorder=''
kill "$pid" 2> /dev/null || true
wait "$pid" 2> /dev/null || true
pid=''
