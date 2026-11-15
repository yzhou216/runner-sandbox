# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Reads wire.pcap and marks.txt of a cues-* artifact and prints what cinq sent
# Usage: analyze.py DIR
import collections
import socket
import struct
import sys

d = sys.argv[1]
marks = []
for line in open(f"{d}/marks.txt", encoding="utf-8-sig", errors="replace"):
    t, _, text = line.strip().partition(" ")
    try:
        marks.append((float(t), text))
    except ValueError:
        pass


def mark(prefix):
    return next((t for t, m in marks if m.startswith(prefix)), None)


packets = []
data = open(f"{d}/wire.pcap", "rb").read()
off = 24
while off + 16 <= len(data):
    sec, usec, incl, _ = struct.unpack("<IIII", data[off : off + 16])
    frame = data[off + 16 : off + 16 + incl]
    off += 16 + incl
    src = socket.inet_ntoa(frame[26:30])
    port = struct.unpack(">H", frame[36:38])[0]
    body = frame[42:]
    t = sec + usec / 1e6
    if port == 6454 and body[:8] == b"Art-Net\0":
        op = struct.unpack("<H", body[8:10])[0]
        if op == 0x5000:
            n = struct.unpack(">H", body[16:18])[0]
            packets.append((t, src, "artnet", struct.unpack("<H", body[14:16])[0], None, body[18 : 18 + n], body[12]))
        else:
            packets.append((t, src, f"artnet-op{op:x}", None, None, b"", 0))
    elif port == 5568 and body[4:16] == b"ASC-E1.17\0\0\0":
        root, framing = struct.unpack(">I", body[18:22])[0], struct.unpack(">I", body[40:44])[0]
        name = body[44:108].split(b"\0")[0].decode()
        if (root, framing) == (8, 2):
            n = (len(body) - 120) // 2
            listed = struct.unpack(f">{n}H", body[120 : 120 + 2 * n])
            packets.append((t, src, "sacn-discovery", listed, name, b"", 0))
        else:
            uni = struct.unpack(">H", body[113:115])[0]
            n = struct.unpack(">H", body[123:125])[0]
            packets.append((t, src, "sacn", uni, (body[108], name), body[126 : 125 + n], body[111]))
print(f"{len(packets)} packets, {len(marks)} marks")
senders = collections.Counter((p[2], p[1], p[3] if p[2] != "sacn-discovery" else p[3]) for p in packets)
for k, v in sorted(senders.items(), key=str):
    print("  ", k, v)

lights = mark("phase lights")
go1, go2, go3 = (mark(f"GO light {i}") for i in (1, 2, 3))
stopped = mark("cinq stopped")
print("marks", lights, go1, go2, go3, stopped)


def window(a, b, kinds=("sacn", "artnet")):
    c = collections.Counter((p[2], p[1], p[3]) for p in packets if a <= p[0] < b and p[2] in kinds)
    return c


def show(title, a, b):
    print(f"-- {title}: {b - a:.1f} s")
    for k, v in sorted(window(a, b).items(), key=str):
        print(f"   {k[0]:7} from {k[1]:15} universe {k[2]:>4}: {v:4d} packets, {v / (b - a):.2f}/s")


if go1:
    show("before GO 1", lights, go1)
    show("idle after GO 1", go1 + 2, go2)
    show("fade (GO 2 + 0.2 to +4.0)", go2 + 0.2, go2 + 4.0)
    show("idle after fade", go2 + 6, go3)
    show("idle after blackout", go3 + 2, stopped)
    if stopped:
        print("-- after cinq stopped, 5 to 12 s:", dict(window(stopped + 1, stopped + 12)))
    for a, b, title in ((go1 + 1, go2, "after GO 1"), (go2 + 6, go3, "after GO 2"), (go3 + 2, stopped, "after GO 3")):
        last = {}
        for p in packets:
            if a <= p[0] < b and p[2] in ("sacn", "artnet"):
                last[(p[2], p[1], p[3])] = p
        print(f"-- last packet of each stream {title}")
        for k, p in sorted(last.items(), key=str):
            nz = {i + 1: v for i, v in enumerate(p[5]) if v}
            print(f"   [{k[0]} {k[1]} u{k[2]}] len {len(p[5])} prio/name {p[4]} nonzero {nz}")
    disc = [p for p in packets if p[2] == "sacn-discovery"]
    times = [p[0] for p in disc]
    print("-- discovery:", len(disc), "packets; sources", collections.Counter((p[1], p[4], p[3]) for p in disc))
    print("   gaps", [round(b - a, 2) for a, b in zip(times, times[1:])][:12])
    # Fade packet times for universe 1
    for kind, src in sorted({(p[2], p[1]) for p in packets if p[2] in ("sacn", "artnet") and p[3] == 1}):
        ts = [p[0] - go2 for p in packets if p[2] == kind and p[1] == src and p[3] == 1 and go2 - 0.5 <= p[0] <= go2 + 5]
        gaps = [b - a for a, b in zip(ts, ts[1:])]
        if gaps:
            print(f"-- fade timing {kind} {src} u1: first {ts[0]:.3f} last {ts[-1]:.3f} n {len(ts)} gap min {min(gaps)*1000:.0f} ms median {sorted(gaps)[len(gaps)//2]*1000:.0f} ms max {max(gaps)*1000:.0f} ms")
    # Level at points in the fade for address 1 of universe 1
    prog = [(p[0] - go2, p[5][0]) for p in packets if p[2] == "sacn" and p[3] == 1 and p[1] == packets[0][1] and go2 - 0.2 <= p[0] <= go2 + 5]
    print("-- sacn u1 slot 1 during fade (t since GO2, value), every 8th:", [(round(t, 2), v) for t, v in prog[::8]])
