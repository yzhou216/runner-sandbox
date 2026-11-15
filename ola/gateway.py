# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Play gateway.toml in cinq into OLA, the Open Lighting Architecture, as a
network-to-DMX gateway: cinq's sACN and Art-Net arrive at OLA's E1.31 and
Art-Net input ports, a stand-in console (sacn.py) sends a second sACN source,
and the universes of OLA are read over its HTTP API and with ola_recorder.

Usage: gateway.py OUT SHOW WORK, started by gateway.sh in its sway session

Prints one line per check, ok or WRONG, and every measurement as info, to
OUT/verdict.txt too.  Exits 1 if a check is wrong.
"""

import http.client
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path

out, show, work = Path(sys.argv[1]), sys.argv[2], Path(sys.argv[3])
here = Path(__file__).parent
HTTP_PORT = 9090
PLUGINS = "artnet dummy e131 espnet ftdidmx gpio karate kinet milinst opendmx openpixelcontrol pathport renard sandnet shownet spi stageprofi uartdmx usbdmx usbserial".split()
# DMX values of gateway.toml's first look, by universe and address
LOOK = {1: {1: 77, 3: 51, 5: 255}, 2: {1: 51, 2: 179}, 999: {1: 128}}
verdict = open(out / "verdict.txt", "a", buffering=1)
marks = open(out / "marks.txt", "a", buffering=1)
wrong = False
# Multicast on this machine's default network, or unicast to the loopback
# address where a machine has no multicast route
UNICAST = os.environ.get("GATEWAY_UNICAST")


def say(line):
    print(line, flush=True)
    print(line, file=verdict)


def check(good, text):
    global wrong
    wrong |= not good
    say(f"{'ok' if good else 'WRONG'}: {text}")


def info(text):
    say(f"info: {text}")


def mark(text):
    print(f"{time.time():.3f} {text}", file=marks)


def tool(*args):
    done = subprocess.run(args, capture_output=True, text=True)
    return done.stdout + done.stderr


def route_address():
    """Returns the address that the default route sends from"""
    return re.search(r"src (\S+)", tool("ip", "-4", "route", "get", "1.1.1.1")).group(1)


# olad and its ports

class Olad:
    def __init__(self, extra="", artnet="net = 0\nsubnet = 0\noutput_ports = 0\nuse_loopback = true\n"):
        self.conf = work / "ola"
        shutil.rmtree(self.conf, ignore_errors=True)
        self.conf.mkdir()
        for plugin in PLUGINS:
            (self.conf / f"ola-{plugin}.conf").write_text("enabled = false\n")
        (self.conf / "ola-e131.conf").write_text("enabled = true\ninput_ports = 4\noutput_ports = 0\nignore_preview = true\n" + extra)
        (self.conf / "ola-artnet.conf").write_text("enabled = true\n" + artnet + extra)
        self.log = open(out / "olad.log", "a")
        self.proc = subprocess.Popen(
            ["olad", "-c", str(self.conf), "--http-port", str(HTTP_PORT), "--no-register-with-dns-sd", "-l", "3"],
            stdout=self.log,
            stderr=subprocess.STDOUT,
        )
        for _ in range(100):
            if "E1.31" in tool("ola_dev_info"):
                return
            time.sleep(0.2)

    def stop(self):
        self.proc.terminate()
        self.proc.wait()


def devices():
    """Returns the ids of the ArtNet and the E1.31 device"""
    found = {kind: int(n) for n, kind in re.findall(r"Device (\d+): (ArtNet|E1\.31)", tool("ola_dev_info"))}
    return found["ArtNet"], found["E1.31"]


def patch(device, port, universe):
    return tool("ola_patch", "--patch", "-d", str(device), "-p", str(port), "-u", str(universe), "-i")


def unpatch(device, port, universe):
    return tool("ola_patch", "--unpatch", "-d", str(device), "-p", str(port), "-u", str(universe), "-i")


class Sampler(threading.Thread):
    """Reads universes of OLA over HTTP about 40 times a second and keeps
    every change, with its time"""

    def __init__(self, universes):
        super().__init__(daemon=True)
        self.universes = universes
        self.history = []
        self.last = {}
        self.sizes = {}
        self.done = False

    def run(self):
        conn = http.client.HTTPConnection("127.0.0.1", HTTP_PORT, timeout=5)
        while not self.done:
            for universe in self.universes:
                try:
                    conn.request("GET", f"/get_dmx?u={universe}")
                    reply = json.loads(conn.getresponse().read())
                except (OSError, ValueError, http.client.HTTPException):
                    conn.close()
                    conn = http.client.HTTPConnection("127.0.0.1", HTTP_PORT, timeout=5)
                    continue
                now = time.time()
                slots = reply.get("dmx", [])
                levels = {i + 1: v for i, v in enumerate(slots) if v}
                self.sizes[universe] = len(slots)
                if self.last.get(universe) != levels:
                    self.last[universe] = levels
                    self.history.append((now, universe, levels))
            time.sleep(0.02)

    def latest(self, universe):
        return self.last.get(universe, {})

    def changes(self, universe, since):
        return [(t, levels) for t, u, levels in self.history if u == universe and t >= since]

    def wait_for(self, universe, want, timeout):
        """Returns the time at which the universe holds WANT, or None"""
        end = time.time() + timeout
        while time.time() < end:
            if self.latest(universe) == want:
                return time.time()
            time.sleep(0.02)
        return None

    def dump(self):
        with open(out / "levels.jsonl", "w") as file:
            for t, universe, levels in self.history:
                print(json.dumps({"t": round(t, 3), "universe": universe, "levels": levels}), file=file)


class Recorder:
    """ola_recorder on universes, whose file lists each frame that OLA
    received and the milliseconds to the next"""

    def __init__(self, name, universes):
        self.path = out / f"{name}.rec"
        self.proc = subprocess.Popen(["ola_recorder", "--record", str(self.path), "--universes", ",".join(map(str, universes))], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(0.7)

    def stop(self):
        """Returns the frames as (seconds since the first, universe, slots)"""
        self.proc.send_signal(signal.SIGINT)
        self.proc.wait()
        lines = self.path.read_text().splitlines()[1:]
        frames, at = [], 0.0
        for frame, delay in zip(lines[0::2], lines[1::2]):
            universe, slots = frame.split(" ", 1)
            frames.append((at, int(universe), [int(v) for v in slots.split(",")]))
            at += int(delay) / 1000
        return frames


# cinq and the console

class Cinq:
    def __init__(self):
        self.log = open(out / "cinq.log", "a")
        self.proc = subprocess.Popen(["cinq", show], stdout=self.log, stderr=subprocess.STDOUT)
        for _ in range(60):
            if re.search(r'"app_id": "cinq"', tool("swaymsg", "-t", "get_tree")):
                break
            time.sleep(0.5)
        time.sleep(3)

    def go(self, label):
        """Presses space, and returns the times before and after the key"""
        tool("swaymsg", "-q", '[app_id="cinq"] focus')
        before = time.time()
        tool("wtype", "-s", "300", " ")
        after = time.time()
        mark(f"GO {label}")
        return before, after

    def quit(self, how):
        """Ends cinq and returns the time at which it was gone"""
        mark(f"cinq {how}")
        if how == "window closed":
            tool("swaymsg", '[app_id="cinq"] kill')
        else:
            self.proc.kill()
        for _ in range(500):
            if self.proc.poll() is not None:
                return time.time()
            time.sleep(0.02)
        info(f"cinq did not end {how}")
        self.proc.kill()
        self.proc.wait()
        return time.time()


def console(name, priority, levels):
    address = route_address()
    args = [sys.executable, str(here / "sacn.py"), "--name", name, "--priority", str(priority), "--levels", levels, "--interval", "0.2"]
    args += ["--to", "127.0.0.1"] if UNICAST else ["--interface", address]
    mark(f"console {name} priority {priority} {levels}")
    return subprocess.Popen(args)


def stop_console(proc):
    mark("console stopped")
    proc.send_signal(signal.SIGTERM)
    proc.wait()
    return time.time()


def fmt(levels):
    return json.dumps({str(k): v for k, v in levels.items()})


# What each phase finds

def run_levels(sampler, cinq):
    before = time.time()
    first = {u: sampler.latest(u) for u in LOOK}
    check(not any(first.values()), f"before the first cue, OLA holds nothing: {first}")
    key = cinq.go("1 look")
    time.sleep(3)
    for universe, want in LOOK.items():
        got = sampler.latest(universe)
        check(got == want, f"(a) OLA universe {universe} holds {fmt(got)}, want {fmt(want)}")
        steps = sampler.changes(universe, before)
        info(f"(a) universe {universe}: {len(steps)} changes, first {steps[0][0] - key[1]:+.2f} s after the key (key between {key[0] - key[1]:+.2f} and 0)" if steps else f"(a) universe {universe}: no change")
    info(f"slots per frame: {sampler.sizes}")


def fade_trace(frames, want_first, want_last):
    """Checks the frames of a fade of slot 1 and returns its numbers"""
    values = [slots[0] for _, _, slots in frames]
    start = next((i for i, v in enumerate(values) if v != want_first), None)
    end = next((i for i, v in enumerate(values) if v == want_last), None)
    if start is None or end is None:
        return None
    span = frames[end][0] - frames[start][0]
    rise = values[start - 1 : end + 1]
    return {
        "frames": len(frames),
        "first value": want_first,
        "frames in the fade": end - start + 1,
        "seconds": round(span, 3),
        "frames per second": round((end - start) / span, 1) if span else None,
        "monotonic": all(a <= b for a, b in zip(rise, rise[1:])),
        "distinct values": len(set(rise)),
        "largest step": max(b - a for a, b in zip(rise, rise[1:])),
        "values": rise,
    }


def run_fade(sampler, cinq):
    recorder = Recorder("fade", [1])
    cinq.go("2 fade up")
    time.sleep(5.5)
    frames = recorder.stop()
    trace = fade_trace(frames, 77, 179)
    check(trace is not None and trace["monotonic"] and trace["values"][-1] == 179, f"(e) levels rise monotonically 77 to 179: {trace and {k: v for k, v in trace.items() if k != 'values'}}")
    if trace:
        info(f"(e) values by frame: {trace['values']}")
        total = frames[-1][0] - frames[0][0]
        info(f"(e) {len(frames)} frames in {total:.1f} s of recording, {len(frames) / total:.1f} per second overall")


def run_merge(sampler, cinq):
    cinq.go("3 back down")
    time.sleep(1.5)
    check(sampler.latest(1).get(1) == 77, f"universe 1 address 1 is back at 77 before the console: {fmt(sampler.latest(1))}")
    started = time.time()
    source = console("console", 100, "1=102,3=128,6=77")
    want = {1: 102, 3: 128, 5: 255, 6: 77}
    seen = sampler.wait_for(1, want, 6)
    check(seen is not None, f"(b) console at priority 100 merges with cinq per address (HTP): {fmt(sampler.latest(1))}, want {fmt(want)}" + (f", after {seen - started:.2f} s" if seen else ""))
    recorder = Recorder("merge", [1])
    cinq.go("4 fade up with console")
    time.sleep(5.5)
    frames = recorder.stop()
    values = [slots[0] for _, _, slots in frames]
    others = {(s[2], s[4], s[5]) for _, _, s in frames}
    check(min(values) >= 102 and values[-1] == 179 and all(a <= b for a, b in zip(values, values[1:])), f"(b) address 1 stays at 102 while cinq is below it and rises to 179 with it: min {min(values)}, last {values[-1]}")
    check(others == {(128, 255, 77)}, f"(b) addresses 3, 5, 6 hold {sorted(others)} in every frame of the fade, want [(128, 255, 77)]")
    leave = next((i for i, v in enumerate(values) if v > 102), None)
    top = next((i for i, v in enumerate(values) if v == 179), None)
    if leave is not None and top is not None:
        info(f"(b) address 1 left 102 and reached 179 {frames[top][0] - frames[leave][0]:.2f} s later (cinq's own fade: 3.0 s from 77, 102 after 0.74 s, so 2.26 s)")
    return source


def run_priority(sampler, cinq, source):
    t = stop_console(source)
    cinq_only = {1: 179, 3: 51, 5: 255}
    seen = sampler.wait_for(1, cinq_only, 10)
    info(f"console at 100 stopped with its terminate packets (SIGTERM): OLA back to cinq alone {'after %.2f s' % (seen - t) if seen else 'never, holds ' + fmt(sampler.latest(1))}")
    weak = console("weak", 1, "1=200,3=255,7=99")
    time.sleep(3)
    got = sampler.latest(1)
    check(got == cinq_only, f"(c) console at priority 1 is ignored while cinq sends at 100: {fmt(got)}, want {fmt(cinq_only)}")
    return weak


def run_takeover(sampler, cinq, weak):
    ended = cinq.quit("window closed")
    takeover = {1: 200, 3: 255, 7: 99}
    seen = sampler.wait_for(1, takeover, 15)
    check(seen is not None, f"(c) console at priority 1 takes over after cinq ends: {fmt(sampler.latest(1))}, " + (f"{seen - ended:.2f} s after cinq's window closed (process gone at +0)" if seen else "no"))
    steps = sampler.changes(1, ended)
    info(f"(c) universe 1 changes after cinq ended: {[(round(t - ended, 2), fmt(v)) for t, v in steps]}")
    t = stop_console(weak)
    seen = sampler.wait_for(1, {}, 15)
    info(f"(d) after the last source sends its terminate packets (SIGTERM), OLA universe 1 {'is empty after %.2f s' % (seen - t) if seen else 'still holds after 15 s ' + fmt(sampler.latest(1))}")


def run_loss(sampler, how, label):
    """cinq alone: OLA's levels, then cinq ends"""
    cinq = Cinq()
    cinq.go("1 look")
    seen = sampler.wait_for(1, LOOK[1], 6)
    check(seen is not None, f"universe 1 back at {fmt(LOOK[1])} for the {label} test: {fmt(sampler.latest(1))}")
    time.sleep(2)
    ended = cinq.quit(how)
    seen = sampler.wait_for(1, {}, 60)
    after = sampler.changes(1, ended)
    info(f"(d) cinq ended ({how}): OLA universe 1 changes after it: {[(round(t - ended, 2), fmt(v)) for t, v in after]}")
    if seen:
        info(f"(d) OLA dropped cinq's levels {seen - ended:.2f} s after cinq ended ({how}); universe 1 then holds {fmt(sampler.latest(1))}")
    else:
        info(f"(d) OLA still holds cinq's levels {fmt(sampler.latest(1))} 60 s after cinq ended ({how}), as a receiver does that holds its last look")


