# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Helpers that jack.sh, pulse.sh, and clamp.sh share, sourced after setting
# cinq, out, and here: a private session with a headless sway, and cinq
# started on a show in a configuration directory of its own, whose GO key is
# F5, so that no Space ever reaches cinq.  The log of cinq, whose lines the
# patch of this branch stamps with the time, goes to OUT/PHASE.log, and the
# marks of the script to OUT/PHASE.marks.
work=$(mktemp -d)
sway='' pid=''
export HOME=$work/home XDG_RUNTIME_DIR=$work/run XDG_CONFIG_HOME=$work/home/.config
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS PULSE_SERVER
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"

stop_cinq() {
  [ -n "$pid" ] && kill "$pid" 2> /dev/null || true
  [ -n "$pid" ] && wait "$pid" 2> /dev/null || true
  pid=''
}

# Starts sway with one output of 1280x900, in which cinq floats at 900x640
session() {
  printf 'output HEADLESS-1 mode 1280x900\ndefault_border none\nfor_window [app_id="cinq"] floating enable, resize set 900 640, move position center\n' > "$work/sway.config"
  sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
  sway=$!
  for _ in $(seq 300); do
    socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
    SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
    [ -n "$socket" ] && [ -n "$SWAYSOCK" ] && SWAYSOCK=$SWAYSOCK swaymsg -t get_version > /dev/null 2>&1 && break
    sleep 0.1
  done
  export WAYLAND_DISPLAY=$socket SWAYSOCK
}

# mark PHASE LABEL: writes the time and the label to PHASE.marks
mark() { printf '%s\t%s\n' "$(date +%s.%N)" "$2" >> "$out/$1.marks"; }

# Starts cinq on the show in the file SHOW.cinq.toml of the work directory,
# for the phase PHASE, with its own configuration directory
cinq_start() {
  mkdir -p "$work/cfg-$1"
  printf 'keymap = "vi"\n\n[keys]\ngo = ["F5"]\n' > "$work/cfg-$1/config.toml"
  cp "$work/$2.cinq.toml" "$out/$1.cinq.toml"
  : > "$out/$1.marks"
  (cd "$work" && exec "$cinq" --config-dir "$work/cfg-$1" "$2.cinq.toml" 2> "$out/$1.log") &
  pid=$!
  mark "$1" "start"
}

# Types KEYS, as wtype takes them, after focusing cinq, and marks the phase
# PHASE with LABEL
keys() {
  local phase=$1 label=$2
  shift 2
  swaymsg -q '[app_id="cinq"] focus' || true
  mark "$phase" "$label"
  wtype -s 300 -d "${KEYDELAY:-80}" "$@"
}

# Waits for the window of cinq
window() {
  for _ in $(seq 200); do
    swaymsg -t get_tree 2> /dev/null | command grep -q '"app_id": "cinq"' && return 0
    sleep 0.1
  done
  return 1
}

# Pieces of a show
# shellcheck source=show.sh
. "$(dirname "${BASH_SOURCE[0]}")/show.sh"
