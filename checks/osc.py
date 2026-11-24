#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Sends an OSC message with no arguments over UDP to 127.0.0.1:53000.

Usage: osc.py ADDRESS
"""
import socket
import sys


def pad(data: bytes) -> bytes:
    return data + b"\0" * (4 - len(data) % 4)


socket.socket(socket.AF_INET, socket.SOCK_DGRAM).sendto(
    pad(sys.argv[1].encode()) + pad(b","), ("127.0.0.1", 53000)
)
