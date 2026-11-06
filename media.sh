# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the pictures and the film of pictures.cinq.toml in the directory
# show, with ffmpeg and the font file $FONT
set -euo pipefail
text() {
  printf "drawtext=fontfile=%s:text='%s':fontcolor=white:fontsize=%s:x=(w-text_w)/2:y=%s" "$FONT" "$1" "$2" "$3"
}
ffmpeg -loglevel error -y -f lavfi -i "gradients=s=1280x720:c0=0x1b2a4a:c1=0x6a2c70:x0=0:y0=0:x1=1280:y1=720,$(text 'Welcome' 96 '(h/2)-120'),$(text 'The show begins shortly' 48 '(h/2)+20')" -frames:v 1 show/preshow.png
ffmpeg -loglevel error -y -f lavfi -i "color=c=0x203040:s=1280x720,$(text 'Intermission' 96 '(h-text_h)/2')" -frames:v 1 -q:v 3 show/intermission.jpg
ffmpeg -loglevel error -y -f lavfi -i "mandelbrot=size=1280x720:rate=30,$(text 'Film' 64 '40')" -f lavfi -i sine=frequency=330 -t 8 -c:v libx264 -profile:v high -pix_fmt yuv420p -c:a aac show/film.mp4
