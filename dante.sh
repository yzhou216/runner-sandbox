#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Plays through an emulated Dante network, in which Inferno stands in for
# the Dante Virtual Soundcard on the computer that runs cinq and for the
# Dante card of the sound board.  The ALSA device dante converts what cinq
# plays for inferno_cinq, the Dante device "cinq" with eight transmit
# channels, and the board is the Dante device "board" with eight receive
# channels, which arecord records for the whole run.  netaudio, an open
# controller in the manner of Dante Controller, subscribes each receive
# channel of the board to the transmit channel of cinq with its number.
#
# Phases, each marked in marks.tsv:
#
# 1. aplay plays a tone on each channel, at a level 3 dB lower than the one
#    before, so that the level of each channel of the board tells which
#    channel of cinq it came from.
# 2. cinq plays desk.cinq.toml, whose outputs fill all eight channels, and
#    its cue is routed from the main window, as demo.sh of the session
#    cinq/2026-11-12-audio-outputs does.
# 3. cinq plays partial.cinq.toml, whose outputs need three channels alone.
#
# Usage: dante.sh CINQ SHOW OUT INFERNO NETAUDIO, with DANTE_IP the address
# that the Dante devices bind to, sway, wtype, grim, arecord, aplay, and
# ffmpeg on PATH, CINQ the program, SHOW the directory of the shows and
# their sound, OUT the directory for the recordings, INFERNO the directory
# of inferno.sh, and NETAUDIO the netaudio program.
set -euo pipefail

cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
inferno=$(readlink -f "$4")
netaudio=$(readlink -f "$5")
ip=${DANTE_IP:?}
work=$(mktemp -d)
pids=() sway='' cinq_pid='' board=''

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
# Inferno logs at the debug level by default
export RUST_LOG=${RUST_LOG:-info}
export ALSA_PLUGIN_DIR=$inferno
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
cleanup() {
  [ -n "$cinq_pid" ] && kill "$cinq_pid" 2> /dev/null || true
  [ -n "$board" ] && kill -INT "$board" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  [ "${#pids[@]}" -gt 0 ] && kill "${pids[@]}" 2> /dev/null || true
}
trap cleanup EXIT

mark() { printf '%s\t%s\n' "$(date +%s.%N)" "$*" >> "$out/marks.tsv"; }
log() { printf '%s %s\n' "$(date +%T.%N | cut -c1-12)" "$*"; }
shot() { grim "$out/$1.png" || true; }

clock=$work/usrvclock
USRVCLOCK_SOCKET=$clock "$inferno/fake_usrvclock_server" > "$out/clock.log" 2>&1 &
pids+=("$!")
for _ in $(seq 50); do
  [ -S "$clock" ] && break
  sleep 0.1
done

# Same settings for every program that opens the devices, since the device
# goes by the same name and ID each time cinq starts, as the Dante Virtual
# Soundcard of one computer does
cat > "$HOME/.asoundrc" << EOF
pcm.inferno_cinq {
  type inferno
  NAME "cinq"
  BIND_IP "$ip"
  SAMPLE_RATE "48000"
  PROCESS_ID "1"
  ALT_PORT "10100"
  RX_CHANNELS "0"
  TX_CHANNELS "8"
  CLOCK_PATH "$clock"
}
pcm.dante {
  type plug
  slave.pcm "inferno_cinq"
  hint {
    show on
    description "Dante"
  }
}
pcm.board {
  type inferno
  NAME "board"
  BIND_IP "$ip"
  SAMPLE_RATE "48000"
  PROCESS_ID "2"
  ALT_PORT "10200"
  RX_CHANNELS "8"
  TX_CHANNELS "0"
  CLOCK_PATH "$clock"
}
EOF

"$cinq" --audio-devices > "$out/devices.tsv" 2> "$out/devices.log" || true
log "devices:"
cat "$out/devices.tsv"

# Board records from its start to the end of the run
arecord -D board -c 8 -r 48000 -f S32_LE -t wav "$out/board.wav" > "$out/board.log" 2>&1 &
board=$!
mark "record board"

