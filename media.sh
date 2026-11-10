# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the sounds, the picture, and the film of storm.cinq.toml in the
# directory SHOW, with ffmpeg and the font file FONT
#
# Usage: media.sh SHOW FONT
set -euo pipefail
show=$1 font=$2
mkdir -p "$show/sounds" "$show/pictures" "$show/video"
make() { ffmpeg -loglevel error -y "$@"; }
text() {
  printf "drawtext=fontfile=%s:text='%s':fontcolor=white:fontsize=%s:x=(w-text_w)/2:y=%s" "$font" "$1" "$2" "$3"
}
make -f lavfi -i "aevalsrc='0.15*sin(2*PI*220*t)+0.12*sin(2*PI*277.18*t)+0.1*sin(2*PI*329.63*t)':s=48000:d=94" -ac 2 "$show/sounds/preshow.ogg"
make -f lavfi -i "anoisesrc=color=brown:a=0.8:d=6,afade=t=in:d=0.1,afade=t=out:st=1:d=5" -ac 2 "$show/sounds/thunder.wav"
make -f lavfi -i "anoisesrc=color=pink:a=0.3:d=30" -ac 2 "$show/sounds/rain.flac"
make -f lavfi -i "aevalsrc='0.5*sin(2*PI*880*t)*exp(-2*t)':s=48000:d=3" -ac 2 "$show/sounds/bell.mp3"
make -f lavfi -i "aevalsrc='0.8*sin(2*PI*90*t)*exp(-30*mod(t,0.4))':s=48000:d=1.2" -ac 2 "$show/sounds/knock.wav"
make -f lavfi -i "gradients=s=1280x720:c0=0x1b2a4a:c1=0x6a2c70:x0=0:y0=0:x1=1280:y1=720,$(text 'The Storm' 112 '(h/2)-120'),$(text 'Act 2' 56 '(h/2)+40')" -frames:v 1 "$show/pictures/title.png"
make -f lavfi -i "mandelbrot=size=1280x720:rate=30" -f lavfi -i "sine=frequency=330:sample_rate=48000" -t 20 -c:v libx264 -profile:v high -pix_fmt yuv420p -c:a aac -ac 2 "$show/video/film.mp4"
