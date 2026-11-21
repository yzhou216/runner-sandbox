#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Composes what record.sh recorded in OUT into one video, RESULT: the screen
# on top, under it a graph of the level that was heard, in dB over time,
# which draws itself as the video plays, and under both a band of captions.
# It also writes levels.tsv and levels.md into OUT, the level that the
# recorded sound had at the end of each wait.
#
# Three clocks meet here.  The wall clock stamps events.tsv.  The screen's
# clock is tied to it by the flash, one white frame that record.sh makes at a
# stamped moment.  The sound's clock is tied to it by cue 1, whose first
# sound follows its GO, so that parec starting late, which it does by a
# second or two, shifts nothing.  The sound then goes into the video's time
# with adelay, never -itsoffset.
#
# Usage: compose.sh OUT FONT RESULT TITLE, with ffmpeg and awk on PATH, FONT a
# TrueType font file, and TITLE the heading of the video.
set -euo pipefail
out=$1 font=$2 result=$3 title=$4
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

width=1024 graph_height=230 left=64 right=16 top=24 bottom=26 floor=-50
band=84 step=0.1

stamp() { awk -F'\t' -v key="$1" '$2 == key { print $1; exit }' "$out/events.tsv"; }
minus() { awk -v a="$1" -v b="$2" 'BEGIN { printf "%.3f", a - b }'; }

# Seconds into the screen's video of its first bright frame
flash_video=$(ffmpeg -nostdin -hide_banner -loglevel error -i "$out/screen.mp4" \
  -vf "signalstats,metadata=print:key=lavfi.signalstats.YAVG:file=$work/yavg.txt" -f null -
  awk -F'[ =:]+' '/pts_time/ { t = $NF } /YAVG/ { if ($NF > 128) { print t; exit } }' "$work/yavg.txt")
test -n "$flash_video"
# Wall clock at the first frame of the screen's video
screen0=$(minus "$(stamp flash)" "$flash_video")

# Level of the sound, as RMS in dB over steps of 100 ms, one per line
levels() {
  ffmpeg -nostdin -hide_banner -loglevel error -i "$1" \
    -af "asetnsamples=n=4800:p=0,astats=metadata=1:reset=1:measure_perchannel=none:measure_overall=RMS_level,ametadata=mode=print:key=lavfi.astats.Overall.RMS_level:file=$work/raw.txt" \
    -f null -
  awk -F= -v floor="$floor" '/RMS_level/ { v = ($2 == "-inf" || $2 < floor) ? floor : $2; printf "%.2f\n", v }' "$work/raw.txt"
}

# First step of the raw recording that holds sound, taken from its start
onset=$(levels "$out/sound.wav" | awk -v step="$step" '$1 > -50 { printf "%.3f", (NR - 1) * step; exit }')
test -n "$onset"
# Wall clock at the first sample of the sound, with GO as the instant that
# the sound starts
sound0=$(minus "$(stamp go-1)" "$onset")

# The video runs from three seconds before GO to the end
go_video=$(minus "$(stamp go-1)" "$screen0")
begin=$(minus "$go_video" 3)
length=$(awk -v end="$(minus "$(stamp end)" "$screen0")" -v begin="$begin" 'BEGIN { printf "%.3f", end - begin }')
# Position of the first sample of the sound in the video, which starts at BEGIN
shift=$(minus "$(minus "$sound0" "$screen0")" "$begin")
delay=$(awk -v s="$shift" 'BEGIN { printf "%d", (s > 0 ? s : 0) * 1000 }')
cut=$(awk -v s="$shift" 'BEGIN { printf "%.3f", (s < 0 ? -s : 0) }')
echo "screen starts $screen0, sound $onset s into its recording, delayed $delay ms, cut $cut s"

ffmpeg -nostdin -hide_banner -loglevel error -y -i "$out/sound.wav" \
  -af "atrim=start=$cut,asetpts=PTS-STARTPTS,adelay=$delay:all=1,apad,atrim=end=$length" \
  -ar 48000 "$work/heard.wav"
levels "$work/heard.wav" > "$work/series.txt"

# Raster of the graph as a plain PPM: the series as a line over a grid at
# every 10 dB, in the part of the picture right of the labels
plot_width=$((width - left - right)) plot_height=$((graph_height - top - bottom))
awk -v w="$width" -v h="$graph_height" -v left="$left" -v pw="$plot_width" \
  -v top="$top" -v ph="$plot_height" -v floor="$floor" -v step="$step" -v len="$length" '
  function row(db) { return top + int((0 - db) / (0 - floor) * (ph - 1)) }
  function put(x, y, r, g, b) { if (x >= 0 && x < w && y >= 0 && y < h) { R[y * w + x] = r; G[y * w + x] = g; B[y * w + x] = b } }
  { level[NR - 1] = $1 }
  END {
    for (i = 0; i < w * h; i++) { R[i] = 16; G[i] = 16; B[i] = 20 }
    for (db = 0; db >= floor; db -= 10) for (x = left; x < left + pw; x++) put(x, row(db), 70, 70, 80)
    for (s = 0; s <= len; s += 5) for (y = top; y < top + ph; y++) put(left + int(s / len * (pw - 1)), y, 40, 40, 48)
    prev = -1
    for (x = 0; x < pw; x++) {
      i = int(x / (pw - 1) * len / step)
      y = row(level[i] == "" ? floor : level[i])
      lo = (prev < 0 || prev > y) ? y : prev
      hi = (prev < 0 || prev < y) ? y : prev
      for (yy = lo; yy <= hi + 1; yy++) for (dx = 0; dx < 2; dx++) put(left + x + dx, yy, 255, 176, 64)
      prev = y
    }
    print "P3"; print w, h; print 255
    for (i = 0; i < w * h; i++) print R[i], G[i], B[i]
  }' "$work/series.txt" > "$work/graph.ppm"