def run_artnet(sampler, olad):
    artnet, e131 = devices()
    for port, universe in enumerate((1, 2, 999)):
        info(f"unpatch E1.31 port {port}: {unpatch(e131, port, universe).strip() or 'ok'}")
    for port, universe in enumerate((0, 1)):
        info(f"patch Art-Net port {port} to OLA universe {universe}: {patch(artnet, port, universe).strip() or 'ok'}")
    info("Art-Net after the patch:\n" + tool("ola_dev_info"))
    cinq = Cinq()
    cinq.go("1 look")
    time.sleep(4)
    check(sampler.latest(0) == LOOK[1], f"(f) cinq universe 1 arrives as Art-Net Port-Address 0: OLA universe 0 holds {fmt(sampler.latest(0))}, want {fmt(LOOK[1])}")
    check(sampler.latest(1) == LOOK[2], f"(f) cinq universe 2 arrives as Port-Address 1: OLA universe 1 holds {fmt(sampler.latest(1))}, want {fmt(LOOK[2])}")
    cinq.quit("window closed")


def run_downstream(olad, address):
    """OLA's own Art-Net output, which a gateway sends on to its lights, as
    cinq runs and after it quits"""
    olad.stop()
    olad = Olad(f"ip = {os.environ.get('GATEWAY_ARTNET_IP', address)}\n", "net = 5\nsubnet = 0\ninput_ports = 0\noutput_ports = 1\nuse_loopback = true\n")
    artnet, e131 = devices()
    info("sACN in: " + (patch(e131, 0, 1).strip() or "ok") + ", Art-Net out: " + (tool("ola_patch", "--patch", "-d", str(artnet), "-p", "0", "-u", "1").strip() or "ok"))
    info("downstream devices:\n" + tool("ola_dev_info"))
    environment = dict(os.environ, LISTEN_UNIVERSES="1", LISTEN_SUMMARY="1")
    log = out / "downstream.jsonl"
    listener = subprocess.Popen([sys.executable, str(here.parent / "eos" / "listen.py"), str(log), "75"], env=environment)
    begin = time.time()
    time.sleep(2)
    cinq = Cinq()
    cinq.go("1 look")
    time.sleep(4)
    cinq.go("2 fade up")
    time.sleep(6)
    ended = cinq.quit("window closed")
    listener.wait()
    # Art-Net universe 1281 is Net 5, Sub-Net 0, Universe 1 of OLA's output
    heard = [json.loads(line) for line in log.read_text().splitlines()]
    changes = [(h["t"], h["levels"], h["from"]) for h in heard if h.get("proto") == "artnet" and h.get("universe") == 1281]
    info(f"(g) OLA's Art-Net output (Port-Address 1281) changes: {[(round(t, 1), fmt({int(k): v for k, v in levels.items()}), sender) for t, levels, sender in changes]}")
    series = [(row["t"], sum(c[4] for c in row["counts"] if c[0] == "artnet" and c[2] == 1281)) for row in heard if "counts" in row]
    after = [count for t, count in series if t > ended - begin + 1]
    info(f"(g) Art-Net packets per second from OLA, before and after cinq ended (at {ended - begin:.0f} s): {[count for _, count in series]}")
    final = changes[-1][1] if changes else None
    check(bool(after) and all(count >= 1 for count in after[:-2]) and final == {"1": 179, "3": 51, "5": 255}, f"(g) after cinq ended, OLA's output keeps sending cinq's last look {final} at {after[:3]} packets per second for {len(after)} s")
    return olad


