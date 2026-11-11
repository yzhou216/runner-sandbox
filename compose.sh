#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Composes what demo.sh recorded in OUT into one video, RESULT: the main
# window beside the stage window, the waveform of each audio device under
# them, and the captions of demo.sh in a band at the bottom.  The house
# sink plays in the left channel and the booth sink in the right, so that a
# sound that moves from one device to the other moves from one ear to the
# other.  The sound lines up with the picture by the music, which starts as
# GO is pressed.
#
# Usage: compose.sh OUT FONT RESULT, with ffmpeg on PATH and FONT a TrueType
# font file.
set -euo pipefail
out=$1 font=$2 result=$3
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

at() { awk -F'\t' -v key="$1" '$2 == key { print $1; exit }' "$out/marks.tsv"; }
since() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%.3f", a - b }'; }
start=$(at "record HEADLESS-1")
length=$(since "$(at end)" "$start")
# GO starts the music made ready ahead, which reaches the sink within a
# buffer of PulseAudio, about 20 milliseconds.  parecord starts recording
# a second or two after it starts, so its sound comes later by the rest.
# Both recorders of the screens start within milliseconds of each other.
onset=$(ffmpeg -hide_banner -i "$out/house.wav" -af silencedetect=n=-50dB:d=0.3 -f null - 2>&1 |
  awk '/silence_end/ { print $5; exit }')
delay=$(awk -v go="$(since "$(at go-music)" "$start")" -v onset="$onset" \
  'BEGIN { printf "%d", (go + 0.02 - onset) * 1000 }')
echo "sound $delay ms after the screens"

# Each caption shows until the next one, or until the end
awk -F'\t' -v start="$start" -v end="$(at end)" '
  $2 == "caption" { time[n] = $1 - start; text[n++] = $3 }
  END { for (i = 0; i < n; i++) printf "%.3f\t%.3f\t%s\n", time[i], (i + 1 < n ? time[i + 1] : end - start), text[i] }
' "$out/marks.tsv" > "$work/captions.tsv"
captions=$(awk -F'\t' -v work="$work" -v font="$font" '{
  file = work "/caption-" NR ".txt"
  printf "%s", $3 > file
  close(file)
  printf ",drawtext=fontfile=%s:textfile=%s:expansion=none:fontcolor=white:fontsize=28:x=(w-text_w)/2:y=h-62:enable=%sbetween(t,%s,%s)%s", font, file, "\047", $1, $2, "\047"
}' "$work/captions.tsv")
label() {
  printf "drawtext=fontfile=%s:text='%s':fontcolor=white:fontsize=24:x=12:y=8" "$font" "$1"
}

ffmpeg -loglevel error -y \
  -i "$out/HEADLESS-1.mkv" \
  -i "$out/HEADLESS-2.mkv" \
  -i "$out/house.wav" \
  -i "$out/booth.wav" \
  -filter_complex "
    [0:v]setpts=PTS-STARTPTS,fps=30[main];
    [1:v]fps=30[stage];
    [main][stage]hstack=inputs=2,scale=1920:-2[screens];
    [2:a]adelay=$delay:all=1,pan=mono|c0=0.5*c0+0.5*c1,asplit=2[house][houseshown];
    [3:a]adelay=$delay:all=1,pan=mono|c0=0.5*c0+0.5*c1,asplit=2[booth][boothshown];
    [houseshown]showwaves=s=960x140:mode=cline:rate=30:colors=0x7fd7ff,$(label 'House speakers, left')[housewave];
    [boothshown]showwaves=s=960x140:mode=cline:rate=30:colors=0xffb070,$(label 'Booth monitors, right')[boothwave];
    [housewave][boothwave]hstack=inputs=2[waves];
    [screens][waves]vstack=inputs=2,pad=iw:ih+80:0:0:black$captions,format=yuv420p[video];
    [house][booth]join=inputs=2:channel_layout=stereo[sound]
  " \
  -map '[video]' -map '[sound]' -t "$length" \
  -c:v libx264 -crf 23 -preset medium -c:a aac -b:a 160k -movflags +faststart "$result"
