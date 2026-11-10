# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes the picture and the film of storm.cinq.toml in the directory SHOW,
# with ffmpeg and the font file FONT.  The sounds come from the demo branch
# of cinq.
#
# Usage: media.sh SHOW FONT
set -euo pipefail
show=$1 font=$2
mkdir -p "$show/pictures" "$show/video"
make() { ffmpeg -loglevel error -y "$@"; }
text() {
  printf "drawtext=fontfile=%s:text='%s':fontcolor=white:fontsize=%s:x=(w-text_w)/2:y=%s" "$font" "$1" "$2" "$3"
}
make -f lavfi -i "gradients=s=1280x720:c0=0x1b2a4a:c1=0x6a2c70:x0=0:y0=0:x1=1280:y1=720,$(text 'The Storm' 112 '(h/2)-120'),$(text 'Act 2' 56 '(h/2)+40')" -frames:v 1 "$show/pictures/title.png"
make -f lavfi -i "mandelbrot=size=1280x720:rate=30" -f lavfi -i "sine=frequency=330:sample_rate=48000" -t 20 -c:v libx264 -profile:v high -pix_fmt yuv420p -c:a aac -ac 2 "$show/video/film.mp4"
