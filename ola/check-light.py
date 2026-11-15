# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Check what listen.py heard of gateway.toml, run by linux.sh with five GOs.

Usage: check-light.py OUT

Prints one line per check, ok or WRONG, and exits 1 if one is wrong.
"""

import json
import sys

out = sys.argv[1]
heard = [json.loads(line) for line in open(f"{out}/dmx.jsonl")]
marks = [line.split(" ", 1) for line in open(f"{out}/marks.txt").read().splitlines()]
wrong = False


def check(good, text):
    global wrong
    wrong |= not good
    print(f"{'ok' if good else 'WRONG'}: {text}")


data = [h for h in heard if "levels" in h and "universe" in h]
# Levels by protocol and universe in the order that they changed
seen = {}
for h in data:
    seen.setdefault((h["proto"], h["universe"], h["from"]), []).append(h["levels"])
want = {
    1: [{"1": 77, "3": 51, "5": 255}, {"1": 179, "3": 51, "5": 255}, {"1": 77, "3": 51, "5": 255}, {"1": 179, "3": 51, "5": 255}, {}],
    2: [{"1": 51, "2": 179}, {}],
    999: [{"1": 128}, {}],
}
for proto, shift in (("sacn", 0), ("artnet", -1)):
    for universe, steps in want.items():
        # Art-Net counts from 0: universe 1 is its Port-Address 0
        # Each sender apart: a network, or the loopback address.  The fade
        # passes through values between the steps, so look for each step in order
        for sender in sorted({key[2] for key in seen if key[:2] == (proto, universe + shift)}):
            levels = seen[(proto, universe + shift, sender)]
            index = 0
            for level in levels:
                if index < len(steps) and level == steps[index]:
                    index += 1
            check(index == len(steps), f"{proto} universe {universe + shift} from {sender}: {index} of {len(steps)} steps in order, {len(levels)} changes, last {levels[-1:]}")
        if not any(key[:2] == (proto, universe + shift) for key in seen):
            check(False, f"{proto} universe {universe + shift}: nothing heard")
# Sources: names and priorities of sACN, senders by protocol
names = {(h["name"], h["priority"]) for h in data if h["proto"] == "sacn"}
check(names == {("cinq", 100)}, f"sacn sources (name, priority): {sorted(names)}")
senders = {}
for h in data:
    senders.setdefault(h["proto"], set()).add(h["from"])
print("info: senders", {k: sorted(v) for k, v in senders.items()})
lengths = {(h["proto"], h["length"]) for h in data}
print("info: slot counts", sorted(lengths))
listed = [h["discovery"] for h in heard if "discovery" in h]
check([1, 2, 999] in listed, f"sacn discovery lists {listed[:1]} ({len(listed)} packets)")
ended = [h for h in heard if "terminated" in h]
print("info: sacn packets with the stream terminated option:", ended or "none")
for h in heard:
    if "join" in h:
        print("info: join", h)
# Packets per second of each source and universe, one count per summary
# line of the listener, a second apart
rows = [h for h in heard if "counts" in h]
idents = sorted({tuple(c[:4]) for row in rows for c in row["counts"]}, key=str)
for proto, sender, universe, priority in idents:
    series = [next((c[4] for c in row["counts"] if tuple(c[:4]) == (proto, sender, universe, priority)), 0) for row in rows]
    print(f"info: packets per second, {proto} from {sender} universe {universe} priority {priority}: {series}")
sys.exit(wrong)
