#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs the three demo shows of cinq in a headless sway session of its own,
# as an operator would with the vi keys, takes screenshots along the way,
# and records what each show plays from the monitor of a PulseAudio null
# sink, then fails if a recording is silent.
#
# Usage: run.sh CINQ DEMO OUT, with sway, swaymsg, grim, wtype, pulseaudio,
# pactl, parec, and ffmpeg on PATH, ALSA_PLUGIN_DIR naming the directory of
# the pulse plugin of ALSA, CINQ the program, DEMO what nix build .#demo
# built, and OUT the directory for the shots, recordings, and logs.
set -euo pipefail

cinq=$(readlink -f "$1")
demo=$(readlink -f "$2")
out=$(mkdir -p "$3" && readlink -f "$3")
work=$(mktemp -d)
show=$work/show
pid='' sway='' rec=''

# Copy of the demo, as the tutorial has readers make, since cinq keeps the
# files that it opens where it can write
cp -RL "$demo" "$show"
chmod -R u+w "$show"

export HOME=$work/home XDG_RUNTIME_DIR=$work/run
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
unset XDG_CONFIG_HOME XDG_DATA_HOME XDG_STATE_HOME XDG_CACHE_HOME
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
cleanup() {
  [ -n "$rec" ] && kill "$rec" 2> /dev/null || true
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$sway" ] && { swaymsg exit > /dev/null 2>&1 || kill "$sway" 2> /dev/null || true; }
  pulseaudio --kill 2> /dev/null || true
}
trap cleanup EXIT

pulseaudio -n --daemonize=yes --exit-idle-time=-1 --realtime=no --high-priority=no \
  --log-target=file:"$out/pulse.log" -L module-native-protocol-unix \
  -L 'module-null-sink sink_name=out rate=48000 channels=2'
for _ in $(seq 100); do
  pactl info > /dev/null 2>&1 && break
  sleep 0.1
done
printf 'pcm.!default { type pulse }\nctl.!default { type pulse }\n' > "$HOME/.asoundrc"

cat > "$work/sway.config" << EOF
output HEADLESS-1 mode 2560x1920 position 0 0 scale 2
output HEADLESS-2 mode 2560x1920 position 1280 0 scale 2
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

# One key sequence per wtype call, since wtype joins its arguments with
# spaces, and a space is GO
keys() { wtype -s 300 "$@"; }
go() { keys -k space; }
main() { swaymsg -q "[title=\"^$1 - cinq\$\"] focus"; }
shot() { grim -o "${2:-HEADLESS-1}" "$out/$1.png"; }
launch() {
  (cd "$show" && exec "$cinq" "$1" >> "$out/cinq-$1.log" 2>&1) &
  pid=$!
  sleep 5
  main "$1"
  sleep 1
}
quit() {
  kill "$pid"
  wait "$pid" 2> /dev/null || true
  pid=''
  sleep 1
}
record() {
  parec --device=out.monitor --file-format=wav "$out/$1.wav" &
  rec=$!
}
stop() {
  kill "$rec"
  wait "$rec" 2> /dev/null || true
  rec=''
}

# Act 1 from the preshow music through the storm, Act 2 on the stage, and
# the effects
launch storm.cinq.toml
shot storm-open
record storm
go
sleep 3
shot storm-preshow
go
sleep 7
shot storm-sequence
go
sleep 2
go
sleep 4
go
sleep 3
shot storm-rain-out
keys -k Escape
sleep 2
keys l
sleep 1
go
sleep 3
shot storm-title HEADLESS-2
main storm.cinq.toml
go
sleep 5
shot storm-film HEADLESS-2
shot storm-act2
main storm.cinq.toml
keys -k Escape
sleep 2
keys l
sleep 1
go
sleep 4
go
sleep 2
shot storm-effects
stop
quit

# Knocks, a wait, and the bell from one GO, then rain that a stop cue ends
launch chains.cinq.toml
shot chains-open
record chains
go
sleep 9
shot chains-bell
go
sleep 2
shot chains-rain
sleep 4
shot chains-stopped
stop
quit

# Main list and effects list, each with a standby of its own
launch lists.cinq.toml
shot lists-open
record lists
go
sleep 3
keys l
sleep 1
go
sleep 3
shot lists-effects
keys h
sleep 1
go
sleep 5
shot lists-main
stop
quit

# Level of each recording, which the null sink leaves at silence unless
# cinq played into it
silent=''
for show in storm chains lists; do
  level=$(ffmpeg -nostdin -hide_banner -i "$out/$show.wav" -af volumedetect -f null - 2>&1 |
    sed -n 's/.*\(mean_volume\|max_volume\): \(.*\) dB/\1 \2/p' | tr '\n' ' ')
  echo "$show: $level" | tee -a "$out/levels.txt"
  max=$(echo "$level" | sed -n 's/.*max_volume \([^ ]*\).*/\1/p')
  [ -z "$max" ] || awk -v max="$max" 'BEGIN { exit !(max < -50) }' && silent="$silent $show"
done
if [ -n "$silent" ]; then
  echo "silent:$silent" >&2
  exit 1
fi
