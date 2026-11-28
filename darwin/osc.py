# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Send an OSC message over UDP: osc.py HOST PORT ADDRESS"""

import socket
import sys


def padded(raw):
    return raw + b"\0" * (4 - len(raw) % 4)


host, port, address = sys.argv[1], int(sys.argv[2]), sys.argv[3]
socket.socket(socket.AF_INET, socket.SOCK_DGRAM).sendto(padded(address.encode()) + padded(b","), (host, port))
