#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays a tone of 1 kHz from a file of 48 kHz on jack:output, the device of
# jackd with its dummy backend, under the quirks of JACK servers, and
# records the playback ports with jack_capture, each take as NAME.wav in
# OUT with a screenshot of cinq as NAME.png:
#
# - rate-44100 and rate-96000: servers at other rates than the file;
# - period-64, period-4096, and period-1000: tiny, large, and uneven
#   periods;
# - late: a server that starts only after cinq has fired the cue;
# - restart: a server that stops midway through the cue and starts again;
# - ports-2: a server of two playback ports under outputs on four
#   channels.
#
# Usage: jack.sh CINQ OUT, with jackd, jack_lsp, jack_capture, sway, wtype,
# grim, and ffmpeg on PATH
set -euo pipefail
cinq=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)
sway='' pid='' server=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'pcm.!default { type null }\n' > "$HOME/.asoundrc"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$server" ] && kill "$server" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
}
trap cleanup EXIT

ffmpeg -loglevel error -y -f lavfi -i "sine=frequency=1000:sample_rate=48000:duration=60" \
  -af volume=0.5 -ac 2 -c:a pcm_f32le "$work/tone.wav"
# Show of an output on channels 1-2, and with $1 set, one on 3-4 at -6 dB
show() {
  printf '[[output]]\nname = "Main"\ndevice = "jack:output"\nchannels = "1-2"\n\n'
  [ -n "${1:-}" ] && printf '[[output]]\nname = "Far"\ndevice = "jack:output"\nchannels = "3-4"\n\n'
  printf '[[list]]\nname = "Main"\n\n[[list.cue]]\nid = 1\nnumber = "1"\nkind = "audio"\nname = "Tone"\nfile = "tone.wav"\n\n'
  printf '[list.cue.sends]\nMain = 0.0\n'
  [ -n "${1:-}" ] && printf 'Far = -6.0\n'
  true
}
show > "$work/tone.cinq.toml"
show far > "$work/far.cinq.toml"

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

# Starts jackd at rate $2 with periods of $3 frames and $4 playback ports,
# logging under the name $1
start_server() {
  jackd --no-realtime -d dummy -r "$2" -p "$3" -P "$4" -C 0 >> "$out/jackd-$1.log" 2>&1 &
  server=$!
  for _ in $(seq 50); do
    jack_lsp > /dev/null 2>&1 && return
    sleep 0.2
  done
  echo "jackd did not start for $1" >> "$out/jackd-$1.log"
}
stop_server() {
  [ -n "$server" ] && kill "$server" 2> /dev/null || true
  [ -n "$server" ] && wait "$server" 2> /dev/null || true
  server=''
  sleep 1
}
launch() {
  (cd "$work" && exec "$cinq" "$2.cinq.toml" > "$out/cinq-$1.log" 2>&1) &
  pid=$!
  sleep 5
}
quit() {
  wtype -s 300 -k Escape
  sleep 1
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
}
record() {
  jack_capture --daemon --recording-time 4 --channels "$2" --port 'system:playback*' "$out/$1.wav" \
    > "$out/capture-$1.log" 2>&1 || true
}
# Plays the take $1 on a server of rate $2, periods $3, and ports $4
take() {
  start_server "$1" "$2" "$3" "$4"
  launch "$1" "${5:-tone}"
  wtype -s 300 -k space
  sleep 2
  record "$1" "$4" &
  local recording=$!
  sleep 1
  grim "$out/$1.png" || true
  wait "$recording" || true
  jack_lsp -c > "$out/connections-$1.txt" 2>&1 || true
  quit
  stop_server
}

take rate-44100 44100 256 2
take rate-96000 96000 256 2
take period-64 48000 64 2
take period-4096 48000 4096 2
take period-1000 48000 1000 2
take ports-2 48000 256 2 far

# Server that starts only once the cue plays
launch late tone
wtype -s 300 -k space
sleep 4
grim "$out/late-before.png" || true
start_server late 48000 256 2
sleep 4
record late 2
grim "$out/late.png" || true
quit
stop_server

# Server that stops midway through the cue and starts again
start_server restart 48000 256 2
launch restart tone
wtype -s 300 -k space
sleep 3
stop_server
sleep 4
grim "$out/restart-gone.png" || true
start_server restart 48000 256 2
sleep 4
record restart 2
grim "$out/restart.png" || true
quit
stop_server
