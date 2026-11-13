# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Talk OSC to Eos over TCP, framed as OSC 1.0 with a length before each
packet or as OSC 1.1 with SLIP.

Usage: osc.py LOG STEPS [HOST]

STEPS holds one step per line: `probe PORT:FRAMING...` connects to the
first port that answers a ping, where FRAMING is `length` or `slip`;
`connect PORT:FRAMING` waits up to fifteen minutes for that port; `send
ADDRESS [ARG...]` sends a message, whose arguments are integers, floats, or
strings (quote a string with spaces); `wait SECONDS` reads replies for that
long; `until ADDRESS SECONDS` reads until a reply to ADDRESS arrives; `mark
TEXT` notes where the steps are.  Every message both ways goes to LOG with
the time since start.
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


END, ESC = b"\xc0", b"\xdb"


class Link:
    """TCP connection to Eos, framed as OSC 1.0 (`length`, a length before
    each packet) or OSC 1.1 (`slip`, packets between END bytes)"""

    def __init__(self, host, port, framing, log, patience=0):
        self.log, self.buffer, self.framing = log, b"", framing
        deadline = time.monotonic() + patience
        while True:
            try:
                self.sock = socket.create_connection((host, port), timeout=3)
                break
            except OSError as error:
                if time.monotonic() > deadline:
                    raise
                self.note(f"connect to {port} failed: {error}")
                time.sleep(3)
        self.note(f"connected to {port} as {framing}")

    def note(self, text):
        print(now(), text, file=self.log, flush=True)

    def send(self, address, args):
        packet = encoded(address, args)
        if self.framing == "slip":
            escaped = packet.replace(ESC, ESC + b"\xdd").replace(END, ESC + b"\xdc")
            self.sock.sendall(END + escaped + END)
        else:
            self.sock.sendall(struct.pack(">i", len(packet)) + packet)
        self.note(f"> {address} {args}")

    def packets(self):
        if self.framing == "slip":
            *frames, self.buffer = self.buffer.split(END)
            return [f.replace(ESC + b"\xdc", END).replace(ESC + b"\xdd", ESC) for f in frames if f]
        found = []
        while len(self.buffer) >= 4:
            size = struct.unpack(">i", self.buffer[:4])[0]
            if len(self.buffer) < 4 + size:
                break
            found.append(self.buffer[4 : 4 + size])
            self.buffer = self.buffer[4 + size :]
        return found

    def read(self, seconds, wanted=None):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            self.sock.settimeout(max(0.05, deadline - time.monotonic()))
            try:
                chunk = self.sock.recv(65536)
            except (TimeoutError, socket.timeout):
                continue
            except OSError as error:
                self.note(f"failed: {error}")
                return False
            if not chunk:
                self.note("closed")
                return False
            self.buffer += chunk
            for packet in self.packets():
                try:
                    messages = decoded(packet)
                except (ValueError, struct.error, IndexError):
                    self.note(f"< undecoded {packet[:64]!r}")
                    continue
                for address, args in messages:
                    self.note(f"< {address} {args}")
                    if wanted is not None and address.startswith(wanted):
                        return True
        return wanted is None


def probe(host, candidates, log):
    """Returns a link to the first of `candidates`, given as PORT:FRAMING,
    that answers a ping, or none"""
    for candidate in candidates:
        port, framing = candidate.split(":")
        try:
            link = Link(host, int(port), framing, log)
        except OSError as error:
            print(now(), f"no connection to {candidate}: {error}", file=log, flush=True)
            continue
        link.send("/eos/ping", [])
        if link.read(5, "/eos/out/ping"):
            return link
        link.sock.close()
    return None


def main():
    log = open(sys.argv[1], "a")
    steps = [shlex.split(line) for line in open(sys.argv[2]) if line.strip() and not line.startswith("#")]
    host = sys.argv[3] if len(sys.argv) > 3 else "127.0.0.1"
    link = None
    for step in steps:
        match step:
            case ["probe", *candidates]:
                link = probe(host, candidates, log)
                if link is None:
                    print(now(), "no port answered", file=log, flush=True)
                    sys.exit(1)
            case ["connect", candidate]:
                port, framing = candidate.split(":")
                link = Link(host, int(port), framing, log, patience=900)
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
