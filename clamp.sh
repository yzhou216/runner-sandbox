#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 3: plays loud sounds on one channel of ALSA's hw device of the
# kernel's loopback card, held to one format of samples, as the hw devices of
# sound cards are, under ALSA's copy plugin, which passes that on and lists
# the device: 16-bit (s16), 24-bit in 4 bytes (s24), packed 24-bit (s24p),
# 32-bit (s32), and 32-bit
# floating point (float), which takes no integers and so cannot wrap.  Each
# takes three shows, one output on channel 1 each:
#
# - single: one cue of clip.wav at 0 dB, which peaks at 0.8 of full scale
# - two: two cues of clip.wav at 0 dB, which one GO fires together, whose
#   sum is 1.6
# - boost: one cue of half.wav sent at +12 dB, which is 1.99
#
# arecord records the other end of the loopback through plughw as floating
# point, as NAME-SHOW.wav in OUT, and clip.py judges whether the peaks wrapped.
#
# Usage: clamp.sh CINQ SOUND OUT, with the snd-aloop module loaded, and sway,
# wtype, arecord, and ffmpeg on PATH, SOUND the directory of media.sh's sounds
set -euo pipefail
cinq=$(readlink -f "$1")
sound=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
here=$(cd "$(dirname "$0")" && pwd)
recorder=''
# shellcheck source=lib.sh
. "$here/lib.sh"
cleanup() {
  stop_cinq
  [ -n "$recorder" ] && kill -INT "$recorder" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
}
trap cleanup EXIT

devices=(
  "s16:format S16_LE"
  "s24:format S24_LE"
  "s24p:format S24_3LE"
  "s32:format S32_LE"
  "float:format FLOAT_LE"
)
{
  printf 'pcm.!default { type null }\n'
  for device in "${devices[@]}"; do
    name=${device%%:*}
    printf 'pcm.held_%s {\n  type hw\n  card Loopback\n  device 0\n  subdevice 0\n  %s\n}\n' "$name" "${device#*:}"
    printf 'pcm.%s {\n  type copy\n  slave.pcm held_%s\n  hint {\n    show on\n    description "Held to %s"\n  }\n}\n' "$name" "$name" "$name"
  done
} > "$HOME/.asoundrc"
cp "$HOME/.asoundrc" "$out/asoundrc"
cat /proc/asound/cards > "$out/cards.txt"
"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log" || true
cp "$sound/clip.wav" "$sound/half.wav" "$work/"
session

for device in "${devices[@]}"; do
  name=${device%%:*}
  {
    output 'Ch 1' "alsa:$name" 1
    list
    cue 1 Loud clip.wav manual 'Ch 1=0.0'
  } > "$work/$name-single.cinq.toml"
  {
    output 'Ch 1' "alsa:$name" 1
    list
    cue 1 First clip.wav continue 'Ch 1=0.0'
    cue 2 Second clip.wav manual 'Ch 1=0.0'
  } > "$work/$name-two.cinq.toml"
  {
    output 'Ch 1' "alsa:$name" 1
    list
    cue 1 Boosted half.wav manual 'Ch 1=12.0'
  } > "$work/$name-boost.cinq.toml"
  for show in single two boost; do
    phase=$name-$show
    cinq_start "$phase" "$phase"
    window
    sleep 6
    keys "$phase" go -k F5
    sleep 1
    timeout -k 2 9 arecord -D plughw:Loopback,1,0 -c 2 -r 48000 -f FLOAT_LE -t wav -d 5 "$out/$phase.wav" \
      > "$out/arecord-$phase.log" 2>&1 &
    recorder=$!
    wait "$recorder" 2> /dev/null || true
    recorder=''
    stop_cinq
    sleep 1
  done
done
