# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Summarize one run of run.sh: what reached the listener and the stand-in
console, against what show.toml sends.

Usage: analyze.py OUT
"""

import collections
import glob
import json
import struct
import sys

out = sys.argv[1]

# DMX of show.toml after each light cue, by universe: slot -> value
EXPECT = {
    "after cue 1 (20, 50, 100 percent)": {1: {1: 51}, 2: {1: 128}, 999: {1: 255}},
    "after cue 2 (fade to 70, 80, 100, 30)": {1: {1: 179, 2: 204}, 2: {1: 255}, 999: {1: 77}},
    "after cue 3 (50, 40, 100)": {1: {1: 128, 2: 204}, 999: {1: 102, 512: 255}},
}


def lines(pattern):
    return [line for path in sorted(glob.glob(f"{out}/{pattern}")) for line in open(path)]


def packets():
    """Returns (time, protocol, source, universe, kind) of each packet of
    wire.pcap, kind being data or discovery"""
    data = open(f"{out}/wire.pcap", "rb").read()
    at, found = 24, []
    while at + 16 <= len(data):
        seconds, micro, size, _ = struct.unpack("<IIII", data[at : at + 16])
        frame = data[at + 16 : at + 16 + size]
        at += 16 + size
        source = ".".join(str(b) for b in frame[26:30])
        port = struct.unpack(">H", frame[36:38])[0]
        body = frame[42:]
        when = seconds + micro / 1e6
        if port == 6454 and len(body) >= 18:
            found.append((when, "artnet", source, struct.unpack("<H", body[14:16])[0] + 1, "data"))
        elif port == 5568 and len(body) >= 44:
            if struct.unpack(">I", body[18:22])[0] == 8:
                found.append((when, "sacn", source, 64214, "discovery"))
            elif len(body) >= 126:
                found.append((when, "sacn", source, struct.unpack(">H", body[113:115])[0], "data"))
    return found


def marks():
    return [(float(a), b.strip()) for a, b in (line.split(" ", 1) for line in lines("marks.txt"))]


def main():
    mark = dict((name, when) for when, name in marks())
    print("== Light output")
    records = [json.loads(line) for line in lines("dmx.jsonl")]
    joins = [r for r in records if "join" in r]
    print(f"group joins that failed: {len(joins)}", sorted({r["error"] for r in joins}))
    seen = collections.defaultdict(list)
    for r in records:
        if "levels" in r:
            seen[(r["proto"], r["from"], r.get("universe"))].append(r["levels"])
    meta = {(r["proto"], r["from"], r["universe"]): (r["priority"], r["name"]) for r in records if "priority" in r}
    print("sACN sources, universes, priorities, names:", sorted({(k[1], k[2], *v) for k, v in meta.items()}))
    discovery = sorted({(r["from"], tuple(r["discovery"]), r["name"]) for r in records if "discovery" in r})
    print("sACN discovery lists:", discovery)
    senders = sorted({k[1] for k in seen})
    print("senders that carried DMX:", senders)
    for sender in senders:
        for proto in ("sacn", "artnet"):
            for label, universes in EXPECT.items():
                # Art-Net counts universes from 0, sACN from 1
                ok = all(
                    {str(s): v for s, v in expected.items()} in seen.get((proto, sender, u if proto == "sacn" else u - 1), [])
                    for u, expected in universes.items()
                )
                print(f"{sender:>16} {proto:>6} {label}: {'PASS' if ok else 'FAIL'}")
    print("distinct artnet universes seen (Port-Address):", sorted({k[2] for k in seen if k[0] == "artnet"}))
    wire = packets()
    start = mark.get("GO 1")
    if start is None:
        print("no GO 1 mark")
        return
    fade = (mark["GO 2"], mark["GO 2"] + 4)
    idle = (mark["end"] - 12, mark["end"] - 2) if "end" in mark else None
    for proto in ("sacn", "artnet"):
        for sender in senders:
            during = [p for p in wire if p[1] == proto and p[2] == sender and p[3] == 1 and p[4] == "data" and fade[0] <= p[0] < fade[1]]
            quiet = [p for p in wire if idle and p[1] == proto and p[2] == sender and p[3] == 1 and p[4] == "data" and idle[0] <= p[0] < idle[1]]
            print(f"{proto} universe 1 from {sender}: {len(during)} packets in the 4 s fade ({len(during) / 4:.1f}/s), {len(quiet)} in 10 s idle")
    gaps = sorted(p[0] for p in wire if p[4] == "discovery" and p[2] != "127.0.0.1")
    print("discovery times (s since GO 1), non-loopback senders:", [round(g - start, 1) for g in gaps])
    print("== Network cues")
    events = [(float(a), "go", b.strip()) for a, b in (line.split(" ", 1) for line in lines("marks.txt"))]
    events += [(float(l.split(" ", 1)[0]), "console", l.split(" ", 1)[1].strip()) for l in lines("console-*.txt")]
    events += [(float(l.split(" ", 1)[0]), "cinq", l.split(" ", 1)[1].strip()) for l in lines("cinq.err")]
    for when, who, what in sorted(events):
        if when >= start - 1:
            print(f"{when - start:8.3f} {who:8} {what}")


main()
