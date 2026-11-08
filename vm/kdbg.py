# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Answer ReactOS's kernel debugger on COM1 with a backtrace and cont, so
that a failed assertion in a test program does not freeze the machine."""

import socket
import sys
import time

sock = socket.socket(socket.AF_UNIX)
for _ in range(600):
    try:
        sock.connect(sys.argv[1])
        break
    except OSError:
        time.sleep(0.1)
tail = b""
for answers in range(200):
    while not tail.rstrip().endswith(b"kdb:>"):
        data = sock.recv(4096)
        if not data:
            sys.exit()
        tail = (tail + data)[-32:]
    tail = b""
    for command in (b"bt\r", b"cont\r"):
        time.sleep(0.3)
        sock.sendall(command)