# Waits for both devices to show on the network, then subscribes each
# receive channel of the board to the transmit channel of cinq with its
# number.  Returns whether the subscriptions were made.
subscribe() {
  local listed=''
  for _ in $(seq 60); do
    listed=$("$netaudio" device list 2> /dev/null || true)
    if grep -q board <<< "$listed" && grep -q cinq <<< "$listed"; then
      break
    fi
    sleep 1
  done
  log "netaudio device list:"
  printf '%s\n' "$listed"
  grep -q cinq <<< "$listed" || return 1
  # Syntax of netaudio 0.5, then that of the releases that Inferno's own
  # tests were written for
  for channel in 1 2 3 4 5 6 7 8; do
    "$netaudio" subscription add --tx "tx:$channel@cinq" --rx "rx:$channel@board" ||
      "$netaudio" subscription add --rx-device-name board --rx-channel-name "RX $channel" \
        --tx-device-name cinq --tx-channel-name "TX $channel" ||
      log "subscription $channel failed"
  done
  sleep 3
  log "netaudio subscription list:"
  "$netaudio" subscription list || true
}

# Phase 1: tones from aplay
# Amplitudes in figures, since a comma would end the filter's option
tones=$(awk 'BEGIN { for (i = 0; i < 8; i++) printf "%s%.5f*sin(2*PI*440*t)", (i ? "|" : ""), 0.4 * 10 ^ (-i * 3 / 20) }')
ffmpeg -loglevel error -y -f lavfi -i "aevalsrc=$tones:s=48000:d=40" -c:a pcm_f32le "$work/tones.wav"
mark "tones"
aplay -D dante "$work/tones.wav" > "$out/aplay.log" 2>&1 &
aplay_pid=$!
subscribe > "$out/subscribe-1.log" 2>&1 || log "no subscriptions in phase 1"
cat "$out/subscribe-1.log"
mark "tones-heard"
sleep 8
mark "tones-end"
kill "$aplay_pid" 2> /dev/null || true
wait "$aplay_pid" 2> /dev/null || true
sleep 3

# Phase 2: cinq in a headless sway session
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

keys() { wtype -s 300 "$@"; }
times() { for _ in $(seq "$1"); do keys "${@:2}"; done; }
launch() {
  (cd "$show" && exec "$cinq" "$1" >> "$out/cinq-${1%%.*}.log" 2>&1) &
  cinq_pid=$!
  sleep 5
}
quit() {
  kill "$cinq_pid" 2> /dev/null || true
  wait "$cinq_pid" 2> /dev/null || true
  cinq_pid=''
}

launch desk.cinq.toml
shot desk-opened
# A board keeps its subscriptions by name, as Dante devices do, so they
# may come back by themselves once cinq's device shows again
sleep 5
"$netaudio" subscription list > "$out/subscriptions-2-before.log" 2>&1 || true
subscribe > "$out/subscribe-2.log" 2>&1 || log "no subscriptions in phase 2"
cat "$out/subscribe-2.log"
sleep 2
keys -k space
mark house
sleep 5
shot house
keys gt
sleep 0.5
keys gt
sleep 2
times 5 -k Tab
sleep 1
keys 0
mark sub
sleep 5
shot sub
keys -k Tab
sleep 1
keys -k minus
keys 6
mark monitors
sleep 5
shot monitors
times 2 -M shift -k Tab -m shift
sleep 1
times 4 -k BackSpace
mark house-off
sleep 5
shot house-off
keys -k Return
sleep 0.5
keys l
sleep 2
keys j
sleep 1
times 4 -k Tab
sleep 1
keys -k BackSpace
keys 4
mark sub-4
sleep 5
shot sub-4
keys -k Return
keys -k Escape
mark panic
sleep 4
shot panic
mark desk-end
quit
sleep 3

# Phase 3: a show that needs three channels of the eight
launch partial.cinq.toml
sleep 5
subscribe > "$out/subscribe-3.log" 2>&1 || log "no subscriptions in phase 3"
sleep 2
keys -k space
mark partial
sleep 6
shot partial
mark partial-end
keys -k Escape
sleep 3
quit
sleep 2
mark end
kill -INT "$board" 2> /dev/null || true
wait "$board" 2> /dev/null || true
board=''
