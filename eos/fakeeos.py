# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Stand in for a console that takes OSC: decode what arrives over TCP,
framed with SLIP, and over UDP, and log each message with its time.

Usage: fakeeos.py LOG SECONDS [TCP_PORT [UDP_PORT]]
"""

import selectors
import socket
import sys
import time

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from osc import decoded  # noqa: E402

END, ESC = b"\xc0", b"\xdb"


def main():
    log = open(sys.argv[1], "a", buffering=1)
    stop = time.monotonic() + float(sys.argv[2])
    tcp_port = int(sys.argv[3]) if len(sys.argv) > 3 else 3037
    udp_port = int(sys.argv[4]) if len(sys.argv) > 4 else 8000
    sel = selectors.DefaultSelector()
    server = socket.create_server(("127.0.0.1", tcp_port), reuse_port=True)
    sel.register(server, selectors.EVENT_READ, "accept")
    udp = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    udp.bind(("127.0.0.1", udp_port))
    sel.register(udp, selectors.EVENT_READ, "udp")
    buffers = {}

    def note(how, packet):
        try:
            messages = decoded(packet)
        except Exception as error:  # Any packet that does not decode
            messages = [("undecoded", [repr(packet[:64]), str(error)])]
        for address, args in messages:
            print(f"{time.time():.3f} {how} {address} {args}", file=log)

    while time.monotonic() < stop:
        for key, _ in sel.select(timeout=1):
            if key.data == "accept":
                connection, peer = server.accept()
                print(f"{time.time():.3f} tcp connected from {peer}", file=log)
                buffers[connection] = b""
                sel.register(connection, selectors.EVENT_READ, "tcp")
            elif key.data == "udp":
                note("udp", udp.recv(65536))
            else:
                chunk = key.fileobj.recv(65536)
                if not chunk:
                    print(f"{time.time():.3f} tcp closed", file=log)
                    sel.unregister(key.fileobj)
                    continue
                *frames, buffers[key.fileobj] = (buffers[key.fileobj] + chunk).split(END)
                for frame in frames:
                    if frame:
                        note("tcp", frame.replace(ESC + b"\xdc", END).replace(ESC + b"\xdd", ESC))


main()