# Labels of the scale
ticks=$(awk -v font="$font" -v left="$left" -v top="$top" -v ph="$plot_height" -v floor="$floor" -v len="$length" -v pw="$plot_width" 'BEGIN {
  for (db = 0; db >= floor; db -= 10)
    printf "drawtext=fontfile=%s:text=%d dB:fontcolor=white:fontsize=16:x=%d-text_w:y=%d-text_h/2,", font, db, left - 6, top + int((0 - db) / (0 - floor) * (ph - 1))
  for (s = 0; s <= len; s += 5)
    printf "drawtext=fontfile=%s:text=%d s:fontcolor=gray:fontsize=14:x=%d-text_w/2:y=%d,", font, s, left + int(s / len * (pw - 1)), top + ph + 6
}')

# Each caption shows until the next, and the last until the end
awk -F'\t' -v screen0="$screen0" -v begin="$begin" -v end="$(stamp end)" '
  $2 == "caption" { time[n] = $1 - screen0 - begin; text[n++] = $3 }
  END { for (i = 0; i < n; i++) printf "%.3f\t%.3f\t%s\n", (time[i] < 0 ? 0 : time[i]), (i + 1 < n ? time[i + 1] : end - screen0 - begin), text[i] }
' "$out/events.tsv" > "$work/captions.tsv"
captions=$(awk -F'\t' -v work="$work" -v font="$font" -v y=$((768 + graph_height + 10)) '{
  file = work "/caption-" NR ".txt"
  printf "%s", $3 > file
  close(file)
  printf ",drawtext=fontfile=%s:textfile=%s:expansion=none:fontcolor=white:fontsize=20:x=(w-text_w)/2:y=%d:enable=%sbetween(t,%s,%s)%s", font, file, y, "\047", $1, $2, "\047"
}' "$work/captions.tsv")

printf "%s" "$title" > "$work/title.txt"

# The graph draws itself: a cover as wide as the plot moves right with the
# time, and a bar marks the moment.  The cover is as dark as the plot.
ffmpeg -nostdin -hide_banner -loglevel error -y \
  -ss "$begin" -t "$length" -i "$out/screen.mp4" \
  -framerate 30 -loop 1 -t "$length" -i "$work/graph.ppm" \
  -f lavfi -i "color=c=0x101014:s=${plot_width}x$((graph_height - top)):r=30:d=$length" \
  -f lavfi -i "color=c=white:s=2x$((graph_height - top)):r=30:d=$length" \
  -i "$work/heard.wav" \
  -filter_complex "
    [0:v]setpts=PTS-STARTPTS,fps=30[screen];
    [1:v]format=yuv420p,${ticks}drawtext=fontfile=$font:textfile=$work/title.txt:expansion=none:fontcolor=white:fontsize=16:x=$left:y=4[base];
    [base][2:v]overlay=x='$left+(W-$left-$right)*t/$length':y=$top:eval=frame[cover];
    [cover][3:v]overlay=x='$left+(W-$left-$right)*t/$length':y=$top:eval=frame[graph];
    [screen]scale=in_range=full:out_range=tv,format=yuv420p[tv];[tv][graph]vstack=inputs=2,pad=iw:ih+$band:0:0:black${captions},format=yuv420p[video]
  " \
  -map '[video]' -map 4:a -t "$length" \
  -c:v libx264 -crf 20 -preset medium -c:a aac -b:a 160k -movflags +faststart "$result"

# Level at the end of each wait: the mean power of the second that ends 0.1 s
# before the stamp, which stays clear of the key that ends the wait
: > "$out/levels.tsv"
awk -F'\t' -v screen0="$screen0" -v begin="$begin" '$2 == "measure" { printf "%.3f\t%s\n", $1 - screen0 - begin, $3 }' \
  "$out/events.tsv" |
  while IFS=$'\t' read -r at what; do
    from=$(minus "$at" 1.1)
    db=$(ffmpeg -nostdin -hide_banner -ss "$from" -t 1 -i "$work/heard.wav" \
      -af "astats=measure_perchannel=none:measure_overall=RMS_level" -f null - 2>&1 |
      awk '/RMS level dB/ { print $NF; exit }')
    printf '%s\t%s\t%s\n' "$at" "$db" "$what" >> "$out/levels.tsv"
  done
# A sine's RMS is 3 dB under its peak, so the peak of the tone is the RMS
# plus 3.01, in dBFS, as the cue plays at 0 dB
{
  echo '| Time in video (s) | RMS level heard (dB) | Tone peak implied (dBFS) | End of the wait |'
  echo '| ---: | ---: | ---: | --- |'
  awk -F'\t' '{ printf "| %.1f | %.1f | %.1f | %s |\n", $1, $2, $2 + 3.01, $3 }' "$out/levels.tsv"
} > "$out/levels.md"
cat "$out/levels.md"
