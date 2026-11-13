# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Talk OSC to Eos over TCP, framed as OSC 1.0 with a length before each
packet, which Eos takes on port 3032.

Usage: osc.py LOG STEPS [HOST]

STEPS holds one step per line: `send ADDRESS [ARG...]` sends a message,
whose arguments are integers, floats, or strings (quote a string with
spaces); `wait SECONDS` reads replies for that long; `until ADDRESS SECONDS`
reads until a reply to ADDRESS arrives.  The script waits up to fifteen
minutes for Eos to accept the connection, and logs every message both ways
with the time since start.
"""

import shlex
import socket
import struct
import sys
import time

BEGIN = time.monotonic()


def now():
    return f"{time.monotonic() - BEGIN:8.3f}"


def padded(raw):
    return raw + b"\0" * (4 - len(raw) % 4)


def encoded(address, args):
    tags, data = ",", b""
    for arg in args:
        if isinstance(arg, int):
            tags, data = tags + "i", data + struct.pack(">i", arg)
        elif isinstance(arg, float):
            tags, data = tags + "f", data + struct.pack(">f", arg)
        else:
            tags, data = tags + "s", data + padded(arg.encode())
    return padded(address.encode()) + padded(tags.encode()) + data


def string(data, at):
    end = data.index(b"\0", at)
    return data[at:end].decode(errors="replace"), (end // 4 + 1) * 4


def decoded(data):
    if data.startswith(b"#bundle\0"):
        at, messages = 16, []
        while at < len(data):
            size = struct.unpack(">i", data[at : at + 4])[0]
            messages += decoded(data[at + 4 : at + 4 + size])
            at += 4 + size
        return messages
    address, at = string(data, 0)
    if at >= len(data):
        return [(address, [])]
    tags, at = string(data, at)
    args = []
    for tag in tags[1:]:
        if tag == "i":
            args.append(struct.unpack(">i", data[at : at + 4])[0])
            at += 4
        elif tag == "f":
            args.append(round(struct.unpack(">f", data[at : at + 4])[0], 4))
            at += 4
        elif tag == "s":
            value, at = string(data, at)
            args.append(value)
        elif tag in "TF":
            args.append(tag == "T")
        else:
            args.append(f"?{tag}")
    return [(address, args)]


def argument(word):
    for kind in (int, float):
        try:
            return kind(word)
        except ValueError:
            pass
    return word


class Link:
    def __init__(self, host, log):
        self.log, self.buffer = log, b""
        deadline = time.monotonic() + 900
        while True:
            try:
                self.sock = socket.create_connection((host, 3032), timeout=3)
                break
            except OSError as error:
                if time.monotonic() > deadline:
                    raise
                self.note(f"connect failed: {error}")
                time.sleep(3)
        self.note("connected")

    def note(self, text):
        print(now(), text, file=self.log, flush=True)

    def send(self, address, args):
        packet = encoded(address, args)
        self.sock.sendall(struct.pack(">i", len(packet)) + packet)
        self.note(f"> {address} {args}")

    def read(self, seconds, wanted=None):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            self.sock.settimeout(max(0.05, deadline - time.monotonic()))
            try:
                chunk = self.sock.recv(65536)
            except TimeoutError:
                continue
            except socket.timeout:
                continue
            if not chunk:
                self.note("closed")
                return False
            self.buffer += chunk
            while len(self.buffer) >= 4:
                size = struct.unpack(">i", self.buffer[:4])[0]
                if len(self.buffer) < 4 + size:
                    break
                packet, self.buffer = self.buffer[4 : 4 + size], self.buffer[4 + size :]
                for address, args in decoded(packet):
                    self.note(f"< {address} {args}")
                    if wanted is not None and address.startswith(wanted):
                        return True
        return wanted is None


def main():
    log = open(sys.argv[1], "a")
    steps = [shlex.split(line) for line in open(sys.argv[2]) if line.strip() and not line.startswith("#")]
    link = Link(sys.argv[3] if len(sys.argv) > 3 else "127.0.0.1", log)
    for step in steps:
        match step:
            case ["send", address, *args]:
                link.send(address, [argument(arg) for arg in args])
            case ["wait", seconds]:
                link.read(float(seconds))
            case ["until", address, seconds]:
                if not link.read(float(seconds), address):
                    link.note(f"no {address} within {seconds} s")
            case ["mark", *words]:
                link.note("## " + " ".join(words))


main()
