#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays riscv.cinq.toml in cinq, in a headless sway session of its own whose
# sound goes to a PulseAudio null sink that records it: GO starts a chord,
# and GO fades it out, after which a video follows, whose picture shows its
# own time and whose sound is a 440 Hz tone.  A screenshot is taken at each
# step, steps.txt gives the time of each since the recording began, with the
# processor time that cinq had used by then, and levels.txt gives the level
# of the recording and the pitch of a tone in it for each half second.
#
# Usage: play.sh CINQ SHOW OUT, with sway, grim, wtype, pulseaudio, pactl,
# parecord, and ffmpeg on PATH, CINQ the program, SHOW the directory of
# riscv.cinq.toml and its files, and OUT the directory for what it records.
set -euo pipefail

cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
work=$(mktemp -d)
pid='' recorder='' sway=''

# Session apart from any other, with a home, a runtime directory, a display,
# and a sound server of its own
export HOME=$work/home XDG_RUNTIME_DIR=$work/run
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
unset XDG_CONFIG_HOME XDG_DATA_HOME XDG_STATE_HOME XDG_CACHE_HOME
mkdir "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
# Ends what the session started, sway by its own command, since a wrapper of
# sway, as in Nixpkgs, can leave it running when killed
cleanup() {
  for process in $pid $recorder; do
    kill "$process" 2> /dev/null || true
  done
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  pulseaudio --kill 2> /dev/null || true
}
trap cleanup EXIT

# Sound server, without which the show still plays, with no sound device
pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix \
  -L 'module-null-sink sink_name=out rate=48000 channels=2' || true
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
if pactl info | grep -E '^(Server Version|Default Sink)'; then
  sound=yes
  printf 'pcm.!default { type pulse }\nctl.!default { type pulse }\n' > "$HOME/.asoundrc"
else
  echo "PulseAudio did not start:"
  cat "$out/pulse.log" || true
  sound=''
fi

printf 'output HEADLESS-1 resolution 1280x720\nxwayland disable\n' > "$work/sway.config"
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
socket=''
for _ in $(seq 300); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
[ -n "$socket" ] || { cat "$out/sway.log"; exit 1; }
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK

# Recording, whose first sound comes well over a second after parecord
# starts, and from which the steps are timed
if [ -n "$sound" ]; then
  parecord --device=out.monitor --rate=48000 --channels=2 --format=s16le \
    --latency-msec=20 --file-format=wav "$out/sound.wav" &
  recorder=$!
  for _ in $(seq 200); do
    [ "$(stat -c %s "$out/sound.wav" 2> /dev/null || echo 0)" -gt 4096 ] && break
    sleep 0.05
  done
fi
start=$(date +%s.%N)
tick=$(getconf CLK_TCK)

# Notes a step with the seconds since the recording began and the processor
# time that cinq has used
mark() {
  local now ticks
  now=$(date +%s.%N)
  ticks=$(awk '{ print $14 + $15 }' "/proc/$pid/stat" 2> /dev/null || echo 0)
  awk -v now="$now" -v start="$start" -v ticks="$ticks" -v tick="$tick" -v step="$1" \
    'BEGIN { printf "%6.2f s  cpu %6.2f s  %s\n", now - start, ticks / tick, step }' |
    tee -a "$out/steps.txt"
}
shot() {
  grim -l 1 "$out/$1.png"
  mark "shot $1"
}
press() {
  wtype -s 300 -k "$1"
  mark "key $1"
}
windows() {
  swaymsg -t get_tree -r | tr -d ' \n' | grep -o '"app_id":"cinq"' | wc -l
}

(cd "$show" && exec "$cinq" riscv.cinq.toml) > "$out/cinq.log" 2>&1 &
pid=$!
for _ in $(seq 240); do
  [ "$(windows)" -ge 2 ] && break
  kill -0 "$pid" || break
  sleep 0.5
done
mark "$(windows) windows of cinq"
kill -0 "$pid" || { cat "$out/cinq.log"; exit 1; }
sleep 3
shot opened
press space # Chord
sleep 4
shot chord
press space # Fade of the chord, and then the video
sleep 1
shot fading
for i in 1 2 3 4 5 6; do
  sleep 1.5
  shot "video-$i"
done
sleep 3
shot ended
grep -o '/[^ ]*\.so[.0-9]*' "/proc/$pid/maps" | sort -u > "$out/libraries.txt" || true
swaymsg -t get_tree > "$out/tree.txt"
kill "$pid"
wait "$pid" || true
pid=''
echo "--- steps"
cat "$out/steps.txt"
echo "--- cinq"
cat "$out/cinq.log"
[ -n "$sound" ] || exit 0
kill -INT "$recorder"
wait "$recorder" || true
recorder=''

# Level and zero crossings of the left channel each half second, from which
# the pitch of a lone tone follows
ffmpeg -hide_banner -loglevel error -i "$out/sound.wav" -af "asetnsamples=n=24000,astats=metadata=1:reset=1:measure_perchannel=RMS_level+Zero_crossings_rate:measure_overall=none,ametadata=mode=print:file=$work/levels" -f null -
awk -F '[:=]' '
  /pts_time/ { time = $NF }
  /\.1\.RMS_level/ { level = $NF }
  /\.1\.Zero_crossings_rate/ { printf "%6.2f s  %7.1f dB  %4.0f Hz\n", time, level, $NF * 24000 }
' "$work/levels" > "$out/levels.txt"
echo "--- sound"
cat "$out/levels.txt"
