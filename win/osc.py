# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Sends one OSC message without arguments over UDP: osc.py HOST PORT ADDRESS
import socket
import sys


def pad(b):
    return b + b"\0" * (4 - len(b) % 4)


host, port, address = sys.argv[1], int(sys.argv[2]), sys.argv[3]
socket.socket(socket.AF_INET, socket.SOCK_DGRAM).sendto(
    pad(address.encode()) + pad(b","), (host, port))
