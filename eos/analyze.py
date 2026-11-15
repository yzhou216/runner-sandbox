# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Summarize a run of play.sh: DMX values, refresh and fade packet rates,
discovery, senders, the arrival of the network cues, and the warnings.

Usage: analyze.py OUT_DIR
"""

import json
import struct
import sys
from collections import defaultdict
from pathlib import Path

out = Path(sys.argv[1])
here = Path(__file__).parent
# Packet decoders of the listener, without its main
source = (here / "listen.py").read_text().split("def main():")[0]
exec(source)  # noqa: S102

marks = {}
for line in (out / "marks.txt").read_text().splitlines():
    stamp, _, what = line.partition(" ")
    marks[what] = float(stamp)

packets = []  # (time, proto, sender, universe, priority, name, levels, discovery)
data = (out / "wire.pcap").read_bytes()[24:]
while len(data) >= 16:
    sec, usec, caplen, _ = struct.unpack("<IIII", data[:16])
    frame, data = data[16 : 16 + caplen], data[16 + caplen :]
    sender = ".".join(str(b) for b in frame[26:30])
    port = struct.unpack(">H", frame[14 + 20 + 2 : 14 + 20 + 4])[0]
    body = frame[14 + 20 + 8 :]
    packet = (artnet if port == ARTNET else sacn)(body)
    if packet is None:
        continue
    slots = packet.get("slots", b"")
    packets.append(
        (
            sec + usec / 1e6,
            "artnet" if port == ARTNET else "sacn",
            sender,
            packet.get("universe"),
            packet.get("priority"),
            packet.get("name"),
            slots,
            packet.get("discovery"),
        )
    )

lit = [p for p in packets if any(p[6])]
t0 = lit[0][0]
print(f"first lit packet {t0:.3f}; GO mark {marks['GO']:.3f} (lit packet {t0 - marks['GO']:+.2f} s after the mark)")
print(f"packets {len(packets)}; senders {sorted({p[2] for p in packets})}")
print("kinds:", sorted({(p[1], p[3], p[4], p[5]) for p in packets if p[7] is None}, key=str))

print("\n== DMX values by universe (time since first light)")
shown = defaultdict(list)
for t, proto, sender, universe, priority, name, slots, discovery in packets:
    if discovery is not None:
        continue
    levels = {i + 1: v for i, v in enumerate(slots) if v}
    key = (proto, universe)
    if not shown[key] or shown[key][-1][1] != levels:
        shown[key].append((t - t0, levels))
for key in sorted(shown, key=str):
    steps = shown[key]
    final = steps[-1][1]
    print(f"{key}: {len(steps)} distinct states; first lit {next((s for s in steps if s[1]), (None,))[0]:.2f} s, final {final}")
    ends = [t for t, _ in steps[1:]] + [packets[-1][0] - t0]
    for (t, levels), end in zip(steps, ends):
        if end - t >= 1.0 and levels:
            print(f"   held {end - t:5.1f} s from t+{t:7.2f}: {levels}")

print("\n== Which sender addresses carried each protocol")
carried = defaultdict(int)
for p in packets:
    carried[(p[1], p[2])] += 1
for key, n in sorted(carried.items()):
    print(f"{key}: {n} packets")

print("\n== Packet rate per second (sACN universe 1, one sender)")


def rate(lo, hi, proto="sacn", universe=1, sender=None):
    sel = [p for p in packets if p[1] == proto and p[3] == universe and p[7] is None and lo <= p[0] - t0 < hi and (sender is None or p[2] == sender)]
    return len(sel) / (hi - lo)


senders = sorted({p[2] for p in packets if p[1] == "sacn"})
for sender in senders:
    print(f"sender {sender}")
    for label, lo, hi in [("idle after cue 1", 2.0, 4.5), ("level cue 2 to cue 3", 6.5, 8.5), ("fade of cue 3 (4 s)", 9.2, 12.8), ("idle after the fade", 14.0, 15.5)]:
        print(f"  {label:24} t+{lo}..{hi}: sACN u1 {rate(lo, hi, sender=sender):6.1f}/s, Art-Net port 0 {rate(lo, hi, 'artnet', 0, sender):6.1f}/s")

print("\n== Universe discovery")
for t, proto, sender, universe, priority, name, slots, discovery in packets:
    if discovery is not None:
        print(f"t+{t - t0:7.2f} from {sender} {name} lists {discovery}")

print("\n== Console (stand-in) log, time since first light")
for name in ("console-a.txt", "console-b.txt"):
    path = out / name
    if not path.exists():
        continue
    print(f"-- {name}")
    for line in path.read_text().splitlines():
        stamp, _, rest = line.partition(" ")
        print(f"t+{float(stamp) - t0:7.2f} {rest}")

print("\n== cinq's standard error")
for line in (out / "cinq.log").read_text().splitlines():
    stamp, _, rest = line.partition(" ")
    print(f"t+{float(stamp) - t0:7.2f} {rest}")
print(f"\nmarks since first light: { {k: round(v - t0, 2) for k, v in marks.items()} }")
