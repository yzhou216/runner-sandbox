#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Runs the eight checks of the brief on cinq in headless sway, on FreeBSD.
# Each phase starts cinq with a config directory of its own, writes its
# standard error to OUT/PHASE.err, and takes screenshots with grim.
#
# Usage: check.sh CINQ OUT SCALE, with sway, wtype, grim, and python3 on PATH;
# SCALE stretches every wait (1 natively, 4 when emulated)
set -uo pipefail

cinq=$1
out=$(mkdir -p "$2" && cd "$2" && pwd)
scale=${3:-1}
label=${HINT_LABEL:-}
work=$(mktemp -d)
nap() { sleep "$(awk -v s="$1" -v k="$scale" 'BEGIN { print s * k }')"; }
mark() { printf '%s %s\n' "$(date +%s)" "$*" | tee -a "$out/marks.txt"; }

export HOME=$work/home XDG_RUNTIME_DIR=$work/run
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS
mkdir -p "$HOME"
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'output HEADLESS-1 resolution 1280x800\n' > "$work/sway.config"
{ uname -a; "$cinq" --version; } > "$out/versions.txt" 2>&1

sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
sway=$!
socket='' ipc=''
for _ in $(seq 300); do
  for path in "$XDG_RUNTIME_DIR"/wayland-[0-9]; do [ -S "$path" ] && socket=${path##*/}; done
  for path in "$XDG_RUNTIME_DIR"/sway-ipc.*.sock; do [ -S "$path" ] && ipc=$path; done
  [ -n "$socket" ] && [ -n "$ipc" ] && break
  sleep 0.2
done
export WAYLAND_DISPLAY=$socket SWAYSOCK=$ipc
until swaymsg -t get_version > /dev/null 2>&1; do sleep 0.5; done
mark "sway up"

# A show of three kinds of cue, with sounds that python makes
shows=$work/shows
mkdir -p "$shows/sounds"
python3 - "$shows/sounds" <<'PY'
import math, struct, sys, wave
for name, hz in (("alpha", 440), ("beta", 330)):
    with wave.open(f"{sys.argv[1]}/{name}.wav", "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(44100)
        w.writeframes(b"".join(struct.pack("<h", int(8000 * math.sin(2 * math.pi * hz * i / 44100))) for i in range(44100 * 4)))
PY
cat > "$shows/main.cinq.toml" <<'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Alpha"
file = "sounds/alpha.wav"

[[list.cue]]
id = 2
number = "2"
kind = "audio"
name = "Beta"
file = "sounds/beta.wav"
TOML
cat > "$shows/net.cinq.toml" <<'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "network"
name = "Closed port"
destination = "127.0.0.1:9"
message = "/test"
TOML

pid=''
shot() { grim "$out/$1.png" || true; }
focus() { swaymsg -q '[title=" - cinq$"] focus' || true; nap 0.5; }
key() { focus; wtype -s 300 -d 100 "$@"; nap 1.5; }
# start PHASE SHOW [CONFIG TEXT]: a fresh copy of the show, the config if given
start() {
  local dir=$work/$1
  mkdir -p "$dir/cfg"
  cp -R "$shows/." "$dir/"
  [ -n "${3:-}" ] && printf '%s\n' "$3" > "$dir/cfg/config.toml"
  mark "start $1"
  "$cinq" --config-dir "$dir/cfg" "$dir/$2" 2> "$out/$1.err" &
  pid=$!
  nap 10
  shot "$1-0-start"
}
stop() {
  mark "stop $1"
  swaymsg -q '[title=" - cinq$"] kill' || true
  wait "$pid" 2> /dev/null || true
  [ -f "$work/$1/cfg/config.toml" ] && cp "$work/$1/cfg/config.toml" "$out/$1.config.toml"
  nap 1
}
osc() {
  python3 - "$1" "$2" <<'PY'
import socket, sys
def pad(b): return b + b"\0" * (4 - len(b) % 4)
msg = pad(sys.argv[2].encode()) + pad(b",")
socket.socket(socket.AF_INET, socket.SOCK_DGRAM).sendto(msg, ("127.0.0.1", int(sys.argv[1])))
PY
}
go='go = ["Space", "F9"]'

# 1. Silent start, no config.toml
start c1 main.cinq.toml
stop c1

# 2. The retired keymap
start c2 main.cinq.toml 'keymap = "cua"'
key j
shot c2-1-j
key o
shot c2-2-o
stop c2

# 3. Hint mode
start c3 main.cinq.toml "[keys]
$go"
key f
shot c3-1-hints
key -k BackSpace
shot c3-2-backspace
if [ -n "$label" ]; then
  key f
  key "$label"
  shot c3-3-typed
fi
stop c3

# 4. OSC listener
start c4 main.cinq.toml 'osc_listen = "127.0.0.1:53000"
[keys]
go = ["Space", "F9"]'
osc 53000 /go
nap 3
shot c4-1-go
osc 53000 /cue/99/start
nap 3
shot c4-2-cue99
stop c4
python3 -c '
import socket, time
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(("127.0.0.1", 53001))
time.sleep(60)' &
holder=$!
nap 1
start c4b main.cinq.toml 'osc_listen = "127.0.0.1:53001"'
nap 3
shot c4b-1-taken
stop c4b
kill "$holder" 2> /dev/null || true

# 5. File watching
start c5 main.cinq.toml "[keys]
$go"
mv "$work/c5/sounds/alpha.wav" "$work/c5/sounds/alpha.gone"
nap 5
shot c5-1-deleted
mv "$work/c5/sounds/alpha.gone" "$work/c5/sounds/alpha.wav"
nap 5
shot c5-2-restored
stop c5

# 6. Paste
start c6 main.cinq.toml "[keys]
$go"
key gg
key yy
key p
nap 6
shot c6-1-pasted
stop c6

# 7. Errors in words
mkdir -p "$work/c7dir"
"$cinq" --config-dir "$work/c7cfg" "$work/c7dir" 2> "$out/c7b.err" &
pid=$!
nap 10
shot c7b-opening-a-directory
swaymsg -q '[title=" - cinq$"] kill' || true
wait "$pid" 2> /dev/null || true
mkdir -p "$work/c7/ro" "$work/c7/cfg"
cp "$shows/main.cinq.toml" "$work/c7/ro/show.cinq.toml"
cp -R "$shows/sounds" "$work/c7/ro/"
chflags schg "$work/c7/ro" 2> "$out/c7-chflags.txt" || true
mark "start c7c"
"$cinq" --config-dir "$work/c7/cfg" "$work/c7/ro/show.cinq.toml" 2> "$out/c7c.err" &
pid=$!
nap 10
key o
key -M ctrl s -m ctrl
nap 2
shot c7c-saving
swaymsg -q '[title=" - cinq$"] kill' || true
wait "$pid" 2> /dev/null || true
chflags noschg "$work/c7/ro" 2>> "$out/c7-chflags.txt" || true

# 8. Network cue to a closed port
start c8 net.cinq.toml "[keys]
$go"
key -k F9
nap 3
shot c8-1-fired
stop c8

mark done
swaymsg exit > /dev/null 2>&1 || true
sleep 2
kill "$sway" 2> /dev/null || true
wait 2> /dev/null || true
