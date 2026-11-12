#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Composes what demo.sh recorded in OUT into one video, RESULT: the main
# window beside the waveform of each of the eight channels of the desk,
# named after the output that plays there, and the captions of demo.sh in
# a band at the bottom.  Odd channels play in the left ear and even ones
# in the right, so that Sub, moving from channel 3 to channel 4, moves from
# one ear to the other.  The sound lines up with the picture by the onset
# of the music, which starts as GO is pressed.
#
# Usage: compose.sh OUT FONT RESULT, with ffmpeg on PATH and FONT a TrueType
# font file.
set -euo pipefail
out=$1 font=$2 result=$3
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

at() { awk -F'\t' -v key="$1" '$2 == key { print $1; exit }' "$out/marks.tsv"; }
since() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%.3f", a - b }'; }
start=$(at "record screen")
length=$(since "$(at end)" "$start")
onset=$(ffmpeg -hide_banner -i "$out/desk.wav" -af 'pan=mono|c0=c0,silencedetect=n=-50dB:d=0.3' \
  -f null - 2>&1 | awk '/silence_end/ { print $5; exit }')
delay=$(awk -v go="$(since "$(at house)" "$start")" -v onset="$onset" \
  'BEGIN { printf "%d", (go + 0.02 - onset) * 1000 }')
echo "sound $delay ms after the screen"

# Each caption shows until the next one, or until the end
awk -F'\t' -v start="$start" -v end="$(at end)" '
  $2 == "caption" { time[n] = $1 - start; text[n++] = $3 }
  END { for (i = 0; i < n; i++) printf "%.3f\t%.3f\t%s\n", time[i], (i + 1 < n ? time[i + 1] : end - start), text[i] }
' "$out/marks.tsv" > "$work/captions.tsv"
captions=$(awk -F'\t' -v work="$work" -v font="$font" '{
  file = work "/caption-" NR ".txt"
  printf "%s", $3 > file
  close(file)
  printf ",drawtext=fontfile=%s:textfile=%s:expansion=none:fontcolor=white:fontsize=26:x=(w-text_w)/2:y=h-58:enable=%sbetween(t,%s,%s)%s", font, file, "\047", $1, $2, "\047"
}' "$work/captions.tsv")

# Eight bands of 112 pixels, each named by its channel and the output that
# the show first puts there
names=("1 House L" "2 House R" "3 Sub" "4" "5 Monitors L" "6 Monitors R" "7 Lobby L" "8 Lobby R")
colors=$(printf "deepskyblue|orange|%.0s" 1 2 3 4)
labels=$(for index in "${!names[@]}"; do
  printf ",drawtext=fontfile=%s:text='%s':fontcolor=white:fontsize=22:x=10:y=%d" \
    "$font" "${names[$index]}" $((index * 112 + 6))
  printf ",drawbox=x=0:y=%d:w=iw:h=1:color=gray" $((index * 112))
done)

ffmpeg -loglevel error -y \
  -i "$out/screen.mkv" \
  -i "$out/desk.wav" \
  -filter_complex "
    [0:v]setpts=PTS-STARTPTS,fps=30,scale=1280:900[screen];
    [1:a]adelay=$delay:all=1,asplit=2[sound][shown];
    [shown]showwaves=s=640x896:mode=cline:draw=full:rate=30:split_channels=1:colors=$colors,pad=640:900:0:2:black$labels[waves];
    [screen][waves]hstack=inputs=2,pad=iw:ih+80:0:0:black$captions,format=yuv420p[video];
    [sound]pan=stereo|c0=0.5*c0+0.5*c2+0.5*c4+0.5*c6|c1=0.5*c1+0.5*c3+0.5*c5+0.5*c7[stereo]
  " \
  -map '[video]' -map '[stereo]' -t "$length" \
  -c:v libx264 -crf 23 -preset medium -c:a aac -b:a 160k -movflags +faststart "$result"
