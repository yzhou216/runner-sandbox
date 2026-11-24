#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays cinq through the eight audio checks in a headless sway session of its
# own, with a PulseAudio server of its own that has two null sinks, house and
# booth, which ~/.asoundrc names as the ALSA devices alsa:house and alsa:booth.
# The monitor of each sink is recorded for the whole session.  marks.tsv in OUT
# holds the wall clock time of each step (seconds since the epoch), which
# analyze.sh lines up with the recordings by a calibration beep on each sink.
# Every key goes through wtype, GO is F9 and Panic is F10, and a mark follows
# the call of wtype that sends the key.
#
# Usage: record.sh CINQ OUT, with sway, swaymsg, wtype, grim, pulseaudio,
# pactl, paplay, parecord, and ffmpeg on PATH, ALSA_PLUGIN_DIR naming the
# directory of the pulse plugin of ALSA, and CINQ the program
set -euo pipefail

here=$(dirname "$(readlink -f "$0")")
cinq=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)
show=$work/show
pid='' sway='' recorders=()

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export XDG_STATE_HOME=$work/home/.state
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
unset XDG_DATA_HOME XDG_CACHE_HOME
mkdir -p "$HOME" "$XDG_CONFIG_HOME/cinq"
mkdir -m 700 "$XDG_RUNTIME_DIR"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ "${#recorders[@]}" -gt 0 ] && kill -INT "${recorders[@]}" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  PULSE_RUNTIME_PATH=$work/run-booth pulseaudio --kill 2> /dev/null || true
  pulseaudio --kill 2> /dev/null || true
}
trap cleanup EXIT

: > "$out/marks.tsv"
mark() { printf '%s\t%s\n' "$(date +%s.%N)" "$1" >> "$out/marks.tsv"; }
shot() { grim -o HEADLESS-1 "$out/$1.png"; }
# One wtype call for each key press, which waits before typing, since sway
# drops keys sent right after wtype uploads its keymap
key() { wtype -s 300 "$@"; }
ctrl() { wtype -s 300 -M ctrl -k "$1" -m ctrl; }
# Gives the keyboard back to the cue list, which Control+c does from a field
leave() { ctrl c; ctrl c; }
# Presses a key and marks the moment that wtype returns
press() { key -k "$1"; mark "$2"; }

pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
pactl load-module module-null-sink sink_name=house rate=48000 channels=2 > /dev/null
booth_server=unix:$work/booth.sock
declare -A server=([booth]=$booth_server)
mkdir -m 700 "$work/run-booth"
# Booth lives on a PulseAudio server of its own, so that killing the server takes the device away (unloading
# a sink makes PulseAudio move the streams to the default sink, which is no fault)
PULSE_RUNTIME_PATH=$work/run-booth pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse-booth.log" -L "module-native-protocol-unix socket=$work/booth.sock"
for _ in $(seq 100); do
  pactl -s "$booth_server" info > /dev/null 2>&1 && break
  sleep 0.1
done
pactl -s "$booth_server" load-module module-null-sink sink_name=booth rate=48000 channels=2 > /dev/null
pactl set-default-sink house
cat > "$HOME/.asoundrc" << RC
pcm.!default { type pulse }
ctl.!default { type pulse }
pcm.house {
  type pulse
  device house
  hint { show on description "House speakers" }
}
pcm.booth {
  type pulse
  server "$booth_server"
  device booth
  hint { show on description "Booth monitors" }
}
RC
printf '[keys]\ngo = ["Space", "F9"]\npanic = ["Escape", "F10"]\n' > "$XDG_CONFIG_HOME/cinq/config.toml"
bash "$here/make-show.sh" "$show"
cp -r "$show" "$out/show"
ffmpeg -nostdin -hide_banner -loglevel error -y -f lavfi -i "sine=frequency=1000:duration=0.4" "$work/beep.wav"
"$cinq" --audio-devices > "$out/devices.tsv" 2>&1 || true

