#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Checks the tutorial that nix build .#doc typeset: it fails on a reference
# that LaTeX left as question marks, or on screenshots other than the ten
# that the tutorial shows at 2560x1920, and lays out the screenshots and the
# pages in contact sheets to look over.
#
# Usage: check-pdf.sh PDF OUT, with poppler's tools and ImageMagick on PATH
set -euo pipefail
pdf=$1 out=$2
mkdir -p "$out/images" "$out/pages"
cp "$pdf" "$out/tutorial.pdf"
pdfinfo "$pdf" | tee "$out/pdfinfo.txt"
pdftotext -layout "$pdf" "$out/tutorial.txt"
pdfimages -list "$pdf" | tee "$out/images.txt"
pdfimages -png "$pdf" "$out/images/shot"
pdftoppm -r 50 -png "$pdf" "$out/pages/page"
montage "$out"/images/shot-*.png -tile 4x3 -geometry 640x480+6+6 -background gray30 "$out/shots.png"
montage "$out"/pages/page-*.png -tile 7x4 -geometry +4+4 -background gray30 "$out/pages.png"

problems=''
if grep -n '??' "$out/tutorial.txt"; then
  problems="$problems undefined-references"
fi
shots=$(awk 'NR > 2 && $3 == "image" && $4 == 2560 && $5 == 1920' "$out/images.txt" | wc -l)
images=$(awk 'NR > 2' "$out/images.txt" | wc -l)
echo "screenshots at 2560x1920: $shots of $images images"
[ "$shots" = 10 ] && [ "$images" = 10 ] || problems="$problems screenshots"
if [ -n "$problems" ]; then
  echo "problems:$problems" >&2
  exit 1
fi
