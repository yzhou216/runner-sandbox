#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Checks 2 to 7 of the second brief on cinq in headless sway, on FreeBSD:
# idle processor time of OLD and NEW with 500 files and with the light patch,
# the file watch, light news in a VNET jail without a network, clean exit, and
# a config.toml that starts with a byte order mark.  Results go to
# OUT/results.txt; screenshots and standard error to OUT.
#
# Usage: idle.sh OLD NEW OUT SCALE STAGE (idle, c4, c5, or c7), with sway, wtype, grim, and python3 on PATH;
# OLD and NEW are cinq binaries; SCALE stretches every wait (1 natively, 4
# when emulated)
set -uo pipefail

old=$1
new=$2
out=$(mkdir -p "$3" && cd "$3" && pwd)
scale=${4:-1}
rounds=${ROUNDS:-2}
stage=${5:-idle}
label=${AUDIO_LABEL:-}
item=${ITEM_LABEL:-}
work=$(mktemp -d)
nap() { sleep "$(awk -v s="$1" -v k="$scale" 'BEGIN { print s * k }')"; }
ts() {
  local t
  t=$(date +%s.%N)
  case $t in
    *N*) python3 -c 'import time; print("%.2f" % time.time())' ;;
    *) echo "$t" ;;
  esac
}
res() { printf '%s\n' "$*" | tee -a "$out/results.txt"; }
mark() { printf '%s %s\n' "$(ts)" "$*" | tee -a "$out/marks.txt"; }

export HOME=$work/home XDG_RUNTIME_DIR=$work/run
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS
mkdir -p "$HOME"
[ "${ASOUND:-null}" = null ] && printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"
tag=${ASOUND:-null}
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf 'output HEADLESS-1 resolution 1280x800\n' > "$work/sway.config"
{ uname -a; "$old" --version; "$new" --version; nproc 2> /dev/null; sysctl -n hw.model hw.ncpu; } > "$out/versions.txt" 2>&1

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

# Sounds: one of 2 s for the 500 cues, one of 5 s to swap in, one for thunder
shows=$work/shows
mkdir -p "$shows/sounds" "$shows/lights/sounds"
python3 - "$work" <<'PY'
import math, struct, sys, wave
def make(path, seconds, hz):
    with wave.open(path, "wb") as w:
        w.setnchannels(1); w.setsampwidth(2); w.setframerate(22050)
        w.writeframes(b"".join(struct.pack("<h", int(8000 * math.sin(2 * math.pi * hz * i / 22050))) for i in range(22050 * seconds)))
make(sys.argv[1] + "/two.wav", 2, 440)
make(sys.argv[1] + "/five.wav", 5, 330)
make(sys.argv[1] + "/lights-thunder.wav", 2, 110)
PY
for i in $(seq 500); do cp "$work/two.wav" "$shows/sounds/$(printf 's%03d' "$i").wav"; done
awk 'BEGIN {
  print "[[list]]\nname = \"Main\"\n"
  for (i = 1; i <= 500; i++)
    printf "[[list.cue]]\nid = %d\nnumber = \"%d\"\nkind = \"audio\"\nname = \"Cue %d\"\nfile = \"sounds/s%03d.wav\"\n\n", i, i, i, i
}' > "$shows/big.cinq.toml"
cp "$work/lights-thunder.wav" "$shows/lights/sounds/thunder.wav"
cp "$LIGHTS_OLD" "$shows/lights/old.cinq.toml"
cp "$LIGHTS_NEW" "$shows/lights/new.cinq.toml"
cat > "$shows/small.cinq.toml" <<'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Alpha"
file = "sounds/alpha.wav"
TOML
cp "$work/two.wav" "$shows/sounds/alpha.wav"
res "show: $(grep -c '^\[\[list.cue\]\]' "$shows/big.cinq.toml") cues, $(ls "$shows/sounds" | wc -l | tr -d ' ') files, $(du -sk "$shows/sounds" | cut -f1) KiB"

