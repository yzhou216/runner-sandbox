#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records cinq routing one cue to the outputs of desk.cinq.toml, which share
# one device of eight channels, from its main window alone: the outputs view
# in place of the cue list, the sends of the cue on its Levels tab, and the
# channels of an output changed as the cue plays.  A PulseAudio server of
# its own has one null sink of eight channels, desk, which ALSA names as the
# device alsa:desk, and the monitor of the sink is recorded with all eight
# channels, so that the recording shows which channels each step reached.
#
# Usage: demo.sh CINQ SHOW OUT, with sway, wtype, wf-recorder, grim,
# pulseaudio, pactl, and parecord on PATH, ALSA_PLUGIN_DIR naming the
# directory of the pulse plugin of ALSA, CINQ the program, SHOW the
# directory of desk.cinq.toml and its sound, and OUT the directory for the
# recordings.  marks.tsv in OUT holds the time of each step, in seconds
# since the epoch, with the captions that compose.sh burns in, and a
# screenshot of each step goes to OUT as well.
set -euo pipefail

cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
size=${SIZE:-1280x900}
work=$(mktemp -d)
pid='' sway='' recorders=()

export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
unset XDG_DATA_HOME XDG_STATE_HOME XDG_CACHE_HOME
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
cleanup() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ "${#recorders[@]}" -gt 0 ] && kill -INT "${recorders[@]}" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  pulseaudio --kill 2> /dev/null || true
}
trap cleanup EXIT

mark() { printf '%s\t%s\n' "$(date +%s.%N)" "$*" >> "$out/marks.tsv"; }
caption() { mark "caption	$*"; }
shot() { grim "$out/$1.png"; }

# Channel map of ALSA for eight channels, so that PulseAudio passes the
# channels of the stream to those of the sink as they are
map=front-left,front-right,rear-left,rear-right,front-center,lfe,side-left,side-right
pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix \
  -L "module-null-sink sink_name=desk rate=48000 channels=8 channel_map=$map sink_properties=device.description=Desk"
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
cat > "$HOME/.asoundrc" << 'EOF'
pcm.!default { type pulse }
ctl.!default { type pulse }
pcm.desk {
  type pulse
  device desk
  hint { show on description "Desk" }
}
EOF
"$cinq" --audio-devices > "$out/devices.tsv"

cat > "$work/sway.config" << EOF
output HEADLESS-1 mode $size position 0 0
default_border none
EOF
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK
sleep 1

# Keys go one wtype call each, which waits before typing, since sway drops
# keys sent right after wtype uploads its keymap
keys() { wtype -s 300 "$@"; }
times() { for _ in $(seq "$1"); do keys "${@:2}"; done; }

wf-recorder -D -r 30 -o HEADLESS-1 -f "$out/screen.mkv" > "$out/wf-recorder.log" 2>&1 &
recorders+=("$!")
mark "record screen"
parecord -d desk.monitor --rate=48000 --channels=8 --channel-map=$map \
  --file-format=wav "$out/desk.wav" &
recorders+=("$!")
mark "record desk"
sleep 1

caption "cinq opens a show whose four outputs share Desk, one device of eight channels."
(cd "$show" && exec "$cinq" desk.cinq.toml >> "$out/cinq.log" 2>&1) &
pid=$!
sleep 5
shot opened
caption "l steps past the cue list to the outputs, which take its place: House on 1-2, Sub on 3, Monitors on 5-6, Lobby on 7-8."
keys l
sleep 5
shot outputs
caption "j selects Sub, which the inspector edits in place of a cue."
keys j
sleep 4
shot sub
caption "h goes back to the cue list, and GO plays Music, which sends to House alone, as new cues do."
keys h
sleep 1.5
keys -k space
mark house
sleep 5
shot house
caption "g t twice shows the Levels tab of the cue, with a field for each output."
keys gt
sleep 0.5
keys gt
sleep 3
shot levels
caption "Tab moves to the field of Sub, and 0 sends the music there at 0 dB, on channel 3, as it plays."
times 5 -k Tab
sleep 1.5
keys 0
mark sub
sleep 5
shot sends-sub
caption "Tab moves to Monitors, and -6 sends the music there 6 dB lower, on channels 5 and 6."
keys -k Tab
sleep 1.5
keys -k minus
keys 6
mark monitors
sleep 5
shot sends-monitors
caption "Shift+Tab twice moves back to House, and emptying its field takes the music off channels 1 and 2."
times 2 -M shift -k Tab -m shift
sleep 1.5
times 4 -k BackSpace
mark house-off
sleep 5
shot sends-house-off
caption "Enter leaves the field, and l shows the outputs again, with Sub still selected."
keys -k Return
sleep 0.5
keys l
sleep 4
shot outputs-again
caption "Tab moves to the channels of Sub, and 4 moves Sub, with the music, from channel 3 to channel 4."
times 4 -k Tab
sleep 1.5
keys -k BackSpace
keys 4
mark sub-4
sleep 5
shot sub-4
keys -k Return
caption "Escape is Panic, which fades the music out, and Control+s saves the show with its new sends and channels."
keys -k Escape
mark panic
sleep 2
keys -M ctrl s -m ctrl
sleep 3
shot panic
mark end
kill -INT "${recorders[@]}"
wait "${recorders[@]}" 2> /dev/null || true
recorders=()
cp "$show/desk.cinq.toml" "$out/saved.cinq.toml"
