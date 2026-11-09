#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Records cinq's light cues driving three lamps, in a headless sway session
# of its own: view.py draws the Art-Net that cinq sends to 127.0.0.1 as
# lamps in a terminal on the left, and cinq's window on the right shows the
# light patch with the level of each instrument as GO runs the cues of
# show.toml.
#
# Usage: demo.sh FONTS OUT, with cinq, sway, foot, python3, wtype,
# wf-recorder, and ffmpeg on PATH, FONTS the directory of the DejaVu fonts,
# and OUT the directory for the video.
set -euo pipefail

here=$(dirname "$(readlink -f "$0")")
fonts=$(readlink -f "$1")
out=$(mkdir -p "$2" && readlink -f "$2")
work=$(mktemp -d)

# Session: one 1920x1080 output, the DejaVu fonts alone, a home of its own,
# and cinq's window moved to the right of the lamps as it opens
export HOME=$work/home XDG_RUNTIME_DIR=$work/run FONTCONFIG_FILE=$work/fonts.conf
export WLR_BACKENDS=headless WLR_RENDERER=pixman WLR_LIBINPUT_NO_DEVICES=1
export ICED_BACKEND=tiny-skia
unset WAYLAND_DISPLAY DISPLAY SWAYSOCK DBUS_SESSION_BUS_ADDRESS
unset XDG_CONFIG_HOME XDG_STATE_HOME XDG_DATA_HOME XDG_CACHE_HOME
mkdir -p "$HOME"
mkdir -m 700 "$XDG_RUNTIME_DIR"
cat > "$FONTCONFIG_FILE" << EOF
<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "urn:fontconfig:fonts.dtd">
<fontconfig>
  <dir>$fonts</dir>
  <cachedir>$work/fontconfig</cachedir>
  <alias><family>monospace</family><prefer><family>DejaVu Sans Mono</family></prefer></alias>
  <alias><family>sans-serif</family><prefer><family>DejaVu Sans</family></prefer></alias>
</fontconfig>
EOF
cat > "$work/sway.config" << 'EOF'
output HEADLESS-1 resolution 1920x1080
font pango:DejaVu Sans 11
default_border normal 2
for_window [app_id="cinq"] move right, resize set width 56 ppt
EOF
sway -c "$work/sway.config" > "$out/sway.log" 2>&1 &
for _ in $(seq 100); do
  socket=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'wayland-[0-9]' -printf '%f\n' | head -1)
  [ -n "$socket" ] && break
  sleep 0.1
done
export WAYLAND_DISPLAY=$socket
SWAYSOCK=$(find "$XDG_RUNTIME_DIR" -maxdepth 1 -name 'sway-ipc.*.sock' | head -1)
export SWAYSOCK

# No sound device to warn about
printf 'pcm.!default { type null }\nctl.!default { type null }\n' > "$HOME/.asoundrc"

foot -o 'font=DejaVu Sans Mono:size=16' -o 'pad=8x8' --app-id=lamps \
  --title='Art-Net lamps (view.py)' python3 "$here/view.py" 3 > "$out/lamps.log" 2>&1 &
sleep 2
cinq "$here/show.toml" > "$out/cinq.log" 2>&1 &
sleep 4

# Keys for cinq, typed one by one so that they show
keys() {
  swaymsg -q '[app_id="cinq"] focus'
  wtype -s 300 -d 90 "$@"
}
# Captions of the video, each from the time it is set
caption() {
  printf '%s\t%s\n' "$(awk -v now="$(date +%s.%N)" -v then="$started" 'BEGIN { printf "%.2f", now - then }')" "$1" >> "$captions"
}
# Burns the captions into a band under the picture
burn() {
  local filters=() at text next
  mapfile -t lines < "$captions"
  for ((i = 0; i + 1 < ${#lines[@]}; i++)); do
    IFS=$'\t' read -r at text <<< "${lines[i]}"
    IFS=$'\t' read -r next _ <<< "${lines[i + 1]}"
    printf '%s' "$text" > "$work/caption-$i.txt"
    filters+=("drawtext=fontfile=$fonts/truetype/DejaVuSans.ttf:textfile=$work/caption-$i.txt:expansion=none:fontsize=30:fontcolor=white:x=(w-text_w)/2:y=1080+(80-text_h)/2:enable='between(t,$at,$next)'")
  done
  local IFS=,
  ffmpeg -loglevel error -y -i "$work/light.mp4" \
    -vf "pad=iw:ih+80:0:0:black,${filters[*]}" \
    -c:v libx264 -crf 22 -pix_fmt yuv420p "$out/light.mp4"
}

captions=$out/light.captions
: > "$captions"
wf-recorder -D -f "$work/light.mp4" > "$out/recorder.log" 2>&1 &
recorder=$!
started=$(date +%s.%N)
sleep 1

caption 'Left: view.py draws the Art-Net that cinq sends as three lamps.  Right: cinq with show.toml'
sleep 4
caption 'g T opens the Light Patch tab, whose Now column shows the level of each instrument'
keys gT
sleep 4
caption 'GO on cue 1 fades Front up to 70% over 3 seconds'
keys -k space
sleep 5
caption 'Cue 2 fades Back and Cyc up over 5 seconds, while Front, which it does not name, stays at 70%'
keys -k space
sleep 7
caption 'Cue 3 starts to fade Front and Back out over 8 seconds, leaving Cyc, which it does not name...'
keys -k space
sleep 3.5
caption '...and Escape (panic) stops it halfway: the lamps hold where the fade left them'
keys -k Escape
sleep 5
caption 'Cue 4 fades all three lamps out over 3 seconds'
keys -k space
sleep 5

sleep 1
kill -INT "$recorder"
wait "$recorder" || true
caption ''
burn
swaymsg -q exit || true