cat > "$work/sway.config" << SWAY
output HEADLESS-1 mode 1280x800 position 0 0 bg #000000 solid_color
default_border none
default_floating_border none
focus_on_window_activation none
SWAY
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
for _ in $(seq 300); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK
# Sway's sockets appear before it answers on them
until swaymsg -t get_version > /dev/null 2>&1; do sleep 0.5; done
# The stage window goes to a second output, so that the main window has the first alone
swaymsg -q create_output
swaymsg -q output HEADLESS-2 mode 1280x800 position 1280 0
swaymsg -q 'for_window [title="stage - cinq$"] move container to output HEADLESS-2'
sleep 1

for sink in house booth; do
  parecord ${server[$sink]:+-s "${server[$sink]}"} -d "$sink.monitor" --rate=48000 --channels=2 --file-format=wav "$out/$sink.wav" &
  recorders+=("$!")
done
sleep 2
for sink in house booth; do
  mark "calibrate-$sink"
  paplay ${server[$sink]:+-s "${server[$sink]}"} -d "$sink" "$work/beep.wav"
  sleep 1
done

# Starts cinq on a show file and puts the keyboard on its main window
launch() {
  (cd "$show" && exec "$cinq" "$1" >> "$out/cinq.log" 2>&1) &
  pid=$!
  sleep 5
  swaymsg -q "[title=\"^$1 - cinq\$\"] focus"
  sleep 1
}
quit() {
  kill "$pid" 2> /dev/null || true
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 2
}
# Selects cue $1 of the list, puts it on standby, and fires it with GO
fire() {
  key gg
  if [ "$1" -gt 1 ]; then key "$(printf '%*s' $(($1 - 1)) '' | tr ' ' j)"; fi
  key -k Return
  press F9 "${2:-go-$1}"
}
# Goes from the cue list to the Levels tab field $1 of the selected cue, 1 being
# the level and 2 and 3 the sends to House and Booth, and types $2 into it
type_into() {
  for _ in $(seq "$1"); do key -k Tab; done
  ctrl a
  if [ -n "$2" ]; then key -- "$2"; else key -k BackSpace; fi
}

# Check 1: an audio cue at -6 dB on the default device
launch plain.cinq.toml
shot 1-start
fire 1 go-plain
sleep 6
shot 1-playing
mark end-plain
quit

launch outputs.cinq.toml
shot 2-start

# Check 2: a cue that sends to both outputs at different levels
fire 1 go-1b
sleep 6
shot 2-playing
mark end-2

# Check 6: a level typed into the Levels tab while the cue plays
key gg
key gtgt
shot 6-before
type_into 1 -12
press Tab typed-level
sleep 4
shot 6-after
mark end-6
leave

# Check 3: a fade cue to -20 dB over 3 s
fire 2
sleep 6
shot 3-after
mark end-3

# Check 5, Stop: a stop cue, on cue 1 that still plays
fire 3
sleep 3
shot 5-stop
mark end-5-stop

# Check 5, Panic: F10 over a cue that plays
fire 6
sleep 4
press F10 panic
sleep 3
shot 5-panic
mark end-5-panic

# Check 4: a looping cue with a loop section, then a devamp cue
fire 4
sleep 6
shot 4-looping
fire 5
sleep 8
shot 4-after
mark end-4

# Check 8: clearing the last send of a cue that plays
fire 8
sleep 4
shot 8-before
type_into 2 ''
press Tab cleared-house
ctrl a
key -k BackSpace
press Tab cleared-sends
sleep 1
shot 8-after
sleep 3
mark end-8
leave
press F10 panic-8
sleep 3

# Check 7: a device that goes away while a cue plays on both outputs
fire 7
sleep 4
shot 7-before
PULSE_RUNTIME_PATH=$work/run-booth pulseaudio --kill
mark unload-booth
sleep 3
shot 7-after-3s
pactl list short sink-inputs > "$out/inputs-after-3s.txt"
sleep 4
shot 7-after-7s
fire 9
sleep 3
shot 7-booth-only
mark end-7
press F10 panic-7
sleep 3
mark end
pactl list sinks short > "$out/sinks.txt"
quit
kill -INT "${recorders[@]}" 2> /dev/null || true
wait "${recorders[@]}" 2> /dev/null || true
recorders=()
