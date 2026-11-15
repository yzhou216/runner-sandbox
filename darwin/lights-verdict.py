# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Check what listen.py heard of lights.toml, and what went out before and
after Allow.

Usage: lights-verdict.py OUT
"""

import collections
import json
import sys

out = sys.argv[1]
marks = []
for line in open(f"{out}/marks.txt"):
    stamp, _, rest = line.rstrip("\n").partition(" ")
    marks.append((float(stamp), rest))


def at(text):
    return next((s for s, r in marks if r.startswith(text)), None)


heard = [json.loads(line) for line in open(f"{out}/dmx.jsonl")]
packets = [json.loads(line) for line in open(f"{out}/dmx.jsonl.packets")]
levels = [h for h in heard if "levels" in h]
wrong = False


def check(good, text):
    global wrong
    wrong |= not good
    print(f"{'ok' if good else 'WRONG'}: {text}")


def between(start, end):
    return [p for p in packets if start <= p["ts"] < end]


def count(window, how):
    return dict(collections.Counter(how(p) for p in window))


print("listener joined:", [h["listening"] for h in heard if "listening" in h], "join errors:", sum("join" in h for h in heard), "(EADDRINUSE: the default network joined the group already)")
# sACN counts universes from 1 and Art-Net's Port-Address from 0
patch = {1: [{"1": 51}, {"1": 179, "2": 128}, {}], 2: [{"1": 128}, {}], 999: [{"5": 255}, {}]}
for proto, shift in (("sacn", 0), ("artnet", 1)):
    for universe, wanted in patch.items():
        seq = []
        for h in levels:
            if h["proto"] == proto and h.get("universe") == universe - shift and (not seq or seq[-1] != h["levels"]):
                seq.append(h["levels"])
        check(all(w in seq for w in wanted) and seq[-1:] == [{}], f"{proto} universe {universe - shift} (patch universe {universe}): {len(seq)} changes, wanted {wanted}, last {seq[-2:]}")
sacn = {(h["priority"], h["name"]) for h in levels if h["proto"] == "sacn" and "priority" in h}
check(sacn == {(100, "cinq")}, f"sACN priority and source name: {sorted(sacn)}")
senders = {proto: sorted({p["from"] for p in packets if p["proto"] == proto}) for proto in ("sacn", "artnet")}
print("senders heard (interfaces that carried packets):", senders)
listed = [p["discovery"] for p in packets if p.get("discovery") is not None]
check([1, 2, 999] in listed, f"sACN discovery lists the universes: {sorted({tuple(x) for x in listed})}, {len(listed)} packets")

# Before and after Allow
first = at("GO 1")
presses = [s for s, r in marks if r.startswith("pressed Allow (cinq")]
prompt = bool(presses)
print(f"Local Network prompts for cinq.app: {len(presses)} Allow presses, at +%s s after GO 1" % [round(p - first, 1) for p in presses] if prompt and first else "Local Network prompts for cinq.app: none appeared")
if first is not None:
    split = presses[0] if prompt else first + 4
    last = presses[-1] if prompt else split
    way = lambda p: f"{p["proto"]} from {p["from"]}"
    print("before the first Allow, from GO 1:", count(between(first, split), way))
    print("between the first and the last Allow:", count(between(split, last), way))
    print("the 5 s after the last Allow:", count(between(last, last + 5), way))
    print("packets by protocol, sender, and universe in the 13 s idle stretch:")
    start, end = at("idle start"), at("idle end")
    window = between(start + 1, end)
    for key, n in sorted(count(window, lambda p: (p["proto"], p["from"], p["universe"], str(p.get("discovery")))).items(), key=str):
        print(f"  {key}: {n} in {end - start - 1:.0f} s")
    fade = at("GO 2")
    inside = between(fade, fade + 4.5)
    print("fade of 4 s, packets in 4.5 s from GO 2:")
    for key, n in sorted(count(inside, lambda p: (p["proto"], p["from"], p["universe"])).items(), key=str):
        print(f"  {key}: {n} ({n / 4.5:.1f} per second)")
sys.exit(1 if wrong else 0)
