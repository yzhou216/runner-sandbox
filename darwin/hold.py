# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Hold a UDP port: hold.py HOST PORT SECONDS"""

import socket
import sys
import time

sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind((sys.argv[1], int(sys.argv[2])))
time.sleep(float(sys.argv[3]))
