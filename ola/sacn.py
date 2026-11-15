# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Stand in for a lighting console that sends sACN (E1.31).

Usage: sacn.py [--universe N] [--priority P] [--name NAME] [--interface IP]
               [--to ADDRESS] [--interval SECONDS] [--seconds S] [--levels 1=102,3=128]

--levels gives DMX values (0-255) by address.  The source sends its universe
every INTERVAL seconds, for SECONDS seconds, or until it is terminated.  On
SIGTERM it sends three packets with the stream terminated option, as a
console that is shut down properly does, unless --no-terminate is given.
"""

import argparse
import signal
import socket
import struct
import time
import uuid


def packet(universe, priority, name, cid, seq, slots, terminated=False):
    """Returns a data packet of E1.31, whose layers each start with their
    flags and length, 0x7 and the length from those two bytes to the end"""
    values = bytes([0]) + slots
    dmp = struct.pack(">HBBHHH", 0x7000 | (10 + len(values)), 2, 0xA1, 0, 1, len(values)) + values
    framing_head = (
        struct.pack(">I", 2)
        + name.encode().ljust(64, b"\0")
        + struct.pack(">BHBBH", priority, 0, seq, 0x40 if terminated else 0, universe)
    )
    framing = struct.pack(">H", 0x7000 | (2 + len(framing_head) + len(dmp))) + framing_head + dmp
    root_head = struct.pack(">I", 4) + cid
    root = struct.pack(">H", 0x7000 | (2 + len(root_head) + len(framing))) + root_head + framing
    return struct.pack(">HH", 0x0010, 0) + b"ASC-E1.17\0\0\0" + root


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--universe", type=int, default=1)
    parser.add_argument("--priority", type=int, default=100)
    parser.add_argument("--name", default="console")
    parser.add_argument("--interface", default="0.0.0.0")
    parser.add_argument("--to", default=None, help="unicast address instead of the multicast group")
    parser.add_argument("--interval", type=float, default=0.2)
    parser.add_argument("--seconds", type=float, default=None)
    parser.add_argument("--levels", default="")
    parser.add_argument("--no-terminate", action="store_true")
    args = parser.parse_args()
    slots = bytearray(512)
    for item in filter(None, args.levels.split(",")):
        address, value = item.split("=")
        slots[int(address) - 1] = int(value)
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_IF, socket.inet_aton(args.interface))
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_MULTICAST_TTL, 4)
    group = (args.to or f"239.255.{args.universe >> 8}.{args.universe & 255}", 5568)
    cid = uuid.uuid5(uuid.NAMESPACE_DNS, args.name).bytes
    done = []
    signal.signal(signal.SIGTERM, lambda *_: done.append(1))
    signal.signal(signal.SIGINT, lambda *_: done.append(1))
    stop = time.monotonic() + args.seconds if args.seconds else None
    seq = 0
    while not done and (stop is None or time.monotonic() < stop):
        sock.sendto(packet(args.universe, args.priority, args.name, cid, seq, bytes(slots)), group)
        seq = (seq + 1) % 256
        time.sleep(args.interval)
    if not args.no_terminate:
        for _ in range(3):
            sock.sendto(packet(args.universe, args.priority, args.name, cid, seq, bytes(slots), True), group)
            seq = (seq + 1) % 256


main()
