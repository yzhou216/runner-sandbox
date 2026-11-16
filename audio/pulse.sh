#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays on four null sinks of a PulseAudio server of its own, which ALSA's
# pulse plugin names alsa:house, alsa:booth, alsa:lobby, and alsa:stage.
# parecord records the monitor of each sink.
#
# Check 2, show two: House and Booth, outputs on 1-2 of two devices, with
# one cue of stereo.wav that sends to House at 0 dB and to Booth at -6 dB.
# F5 fires it, and Escape, Panic, stops it, with nothing else playing.
#
# Check 4, show slow: outputs on all four devices, which open one after the
# other, and memo cues.  The server stops for two seconds, runs for a
# second, and stops again, as cinq starts, so that each opening of a device
# waits on the server for seconds, as one of the plugin of PulseAudio does
# when it starts a stream.  The window is typed into as soon as it shows, 36
# keys of j and k 250 ms apart, so that some arrive while devices open.
# slow.log holds the time of each key (K lines), update (U lines), and open
# (O lines).
#
# Usage: pulse.sh CINQ SOUND OUT, with pulseaudio, pactl, parecord, sway,
# wtype, and ffmpeg on PATH, ALSA_PLUGIN_DIR naming the directory of the
# pulse plugin of ALSA, SOUND the directory of media.sh's sounds, and OUT
# the directory for what is recorded
set -euo pipefail
cinq=$(readlink -f "$1")
sound=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
here=$(cd "$(dirname "$0")" && pwd)
recorders=() staller=''
# shellcheck source=lib.sh
. "$here/lib.sh"
cleanup() {
  stop_cinq
  [ "${#recorders[@]}" -gt 0 ] && kill -INT "${recorders[@]}" 2> /dev/null || true
  [ -n "$staller" ] && kill "$staller" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  [ -s "$XDG_RUNTIME_DIR/pulse/pid" ] && kill -CONT "$(cat "$XDG_RUNTIME_DIR/pulse/pid")" 2> /dev/null || true
  pulseaudio --kill 2> /dev/null || true
}
trap cleanup EXIT

sinks=(house booth lobby stage)
modules=()
for sink in "${sinks[@]}"; do
  modules+=(-L "module-null-sink sink_name=$sink rate=48000 channels=2 sink_properties=device.description=$sink")
done
pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix "${modules[@]}"
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
pactl set-default-sink house
{
  printf 'pcm.!default { type pulse }\nctl.!default { type pulse }\n'
  for sink in "${sinks[@]}"; do
    printf 'pcm.%s {\n  type pulse\n  device %s\n  hint {\n    show on\n    description "%s"\n  }\n}\n' "$sink" "$sink" "$sink"
  done
} > "$HOME/.asoundrc"
"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log" || true
cp "$sound/stereo.wav" "$work/"
session

{
  output House alsa:house 1-2
  output Booth alsa:booth 1-2
  list
  cue 1 Tone stereo.wav manual 'House=0.0' 'Booth=-6.0'
} > "$work/two.cinq.toml"
{
  for sink in "${sinks[@]}"; do output "$sink" "alsa:$sink" 1-2; done
  list
  for id in 1 2 3 4 5 6; do memo "$id" "Memo $id"; done
} > "$work/slow.cinq.toml"

# Check 2
for sink in house booth; do
  parecord -d "$sink.monitor" --rate=48000 --channels=2 --file-format=wav "$out/$sink.wav" &
  recorders+=("$!")
done
cinq_start two two
window
sleep 9
keys two go -k F5
sleep 6
keys two panic -k Escape
sleep 4
kill -INT "${recorders[@]}" 2> /dev/null || true
wait "${recorders[@]}" 2> /dev/null || true
recorders=()
stop_cinq
sleep 2

# Check 4: the server stops and runs again while cinq starts, and the keys
# come as soon as the window shows
server=$(cat "$XDG_RUNTIME_DIR/pulse/pid")
(
  for _ in 1 2 3 4; do
    kill -STOP "$server"
    sleep 2
    kill -CONT "$server"
    sleep 1
  done
) &
staller=$!
cinq_start slow slow
window
KEYDELAY=250 keys slow typing 'jkjkjkjkjkjkjkjkjkjkjkjkjkjkjkjkjkjk'
sleep 12
stop_cinq

# Check 5, on GNU/Linux too: Left on channel 1 and Right on channel 2 of
# alsa:house, with the keys of the outputs view and of the Levels tab, as on
# Windows and Darwin
{
  output Left alsa:house 1
  output Right alsa:house 2
  list
  cue 1 Tone stereo.wav manual 'Left=0.0' 'Right=0.0'
} > "$work/keys.cinq.toml"
cinq_start keys keys
window
sleep 8
for step in 'outputs l:l' 'outputs j:j' 'outputs o:o' 'outputs d d:dd' 'outputs p:p'; do
  keys keys "${step%%:*}" "${step#*:}"
  sleep 1.5
done
keys keys 'outputs ctrl-o' -M ctrl o -m ctrl
sleep 1.5
parecord -d house.monitor --rate=48000 --channels=2 --file-format=wav "$out/keys-before.wav" &
recorders+=("$!")
sleep 1
keys keys 'go F5' -k F5
sleep 5
kill -INT "${recorders[@]}" 2> /dev/null || true
wait "${recorders[@]}" 2> /dev/null || true
recorders=()
keys keys 'levels g t 1' gt
sleep 1.5
keys keys 'levels g t 2' gt
sleep 1.5
for n in 1 2 3; do
  keys keys "levels tab $n" -k Tab
  sleep 1
done
keys keys 'levels select all' -M ctrl a -m ctrl
sleep 1
keys keys 'levels send' -- -12
sleep 1
keys keys 'levels enter' -k Return
sleep 1
parecord -d house.monitor --rate=48000 --channels=2 --file-format=wav "$out/keys-after.wav" &
recorders+=("$!")
sleep 6
kill -INT "${recorders[@]}" 2> /dev/null || true
wait "${recorders[@]}" 2> /dev/null || true
recorders=()
stop_cinq
