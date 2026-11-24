#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Reads the text that `ui list` printed (text, x, y per line) in the sidebar's audio device row.

Usage: pick.py label FILE   prints the hint label of the Audio device list, or nothing
       pick.py item FILE    prints the x and y of the first line below the Default device entry
"""
import re
import sys

rows = []
for line in open(sys.argv[2], errors="replace"):
    parts = line.rstrip("\n").split("\t")
    if len(parts) == 3 and parts[1].isdigit() and parts[2].isdigit():
        rows.append((parts[0], int(parts[1]), int(parts[2])))
device = [r for r in rows if re.search(r"ystem|ystern", r[0]) and r[1] > 700] or [r for r in rows if re.search(r"efault|device", r[0], re.I) and r[1] > 700]
if not device:
    sys.exit(0)
anchor = min(device, key=lambda r: r[2])
if sys.argv[1] == "label":
    near = [r for r in rows if abs(r[2] - anchor[2]) <= 14 and re.fullmatch(r"\W*[a-z]{2}", r[0])]
    if near:
        print(re.sub(r"\W", "", near[0][0]))
else:
    named = [r for r in rows if re.search(r"Apple|Null|Virtual", r[0]) and r[1] > 700]
    if named:
        print(named[0][1], named[0][2])
        sys.exit(0)
    below = [r for r in rows if 12 < r[2] - anchor[2] < 80 and abs(r[1] - anchor[1]) < 90 and not re.search(r"efault", r[0], re.I)]
    if below:
        first = min(below, key=lambda r: r[2])
        print(first[1], first[2])
