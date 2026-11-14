# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Log the DMX that arrives as Art-Net or sACN, one JSON line per change.

Usage: listen.py OUT.jsonl [SECONDS [OUT.pcap]]

Each line holds the time since start, the protocol, the sender, the
universe, the sACN priority, and the slots that are not zero.  A summary
line every five seconds counts the packets of each source and universe, so
that refresh rates and dropouts show too.
"""

import json
import selectors
import socket
import struct
import sys
import time

ARTNET, SACN = 6454, 5568
# Groups of the universes that the tests use, and of universe discovery,
# fewer than the 20 groups that GNU/Linux lets a socket join by default
UNIVERSES = list(range(1, 17)) + [300, 999]
GROUPS = [f"239.255.{u >> 8}.{u & 0xFF}" for u in UNIVERSES] + ["239.255.250.214"]


def addresses():
    """Returns the addresses of the networks to join the groups of sACN on:
    the computer's own, by its name, and the default network, 0.0.0.0,
    where its name says nothing"""
    names = {"127.0.0.1", "0.0.0.0"}
    try:
        names |= set(socket.gethostbyname_ex(socket.gethostname())[2])
    except OSError:
        pass
    return sorted(names)


def bound(port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM, socket.IPPROTO_UDP)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    sock.bind(("", port))
    return sock


class Pcap:
    """Every packet heard, behind made-up Ethernet, IPv4, and UDP headers,
    for Wireshark's dissectors to check"""

    def __init__(self, path):
        self.file = open(path, "wb")
        self.file.write(struct.pack("<IHHiIII", 0xA1B2C3D4, 2, 4, 0, 0, 65535, 1))

    def write(self, data, sender, port):
        source = socket.inet_aton(sender[0])
        target = socket.inet_aton("255.255.255.255")
        udp = struct.pack(">HHHH", sender[1], port, 8 + len(data), 0) + data
        ip = struct.pack(">BBHHHBBH4s4s", 0x45, 0, 20 + len(udp), 0, 0, 64, 17, 0, source, target)
        frame = b"\xff" * 6 + b"\x02" + b"\0" * 5 + b"\x08\x00" + ip + udp
        now = time.time()
        self.file.write(struct.pack("<IIII", int(now), int(now % 1 * 1e6), len(frame), len(frame)) + frame)
        self.file.flush()


def artnet(data):
    if data[:8] != b"Art-Net\0" or struct.unpack("<H", data[8:10])[0] != 0x5000:
        return None
    universe = struct.unpack("<H", data[14:16])[0]
    length = struct.unpack(">H", data[16:18])[0]
    return {"universe": universe, "seq": data[12], "slots": data[18 : 18 + length]}


def sacn(data):
    # Root layer ACN identifier, then framing and DMP layers of E1.31
    if data[4:16] != b"ASC-E1.17\0\0\0" or len(data) < 126:
        return None
    # Root vector 4 and framing vector 2 mark data; 8 marks sync or discovery
    root, framing = struct.unpack(">I", data[18:22])[0], struct.unpack(">I", data[40:44])[0]
    source = data[44:108].split(b"\0")[0].decode(errors="replace")
    if (root, framing) == (8, 2):
        # Universe discovery: page, last page, then the universes listed
        count = (len(data) - 120) // 2
        listed = list(struct.unpack(f">{count}H", data[120 : 120 + 2 * count]))
        return {"discovery": listed, "page": data[118], "last": data[119], "name": source}
    if (root, framing) != (4, 2):
        return {"root": root, "framing": framing}
    priority, seq, options = data[108], data[111], data[112]
    universe = struct.unpack(">H", data[113:115])[0]
    count = struct.unpack(">H", data[123:125])[0]
    start = data[125]
    return {
        "universe": universe,
        "priority": priority,
        "seq": seq,
        "options": options,
        "name": source,
        "start": start,
        "slots": data[126 : 125 + count],
    }


def main():
    out = open(sys.argv[1], "a", buffering=1)
    stop = time.monotonic() + float(sys.argv[2]) if len(sys.argv) > 2 else None
    pcap = Pcap(sys.argv[3]) if len(sys.argv) > 3 else None
    begin = time.monotonic()
    sel = selectors.DefaultSelector()
    sel.register(bound(ARTNET), selectors.EVENT_READ, ("artnet", artnet))
    multicast = bound(SACN)
    for group in GROUPS:
        # Loopback takes no joins: senders reach it by unicast, and its
        # joins would count against the limit of the real networks
        for local in (local for local in addresses() if not local.startswith("127.")):
            try:
                multicast.setsockopt(
                    socket.IPPROTO_IP,
                    socket.IP_ADD_MEMBERSHIP,
                    socket.inet_aton(group) + socket.inet_aton(local),
                )
            except OSError as error:
                print(json.dumps({"join": group, "on": local, "error": str(error)}), file=out)
    sel.register(multicast, selectors.EVENT_READ, ("sacn", sacn))
    print(json.dumps({"listening": addresses()}), file=out)
    last, counts, summary = {}, {}, begin + 5
    while stop is None or time.monotonic() < stop:
        for key, _ in sel.select(timeout=1):
            data, sender = key.fileobj.recvfrom(2048)
            if pcap:
                pcap.write(data, sender, key.fileobj.getsockname()[1])
            proto, parse = key.data
            packet = parse(data)
            now = round(time.monotonic() - begin, 3)
            if packet is None:
                print(json.dumps({"t": now, "proto": proto, "from": sender[0], "other": data[:12].hex()}), file=out)
                continue
            slots = packet.pop("slots", b"")
            ident = (proto, sender[0], packet.get("universe"), packet.get("priority"))
            counts[ident] = counts.get(ident, 0) + 1
            levels = {i + 1: v for i, v in enumerate(slots) if v}
            if last.get(ident) != (levels, len(slots)):
                last[ident] = (levels, len(slots))
                packet.pop("seq", None)
                print(
                    json.dumps({"t": now, "proto": proto, "from": sender[0], **packet, "length": len(slots), "levels": levels}),
                    file=out,
                )
        if time.monotonic() >= summary:
            summary += 5
            print(
                json.dumps({"t": round(time.monotonic() - begin, 3), "counts": [list(k) + [v] for k, v in counts.items()]}),
                file=out,
            )
            counts = {}


main()