pid=''
shot() { grim "$out/$1.png" || true; }
focus() { swaymsg -q '[title=" - cinq$"] focus' || true; nap 0.5; }
key() { focus; wtype -s 300 -d 100 "$@"; nap 1.5; }
# cpu PID: processor seconds of the process, as ps shows them
cpu() { ps -o time= -p "$1" | awk -F: '{ s = 0; for (i = 1; i <= NF; i++) s = s * 60 + $i; printf "%.2f\n", s }'; }
# start PHASE REVISION SHOW [CONFIG TEXT [JEXEC]]: a fresh copy of the show
start() {
  local dir=$work/$1 bin=$2
  mkdir -p "$dir/cfg"
  cp -R "$shows/." "$dir/"
  [ -n "${4:-}" ] && printf '%s\n' "$4" > "$dir/cfg/config.toml"
  mark "start $1"
  ${5:+jexec "$5"} "$bin" --config-dir "$dir/cfg" "$dir/$3" 2> "$out/$1.err" &
  pid=$!
}
# stop PHASE: closes the window and times how long the process lasts
stop() {
  local t0 t1 gone=gone
  t0=$(ts)
  mark "stop $1"
  swaymsg -q '[title=" - cinq$"] kill' || true
  for _ in $(seq 600); do kill -0 "$pid" 2> /dev/null || break; sleep 0.1; done
  t1=$(ts)
  ps -H -o tid,time,pcpu,state,wchan,comm -p "$pid" > "$out/$1.threads-after" 2>&1
  kill -0 "$pid" 2> /dev/null && { gone=LINGERS; kill "$pid"; }
  wait "$pid" 2> /dev/null || true
  res "exit $1: $(awk -v a="$t0" -v b="$t1" 'BEGIN { printf "%.2f", b - a }') s, process $gone"
  [ -f "$work/$1/cfg/config.toml" ] && cp "$work/$1/cfg/config.toml" "$out/$1.config.toml"
  nap 1
}
# idle PHASE: processor time over 60 s, after the files have loaded
idle() {
  local a b t0 t1
  nap 20
  shot "$1-0-loaded"
  ps -H -o tid,time,pcpu,state,wchan,comm -p "$pid" > "$out/$1.threads-before" 2>&1
  a=$(cpu "$pid")
  t0=$(ts)
  sleep 60
  b=$(cpu "$pid")
  t1=$(ts)
  ps -H -o tid,time,pcpu,state,wchan,comm -p "$pid" > "$out/$1.threads-after" 2>&1
  res "idle $1: $a s -> $b s over $(awk -v a="$t0" -v b="$t1" 'BEGIN { printf "%.1f", b - a }') s = $(awk -v a="$a" -v b="$b" -v t0="$t0" -v t1="$t1" 'BEGIN { printf "%.2f", (b - a) / (t1 - t0) * 100 }') percent of one core"
}
# burst NAME SECONDS: frames of the screen for a while, with their times,
# keeping only the first and each that differs from the one before
burst() {
  local end t sum last=''
  end=$(awk -v n="$(ts)" -v s="$2" -v k="$scale" 'BEGIN { printf "%.2f", n + s * k }')
  while [ "$(awk -v n="$(ts)" -v e="$end" 'BEGIN { print (n < e) }')" = 1 ]; do
    t=$(ts)
    grim "$out/$1-$t.png" || continue
    sum=$(sha256 -q "$out/$1-$t.png")
    echo "$t $sum" >> "$out/$1.frames"
    if [ "$sum" = "$last" ]; then rm -f "$out/$1-$t.png"; fi
    last=$sum
  done
}
go='go = ["Space", "F9"]'

if [ "$stage" = idle ]; then
# Checks 2, 3, and 6: idle time and exit, OLD and NEW alternately
for round in $(seq "$rounds"); do
  for which in old new; do
    bin=$old
    [ "$which" = new ] && bin=$new
    start "big-$which-$round-$tag" "$bin" big.cinq.toml
    idle "big-$which-$round-$tag"
    stop "big-$which-$round-$tag"
    start "lights-$which-$round-$tag" "$bin" "lights/$which.cinq.toml"
    idle "lights-$which-$round-$tag"
    stop "lights-$which-$round-$tag"
  done
done
fi

if [ "$stage" = c4 ]; then
# Check 4: file watch under NEW, 500 cues
start c4 "$new" big.cinq.toml "[keys]
$go"
nap 20
d=$work/c4/sounds
burst c4a 14 &
b=$!
nap 3
mv "$d/s003.wav" "$d/s003.gone"
res "c4 deleted s003 at $(ts)"
wait "$b"
burst c4b 14 &
b=$!
nap 3
mv "$d/s003.gone" "$d/s003.wav"
res "c4 restored s003 at $(ts)"
wait "$b"
burst c4c 14 &
b=$!
nap 3
cp "$work/five.wav" "$d/s005.new"
mv "$d/s005.new" "$d/s005.wav"
res "c4 replaced s005 (2 s -> 5 s) at $(ts)"
wait "$b"
shot c4-final
stop c4
fi


if [ "$stage" = c5 ]; then
# Check 5: light news in a VNET jail, loopback at first, then none, then back
if jail -c name=nonet path=/ vnet persist > "$out/jail.txt" 2>&1; then
  res "jail nonet created (vnet)"
  jexec nonet ifconfig lo0 inet 127.0.0.1/8 up
  jexec nonet ifconfig > "$out/jail-ifconfig-1.txt" 2>&1
  start c5 "$new" lights/new.cinq.toml "[keys]
$go" nonet
  nap 10
  shot c5-0-start
  key -k F9
  nap 2
  shot c5-1-go-loopback
  jexec nonet ifconfig lo0 inet 127.0.0.1 -alias
  res "c5 loopback address removed at $(ts)"
  burst c5a 12
  jexec nonet ifconfig > "$out/jail-ifconfig-2.txt" 2>&1
  jexec nonet ifconfig lo0 inet 127.0.0.1/8
  res "c5 loopback address back at $(ts)"
  burst c5b 12
  stop c5
  jail -r nonet || true
else
  res "jail nonet NOT created: $(cat "$out/jail.txt")"
fi
fi

if [ "$stage" = c7 ]; then
# Check 7: byte order mark
start c7 "$new" small.cinq.toml "$(printf '\357\273\277[keys]\n%s' "$go")"
nap 10
shot c7-0-start
key -k F9
nap 2
shot c7-1-f9
if [ -n "$label" ]; then
  key f
  shot c7-2-hints
  key "$label"
  shot c7-3-label
  if [ -n "$item" ]; then
    key f
    shot c7-4-hints2
    key "$item"
    shot c7-5-item
  fi
fi
stop c7
od -c "$work/c7/cfg/config.toml" | head -5 > "$out/c7-config.od"
start c7b "$new" small.cinq.toml "$(cat "$work/c7/cfg/config.toml")"
nap 10
shot c7b-0-restart
stop c7b
res "c7 stderr bytes: first $(wc -c < "$out/c7.err" | tr -d ' '), restart $(wc -c < "$out/c7b.err" | tr -d ' ')"
res "c7 config after first run: $(od -c "$work/c7/cfg/config.toml" | head -4 | tr '\n' ' ')"
res "c7 config after restart: $(od -c "$work/c7b/cfg/config.toml" | head -4 | tr '\n' ' ')"
fi

mark done
swaymsg exit > /dev/null 2>&1 || true
sleep 2
kill "$sway" 2> /dev/null || true
