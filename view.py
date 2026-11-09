#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Draws the first channels of each Art-Net universe that reaches UDP port
6454 as lamps in a terminal, each as bright as its channel, standing in for
the dimmers that cinq's light cues would drive.

Usage: view.py [LAMPS], where LAMPS is the number of channels to draw.
"""

import shutil
import socket
import struct
import sys

LAMPS = int(sys.argv[1]) if len(sys.argv) > 1 else 8

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
sock.bind(("", 6454))
universes, size = {}, None
print("\x1b[?25l", end="")
while True:
    packet, _ = sock.recvfrom(1024)
    # ArtDmx only, which opcode 0x5000 marks
    if packet[:8] != b"Art-Net\0" or struct.unpack_from("<H", packet, 8)[0] != 0x5000:
        continue
    (universe,) = struct.unpack_from("<H", packet, 14)
    (length,) = struct.unpack_from(">H", packet, 16)
    universes[universe] = packet[18 : 18 + length]
    columns, rows = shutil.get_terminal_size()
    width, height = columns // LAMPS - 2, rows // len(universes) - 4
    # Lamps drawn at another size would linger
    out = ["\x1b[2J" if (columns, rows) != size else "", "\x1b[H"]
    size = columns, rows
    for row, (universe, channels) in enumerate(sorted(universes.items())):
        top = 1 + row * (height + 4)
        out.append(f"\x1b[{top};2H\x1b[0mUniverse {universe}")
        for i, value in enumerate(channels[:LAMPS]):
            # Warm white, as a tungsten lamp dims
            color = f"{value};{value * 85 // 100};{value * 60 // 100}"
            left = 2 + i * (width + 2)
            out += [
                f"\x1b[{top + 2 + line};{left}H\x1b[48;2;{color}m{' ' * width}"
                for line in range(height)
            ]
            out.append(f"\x1b[{top + 2 + height};{left}H\x1b[0m{i + 1:>2}: {round(value / 2.55):>3}%")
    print("".join(out), end="", flush=True)
