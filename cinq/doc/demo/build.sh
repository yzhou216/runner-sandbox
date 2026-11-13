#!/bin/sh
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# Builds the demo shows in DIRECTORY: copies the shows beside this script
# there, and makes the sounds, the picture, and the film that they play
# from ffmpeg's lavfi sources, drawing text with the font file FONT, such
# as DejaVu Sans.  ffmpeg needs libvorbis, libmp3lame, libx264, and
# drawtext.  A given ffmpeg on a given platform makes the same files on
# every run: noise has a seed, as does the gradient, x264 a fixed number of
# threads, which it would take from the processor's cores, and the Ogg
# muxer writes bit-exact, where it would pick a random serial number.
#
# Usage: build.sh DIRECTORY FONT
set -eu
out=$1 font=$2
mkdir -p "$out/sounds" "$out/pictures" "$out/video"
cp "$(dirname "$0")"/*.cinq.toml "$out"
encode() { ffmpeg -nostdin -hide_banner -loglevel error -y "$@"; }
text() {
  printf "drawtext=fontfile=%s:text='%s':fontcolor=white:fontsize=%s:x=(w-text_w)/2:y=%s" \
    "$font" "$1" "$2" "$3"
}

encode -f lavfi \
  -i "aevalsrc='0.18*(sin(2*PI*220*t)+sin(2*PI*275*t)+sin(2*PI*330*t))*(0.8+0.2*sin(2*PI*0.25*t))':s=44100:d=12" \
  -c:a libvorbis -q:a 3 -fflags +bitexact "$out/sounds/preshow.ogg"
encode -f lavfi -i "anoisesrc=color=pink:amplitude=0.35:sample_rate=44100:duration=8:seed=3" \
  -af "highpass=f=500,lowpass=f=7000,volume=3" -c:a flac "$out/sounds/rain.flac"
encode -f lavfi -i "anoisesrc=color=brown:amplitude=0.9:sample_rate=44100:duration=5:seed=7" \
  -af "lowpass=f=250,afade=t=in:d=0.04,afade=t=out:st=0.4:d=4.6,volume=1.4" -c:a pcm_s16le "$out/sounds/thunder.wav"
encode -f lavfi \
  -i "aevalsrc='0.45*exp(-1.1*t)*sin(2*PI*523.25*t)+0.25*exp(-1.7*t)*sin(2*PI*1046.5*t)+0.15*exp(-2.4*t)*sin(2*PI*1413*t)':s=44100:d=4" \
  -c:a libmp3lame -b:a 128k "$out/sounds/bell.mp3"
encode -f lavfi \
  -i "aevalsrc='0.9*exp(-35*t)*sin(2*PI*110*t)+0.9*gte(t,0.22)*exp(-35*(t-0.22))*sin(2*PI*110*(t-0.22))':s=44100:d=0.7" \
  -c:a pcm_s16le "$out/sounds/knock.wav"
encode -f lavfi \
  -i "gradients=s=1280x720:seed=1:c0=0x1b2a4a:c1=0x6a2c70:x0=0:y0=0:x1=1280:y1=720,$(text 'The Storm' 112 '(h/2)-120'),$(text 'Act 2' 56 '(h/2)+40')" \
  -frames:v 1 "$out/pictures/title.png"
encode -f lavfi -i "mandelbrot=size=1280x720:rate=30" -f lavfi -i "sine=frequency=330:sample_rate=48000" -t 20 \
  -c:v libx264 -profile:v high -crf 34 -preset slow -threads 4 -pix_fmt yuv420p -c:a aac -ac 2 "$out/video/film.mp4"
