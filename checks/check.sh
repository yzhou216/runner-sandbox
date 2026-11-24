#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Eight checks of cinq on GNU/Linux in headless sway.  Every phase starts cinq
# with a configuration directory of its own, logs standard error to
# OUT/PHASE.err, and takes screenshots into OUT.
#
# Usage: check.sh FONTS OUT DEMO, with cinq, sway, wtype, grim, and python3 on
# PATH, DEMO the directory of the demo shows, and HINT_LABEL the label of Add cue
set -uo pipefail
here=$(dirname "$(readlink -f "$0")")
fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
demo=$(readlink -f "$3")
work=$(mktemp -d)

export HOME=$work/home XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
printf '<?xml version="1.0"?><fontconfig><dir>%s</dir><cachedir>%s/fc</cachedir></fontconfig>\n' "$fonts" "$work" > "$FONTCONFIG_FILE"
cat > "$work/sway.config" <<'CONF'
output HEADLESS-1 resolution 1280x800
for_window [app_id="cinq"] floating enable, resize set 1100 700, move position center
CONF
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"

sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK
until swaymsg -t get_version > /dev/null 2>&1; do sleep 0.5; done
uname -a > "$out/runner.txt"

shot() { grim "$out/$1.png" || true; }
focus() { swaymsg -q '[app_id="cinq"] focus'; sleep 0.5; }
key() { focus; wtype -s 300 -d 100 "$@"; sleep 1; }
ctrl() { focus; wtype -s 300 -M ctrl "$1" -m ctrl; sleep 1; }
say() { printf '%s %s\n' "$(date +%T)" "$*" | tee -a "$out/steps.txt"; }

# A show of sounds beside the show file, in DIR
mkshow() {
  mkdir -p "$1/sounds"
  cp "$demo"/sounds/rain.flac "$demo"/sounds/bell.mp3 "$demo"/sounds/knock.wav "$1/sounds/"
  cat > "$1/show.cinq.toml"
}
# conf DIR LINE...: config.toml with GO also on F9
conf() {
  local dir=$1
  shift
  mkdir -p "$dir"
  { printf '%s\n' "$@"; printf '\n[keys]\ngo = ["Space", "F9"]\n'; } > "$dir/config.toml"
}
# start PHASE CONFIGDIR [SHOW]
start() {
  phase=$1
  : > "$out/$phase.err"
  cinq --config-dir "$2" ${3:+"$3"} 2>> "$out/$phase.err" &
  cinq=$!
  sleep 8
  shot "$phase-0-start"
}
stop() {
  swaymsg -q '[app_id="cinq"] kill' || true
  wait "$cinq" || true
  sleep 1
}

audio_show() {
  mkshow "$1" <<'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "audio"
name = "Rain"
file = "sounds/rain.flac"

[[list.cue]]
id = 2
number = "2"
kind = "audio"
name = "Bell"
file = "sounds/bell.mp3"

[[list.cue]]
id = 3
number = "3"
kind = "memo"
name = "Note"
TOML
}

# 1. Silent start: the demo, and no show at all, without a config.toml
say "1 silent start"
mkdir -p "$work/c1"
start 1-demo "$work/c1" "$demo/storm.cinq.toml"
sleep 4
shot 1-demo-1-idle
stop
start 1-empty "$work/c1"
stop
for p in 1-demo 1-empty; do say "1 $p stderr bytes: $(wc -c < "$out/$p.err")"; done

# 2. The retired keymap
say "2 keymap cua"
audio_show "$work/s2"
conf "$work/c2" 'keymap = "cua"'
start 2-cua "$work/c2" "$work/s2/show.cinq.toml"
key j j
shot 2-cua-1-j
key o o
shot 2-cua-2-o
stop
say "2 stderr: $(cat "$out/2-cua.err")"

# 3. Hint mode
say "3 hint"
audio_show "$work/s3"
conf "$work/c3"
start 3-hint "$work/c3" "$work/s3/show.cinq.toml"
key f f
shot 3-hint-1-labels
key a a
key -k BackSpace
shot 3-hint-2-backspace-after-a
key -k BackSpace
shot 3-hint-3-left
key f f
if [ -n "${HINT_LABEL:-}" ]; then
  key "$HINT_LABEL"
  shot 3-hint-4-typed
fi
stop
say "3 stderr: $(cat "$out/3-hint.err")"

# 4. OSC listener
say "4 osc"
audio_show "$work/s4"
conf "$work/c4" 'osc_listen = "127.0.0.1:53000"'
start 4-osc "$work/c4" "$work/s4/show.cinq.toml"
python3 "$here/osc.py" /go
sleep 3
shot 4-osc-1-go
python3 "$here/osc.py" /cue/99/start
sleep 3
shot 4-osc-2-cue99
stop
say "4 stderr: $(cat "$out/4-osc.err")"
# The port taken by another socket
python3 - <<'PY' &
import socket, time
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(("127.0.0.1", 53000))
time.sleep(40)
PY
holder=$!
sleep 1
start 4-taken "$work/c4" "$work/s4/show.cinq.toml"
sleep 3
shot 4-taken-1
kill "$holder" 2>/dev/null || true
sleep 5
shot 4-taken-2-released
stop
say "4 taken stderr: $(cat "$out/4-taken.err")"

# 5. File watching
say "5 watch"
audio_show "$work/s5"
conf "$work/c5"
start 5-watch "$work/c5" "$work/s5/show.cinq.toml"
mv "$work/s5/sounds/rain.flac" "$work/rain.flac.away"
sleep 4
shot 5-watch-1-deleted
mv "$work/rain.flac.away" "$work/s5/sounds/rain.flac"
sleep 4
shot 5-watch-2-restored
stop
say "5 stderr: $(cat "$out/5-watch.err")"

# 6. Paste
say "6 paste"
audio_show "$work/s6"
conf "$work/c6"
start 6-paste "$work/c6" "$work/s6/show.cinq.toml"
key yy yy
key p p
shot 6-paste-1-now
sleep 5
shot 6-paste-2-later
stop
say "6 stderr: $(cat "$out/6-paste.err")"

# 7. Errors in words
say "7 errors"
mkdir -p "$work/adir"
conf "$work/c7"
start 7-dir "$work/c7" "$work/adir"
stop
say "7 dir stderr: $(cat "$out/7-dir.err")"
audio_show "$work/s7"
start 7-save "$work/c7" "$work/s7/show.cinq.toml"
chmod 555 "$work/s7"
ctrl s
shot 7-save-1
stop
chmod 755 "$work/s7"
say "7 save stderr: $(cat "$out/7-save.err")"

# 8. A network cue to a closed port
say "8 network"
mkshow "$work/s8" <<'TOML'
[[list]]
name = "Main"

[[list.cue]]
id = 1
number = "1"
kind = "network"
name = "Send"
destination = "127.0.0.1:9"
message = "/test"
TOML
conf "$work/c8"
start 8-net "$work/c8" "$work/s8/show.cinq.toml"
key F9 -k F9
sleep 3
shot 8-net-1-go
stop
say "8 stderr: $(cat "$out/8-net.err")"

swaymsg exit > /dev/null 2>&1 || true
