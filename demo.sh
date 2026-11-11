#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records cinq choosing its audio device from the list in its sidebar, in a
# headless sway session of its own with two outputs, the main window on the
# first and the stage window on the second.  A PulseAudio server of its own
# has two null sinks, house and booth, which ALSA names as the devices
# alsa:house and alsa:booth, and the monitor of each sink is recorded
# apart, so that the recording shows which device each sound played on.
#
# Usage: demo.sh CINQ SHOW OUT, with sway, swaymsg, wtype, wf-recorder,
# pulseaudio, pactl, and parecord on PATH, ALSA_PLUGIN_DIR naming the
# directory of the pulse plugin of ALSA, CINQ the program, SHOW the
# directory of devices.cinq.toml and its files, and OUT the directory for
# the recordings.  marks.tsv in OUT holds the time of each step, in seconds
# since the epoch, with the captions that compose.sh burns in.
set -euo pipefail

cinq=$(readlink -f "$1")
show=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
size=${SIZE:-1280x960}
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

pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix \
  -L 'module-null-sink sink_name=house rate=48000 channels=2 sink_properties=device.description=House' \
  -L 'module-null-sink sink_name=booth rate=48000 channels=2 sink_properties=device.description=Booth'
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
pactl set-default-sink house
cat > "$HOME/.asoundrc" << 'EOF'
pcm.!default { type pulse }
ctl.!default { type pulse }
pcm.house {
  type pulse
  device house
  hint { show on description "House speakers" }
}
pcm.booth {
  type pulse
  device booth
  hint { show on description "Booth monitors" }
}
EOF
"$cinq" --audio-devices > "$out/devices.tsv"
row() { grep -n -F "$1	" "$out/devices.tsv" | cut -d: -f1; }
house=$(row alsa:house) booth=$(row alsa:booth)

width=${size%x*}
cat > "$work/sway.config" << EOF
output HEADLESS-1 mode $size position 0 0
output HEADLESS-2 mode $size position $width 0
default_border none
default_floating_border none
focus_on_window_activation none
for_window [title=" stage - cinq\$"] move container to output HEADLESS-2
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
swaymsg -q create_output
sleep 1

# Keys go one wtype call each, which waits before typing, since sway drops
# keys sent right after wtype uploads its keymap
keys() { wtype -s 300 "$@"; }
times() { printf "%${2}s" '' | tr ' ' "$1"; }
state() {
  cp "$XDG_CONFIG_HOME/cinq/config.toml" "$out/config-$1.toml" 2> /dev/null || true
  pactl list sink-inputs > "$out/inputs-$1.txt"
}

for output in HEADLESS-1 HEADLESS-2; do
  wf-recorder -D -r 30 -o "$output" -f "$out/$output.mkv" > "$out/$output.log" 2>&1 &
  recorders+=("$!")
  mark "record $output"
done
for sink in house booth; do
  parecord -d "$sink.monitor" --rate=48000 --channels=2 --file-format=wav "$out/$sink.wav" &
  recorders+=("$!")
  mark "record $sink"
done
sleep 1

caption "cinq opens a show of two cues.  Sound plays on the default device, House speakers, heard on the left."
(cd "$show" && exec "$cinq" devices.cinq.toml >> "$out/cinq.log" 2>&1) &
pid=$!
sleep 5
swaymsg -q '[title="^devices.cinq.toml - cinq$"] focus'
sleep 1
caption "GO plays the Music cue on House speakers."
keys -k space
mark go-music
sleep 5
state default

caption "Tab twice moves to the Audio device list in the sidebar, and Enter opens it."
keys -k Tab
keys -k Tab
sleep 1
keys -k Return
sleep 3
caption "j moves to Booth monitors, and Enter chooses it.  The music goes on there, heard on the right, once the device starts."
keys "$(times j "$booth")"
sleep 1.5
keys -k Return
mark chose-booth
sleep 5
state booth

caption "GO plays the Film cue.  Its sound plays on Booth monitors as well."
keys -k space
mark go-film
sleep 7

caption "The list chooses House speakers again, and both sounds move back."
keys -k Tab
keys -k Tab
sleep 1
keys -k Return
sleep 2
if [ "$house" -lt "$booth" ]; then
  keys "$(times k $((booth - house)))"
else
  keys "$(times j $((house - booth)))"
fi
sleep 1.5
keys -k Return
mark chose-house
sleep 5
state house

caption "config.toml now holds: $(grep audio_device "$out/config-house.toml" || echo nothing)"
sleep 5
caption "Escape is Panic, which fades everything out."
keys -k Escape
sleep 4
mark end
kill -INT "${recorders[@]}"
wait "${recorders[@]}" 2> /dev/null || true
recorders=()
