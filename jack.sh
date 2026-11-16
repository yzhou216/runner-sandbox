#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Check 1: plays on jack:output, the device of jackd with its dummy backend
# of 16 playback ports, outputs on channels 10, 11, and 12, which should
# connect to system:playback_10 to system:playback_12, where a sort of the
# port names as text would put 10 to 16 between 1 and 2.  Two shows play,
# each with one cue of stereo.wav that F5 fires:
#
# - pair: Pair on 10-11 at 0 dB and Mono 12 on 12 at -12 dB, which take
#   the left, the right, and the middle of the sound
# - singles: M10, M11, and M12 on channels 10, 11, and 12 at 0, -6, and
#   -12 dB, which take the middle each
#
# For each, jack_lsp -c says where the ports of cinq connect, and
# jack_capture records all 16 playback ports.
#
# Usage: jack.sh CINQ SOUND OUT, with jackd, jack_lsp, jack_capture, sway,
# wtype, and ffmpeg on PATH, SOUND the directory of media.sh's sounds, and
# OUT the directory for what is recorded
set -euo pipefail
cinq=$(readlink -f "$1")
sound=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
here=$(cd "$(dirname "$0")" && pwd)
server=''
# shellcheck source=lib.sh
. "$here/lib.sh"
cleanup() {
  stop_cinq
  [ -n "$server" ] && kill "$server" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
}
trap cleanup EXIT

# No ALSA device, so that sound has only JACK to go to
printf 'pcm.!default { type null }\nctl.!default { type hw card 0 }\n' > "$HOME/.asoundrc"
cp "$sound/stereo.wav" "$work/"
jackd --no-realtime -d dummy -r 48000 -p 1024 -P 16 -C 0 > "$out/jackd.log" 2>&1 &
server=$!
for _ in $(seq 50); do
  jack_lsp > /dev/null 2>&1 && break
  sleep 0.2
done
jack_lsp > "$out/ports.txt"
"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log" || true
session

{
  output Pair jack:output 10-11
  output 'Mono 12' jack:output 12
  list
  cue 1 Tone stereo.wav manual 'Pair=0.0' 'Mono 12=-12.0'
} > "$work/pair.cinq.toml"
{
  output M10 jack:output 10
  output M11 jack:output 11
  output M12 jack:output 12
  list
  cue 1 Tone stereo.wav manual 'M10=0.0' 'M11=-6.0' 'M12=-12.0'
} > "$work/singles.cinq.toml"

for name in pair singles; do
  cinq_start "$name" "$name"
  window
  sleep 6
  jack_lsp -c > "$out/connections-$name.txt"
  keys "$name" go -k F5
  sleep 1
  jack_capture --daemon --recording-time 4 --channels 16 --port 'system:playback*' "$out/$name.wav" \
    > "$out/capture-$name.log" 2>&1 || true
  stop_cinq
  sleep 2
done