def main():
    olad = Olad()
    address = route_address()
    artnet, e131 = devices()
    listing = tool("ola_dev_info")
    info("ola_dev_info after start:\n" + listing)
    chosen = re.search(r"E1\.31 \(DMX over ACN\) \[(\S+)\]", listing)
    info(f"default route sends from {address}, OLA's E1.31 uses {chosen and chosen[1]}")
    if chosen and chosen[1] != address:
        olad.stop()
        olad = Olad(f"ip = {address}\n")
        artnet, e131 = devices()
        info("restarted with ip = " + address + "\n" + tool("ola_dev_info"))
    for port, universe in enumerate((1, 2, 999)):
        info(f"patch E1.31 port {port} to universe {universe}: {patch(e131, port, universe).strip() or 'ok'}")
    info("after patching:\n" + tool("ola_dev_info") + tool("ola_uni_info"))
    sampler = Sampler([0, 1, 2, 998, 999])
    sampler.start()
    # Only the last phase, for trying it out
    if os.environ.get("GATEWAY_ONLY") == "downstream":
        run_downstream(olad, address).stop()
        sampler.done = True
        return
    cinq = Cinq()
    steps = (
        lambda: run_levels(sampler, cinq),
        lambda: run_fade(sampler, cinq),
    )
    for step in steps:
        step()
    source = run_merge(sampler, cinq)
    weak = run_priority(sampler, cinq, source)
    run_takeover(sampler, cinq, weak)
    run_loss(sampler, "window closed", "window closed")
    run_loss(sampler, "killed", "killed")
    run_artnet(sampler, olad)
    olad = run_downstream(olad, address)
    sampler.done = True
    sampler.dump()
    olad.stop()


try:
    main()
except Exception:
    import traceback

    wrong = True
    say("WRONG: " + traceback.format_exc())
sys.exit(1 if wrong else 0)
