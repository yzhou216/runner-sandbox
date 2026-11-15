# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

import json, sys
last = {}
rows = []
for l in open(sys.argv[1]):
    d = json.loads(l)
    if d.get("proto") == "sacn" and d.get("from") == "10.1.0.49" and d.get("start") == 0:
        rows.append((d["t"], d["levels"]))
prev = None
for t, lv in rows:
    if lv != prev:
        print(round(t, 2), lv)
    prev = lv
