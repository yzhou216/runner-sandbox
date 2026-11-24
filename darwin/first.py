#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Finds the first frame of a burst after an event that shows, or lacks, a text.

Usage: first.py DIR REGEX present|absent EVENT_TIME
"""
import pathlib
import re
import sys

directory = pathlib.Path(sys.argv[1])
pattern = re.compile(sys.argv[2], re.I)
wanted = sys.argv[3] == "present"
event = float(sys.argv[4])
frames = []
for line in (directory / "stamps.txt").read_text().splitlines():
    index, before, after = line.split()
    text = (directory / f"f{index}.txt").read_text()
    frames.append((index, float(before), float(after), bool(pattern.search(text))))
last = None
for index, before, after, present in frames:
    if after <= event:
        continue
    if present == wanted:
        gap = f"; the frame before, {last[0]}, was taken at +{last[2] - event:.2f} s and did not" if last else ""
        print(f"{sys.argv[2]!r} {sys.argv[3]} in frame {index}: taken +{before - event:.2f} to +{after - event:.2f} s after the event{gap}")
        sys.exit(0)
    last = (index, before, after)
print(f"{sys.argv[2]!r} never {sys.argv[3]} after the event, in {len(frames)} frames")
