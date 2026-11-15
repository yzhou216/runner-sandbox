# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Check the cues of net.toml against what the stand-in console heard.

Usage: net-verdict.py OUT
"""

import re
import sys

out = sys.argv[1]
gos = []
for line in open(f"{out}/marks.txt"):
    stamp, _, rest = line.rstrip("\n").partition(" ")
    if rest.startswith("GO "):
        gos.append((float(stamp), rest[3:]))
events = []
for line in open(f"{out}/console.txt"):
    stamp, _, rest = line.rstrip("\n").partition(" ")
    events.append((float(stamp), rest))
go = dict((name, when) for when, name in gos)
print("everything the stand-in console logged:")
for stamp, rest in events:
    print(f"  {stamp:.3f} {rest}")


def arrival(how, address, first=None):
    for stamp, rest in events:
        if rest.startswith(f"{how} {address} ") and (first is None or f"[{first}" in rest or f"'{first}'" in rest):
            return stamp
    return None


expected = {
    "tcp": ("tcp", "/test/tcp"),
    "udp": ("udp", "/test/udp"),
    "burst-1": ("tcp", "/test/burst", 1),
    "burst-2": ("tcp", "/test/burst", 2),
    "burst-3": ("tcp", "/test/burst", 3),
    "apart-1": ("tcp", "/test/apart", 1),
    "apart-2": ("tcp", "/test/apart", 2),
    "apart-3": ("tcp", "/test/apart", 3),
    "back": ("tcp", "/test/back"),
}
wrong = False
for name, (how, address, *first) in expected.items():
    stamp = arrival(how, address, *first)
    ok = stamp is not None and name in go and stamp - go[name] < 1
    wrong |= not ok
    shown = "never" if stamp is None else f"{(stamp - go.get(name, 0)) * 1000:.0f} ms after GO"
    print(f"{'ok' if ok else 'WRONG'}: {name}: {shown}")
for name in ("refused", "down"):
    stamp = arrival("tcp", f"/test/{name}")
    ok = stamp is None
    wrong |= not ok
    print(f"{'ok' if ok else 'WRONG'}: {name}: {'never arrived' if ok else 'arrived'}")


def connections(start, end):
    return sum(1 for stamp, rest in events if start <= stamp < end and rest.startswith("tcp connected"))


burst = connections(go["burst-1"] - 0.1, go["apart-1"] - 0.1)
print(f"{'ok' if burst == 1 else 'WRONG'}: burst of 3 cues within 0.6 s: {burst} connection(s)")
wrong |= burst != 1
order = ["apart-1", "apart-2", "apart-3"]
bounds = [go[n] - 0.1 for n in order] + [go["down"] - 0.1]
for name, start, end in zip(order, bounds, bounds[1:]):
    count = connections(start, end)
    print(f"{'ok' if count == 1 else 'WRONG'}: {name}: {count} connection(s)")
    wrong |= count != 1
print("warnings on standard error, seconds after the last GO:")
try:
    stamped = [line.rstrip("\n") for line in open(f"{out}/cinq.log.stamped")]
except OSError:
    stamped = []
for line in stamped:
    stamp, _, text = line.partition(" ")
    last = max((g for g in gos if g[0] <= float(stamp)), default=None, key=lambda g: g[0])
    print(f"  +{float(stamp) - last[0]:.3f} s after GO {last[1]}: {text}" if last else f"  {line}")
sys.exit(1 if wrong else 0)
