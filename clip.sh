#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Makes a short H.264 clip with two chapters, whose chapter track is a text
# track, and damages that track: the stbl box of the last track of the file
# becomes a skip box, so that mp4-atom cannot decode the track while the
# picture track stays readable.
#
# Usage: clip.sh OUT, with ffmpeg, grep, dd, and tail on PATH
set -euo pipefail

out=$1
chapters=$(mktemp)
cat > "$chapters" << 'EOT'
;FFMETADATA1

[CHAPTER]
TIMEBASE=1/1000
START=0
END=3000
title=Opening

[CHAPTER]
TIMEBASE=1/1000
START=3000
END=6000
title=Closing
EOT
ffmpeg -loglevel error -y -f lavfi -i 'testsrc2=size=640x360:rate=30:duration=6' \
  -i "$chapters" -map 0:v -map_metadata 1 -map_chapters 1 \
  -c:v libx264 -profile:v high -pix_fmt yuv420p -g 30 -crf 28 "$out"
rm "$chapters"
offset=$(grep -abo stbl "$out" | tail -1 | cut -d: -f1)
printf skip | dd of="$out" bs=1 seek="$offset" conv=notrunc status=none
