# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Listen on all addresses, as a program does that Windows Defender Firewall
asks about, for two minutes"""

import socket
import time

server = socket.socket()
server.bind(("0.0.0.0", 5999))
server.listen()
datagrams = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
datagrams.bind(("0.0.0.0", 5998))
print("listening", flush=True)
time.sleep(120)
