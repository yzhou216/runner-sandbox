# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Judges the checks of this branch from the recordings and from the log
that the patch of cinq makes.

Usage: verdict.py KIND DIRECTORY, where KIND is jack, pulse, clamp, windows,
or darwin.  Each line it prints reads CHECK, the show or phase, passed,
failed, or inconclusive, and the evidence.  Lines of cinq's log are:
K TIME EVENT for each key that no widget took, U TIME MESSAGE | STATE after
each update, and O TIME open DEVICE begins or ends RESULT around each opening
of a device, TIME being seconds since the epoch.
"""

import array
import math
import re
import subprocess
import sys
from pathlib import Path

RATE = 48000


def lines(path):
    path = Path(path)
    return path.read_text(errors="replace").splitlines() if path.exists() else []


def decode(path, channels=2):
    """Returns the first CHANNELS channels of the file as lists of samples."""
    pan = "|".join(f"c{n}=c{n}" for n in range(channels))
    raw = subprocess.run(
        ["ffmpeg", "-loglevel", "error", "-i", str(path), "-af", f"pan={channels}c|{pan}",
         "-ar", str(RATE), "-f", "f32le", "-"],
        capture_output=True,
    ).stdout
    samples = array.array("f")
    samples.frombytes(raw[: len(raw) // 4 * 4])
    return [samples[n::channels] for n in range(channels)]


def db(samples):
    mean = sum(s * s for s in samples) / max(len(samples), 1)
    return 10 * math.log10(mean) if mean > 1e-14 else -140.0


def sound(path):
    """Returns the start, the end, and the length in seconds of the sound in
    the first channel of a file, and its level in dBFS RMS from one second
    after it starts to one second before it ends."""
    channel = decode(path)[0]
    frame = RATE // 10
    levels = [db(channel[i : i + frame]) for i in range(0, len(channel) - frame + 1, frame)]
    loud = [n for n, level in enumerate(levels) if level > -60]
    if not loud:
        return None
    first, last = loud[0], loud[-1]
    middle = channel[(first + 10) * frame : (last - 9) * frame]
    return first / 10, (last + 1) / 10, len(levels) / 10, db(middle) if middle else None


def log(path):
    """Returns the keys, updates, and opens of a log of cinq."""
    keys, updates, opens = [], [], []
    for line in lines(path):
        if line.startswith("K ") and "KeyPressed" in line:
            time, event = line[2:].split(" ", 1)
            key = re.search(r"key: (?:Character\(\"(.*?)\"\)|Named\((\w+)\))", event)
            keys.append((float(time), key.group(1) or key.group(2) if key else "?"))
        elif line.startswith("U "):
            time, rest = line[2:].split(" ", 1)
            message, _, state = rest.partition(" | ")
            state, _, sends = state.partition(" sends=")
            fields = dict(re.findall(r"(\w+)=(\[.*?\]|\S+)", state))
            fields["sends"] = sends
            updates.append((float(time), message, state, fields))
        elif line.startswith("O "):
            time, rest = line[2:].split(" ", 1)
            opens.append((float(time), rest))
    return keys, updates, opens


def say(check, show, verdict, evidence=""):
    print(f"{check}\t{show}\t{verdict}\t{evidence}")


def ports(path):
    """Returns (output port number, playback port number) of each connection
    that jack_lsp -c lists, whichever end it lists first."""
    found, port = set(), None
    for line in lines(path):
        name = line.strip()
        if not name:
            continue
        if not line.startswith(" "):
            port = name
            continue
        for a, b in ((port, name), (name, port)):
            out, play = re.search(r"out_(\d+)$", a or ""), re.search(r"system:playback_(\d+)$", b or "")
            if out and play:
                found.add((int(out.group(1)), int(play.group(1))))
    return sorted(found)


def jack(directory):
    expected = {
        "pair": {10: -13.5, 11: -19.5, 12: -28.0},
        "singles": {10: -16.0, 11: -22.0, 12: -28.0},
    }
    for show, want in expected.items():
        connected = ports(directory / f"connections-{show}.txt")
        wrong = [(o, p) for o, p in connected if p != o + 1]
        reached = {p for _, p in connected}
        say("1", show + " connections", "passed" if connected and not wrong and set(want) <= reached else "failed",
            "out_N -> playback_M: " + " ".join(f"{o}>{p}" for o, p in connected))
        levels = (lines(directory / f"levels-{show}.tsv") or [""])[0].split("\t")
        heard = {n: (None if v == "-" else float(v)) for n, v in enumerate(levels, 1) if v}
        bad = [
            n for n in heard
            if (n in want and (heard[n] is None or abs(heard[n] - want[n]) > 1.5))
            or (n not in want and heard[n] is not None)
        ]
        say("1", show + " sound", "passed" if len(heard) == 16 and not bad else "failed",
            "dBFS per playback port 1-16: " + " ".join("-" if heard[n] is None else f"{heard[n]:.1f}" for n in sorted(heard)))


def panic(directory, houses, phase):
    keys, _, _ = log(directory / f"{phase}.log")
    go = next((t for t, k in keys if k == "F5"), None)
    stop = next((t for t, k in keys if k == "Escape"), None)
    results = {name: sound(directory / f"{name}.wav") for name in houses}
    for name, found in results.items():
        if found is None:
            say("2", name, "failed", "no sound recorded")
            continue
        start, end, total, level = found
        held = stop - go if go and stop else None
        ok = held is not None and abs((end - start) - held) < 2.0 and total - end >= 1.5
        say("2", name, "passed" if ok else "failed",
            f"sound from {start:.1f} to {end:.1f} s of {total:.1f}, {end - start:.1f} s against {held and round(held, 1)} s between F5 and Escape, "
            f"level {level and round(level, 1)} dBFS, silent for the last {total - end:.1f} s")


def slow(directory):
    keys, updates, opens = log(directory / "slow.log")
    typed = [(t, k) for t, k in keys if k in ("j", "k")]
    pairs = []
    begins = {}
    for time, rest in opens:
        match = re.match(r"open (.*?) (begins|ends)", rest)
        if match and match.group(2) == "begins":
            begins[match.group(1)] = time
        elif match:
            pairs.append((begins.pop(match.group(1), time), time, match.group(1) + ("" if rest.endswith(" ok") else " failed")))
    say("4", "opens", "info", "; ".join(f"{d} {e - b:.2f} s from {b % 1000:.2f}" for b, e, d in pairs) or "no open logged")
    # Only an opening that succeeded and took a second or more is slow
    pairs = [(b, e, d) for b, e, d in pairs if e - b >= 1.0 and not d.endswith(" failed")]
    delays = []
    for time, key in typed:
        after = next((t for t, message, _, _ in updates if t >= time and message.startswith("Typed")), None)
        busy = any(b <= time <= e for b, e, _ in pairs)
        delays.append((time, key, None if after is None else after - time, busy))
    overlapped = [d for d in delays if d[3]]
    worst = max((d[2] for d in overlapped if d[2] is not None), default=None)
    say("4", "keys", "info",
        f"{len(typed)} keys; during an open: " + " ".join(f"{k}@{t % 1000:.2f}+{'-' if d is None else round(d * 1000)}ms" for t, k, d, b in overlapped))
    if len(typed) < 30:
        verdict = "failed"
        note = f"only {len(typed)} of 36 keys reached cinq"
    elif len(overlapped) < 3:
        verdict = "inconclusive"
        note = f"{len(overlapped)} keys fell in an open"
    elif worst is not None and worst < 0.25 and all(d[2] is not None for d in overlapped):
        verdict = "passed"
        note = f"{len(overlapped)} keys fell in an open, the slowest update came {worst * 1000:.0f} ms after its key"
    else:
        verdict = "failed"
        note = f"slowest update {worst} s after its key"
    say("4", "interface during opens", verdict, note)


def clipped(directory, phase, show):
    """Judges the recording of a phase of loud sounds, unless cinq could not
    open the device, which skips it."""
    opened = [l[2:].split(" ", 1)[1] for l in lines(directory / f"{phase}.log") if l.startswith("O ") and " ends " in l]
    last = opened[-1] if opened else "no open logged"
    if "Unplayable" in last:
        say("3", phase, "skipped", f"cinq cannot play on it: {last}")
        return
    file = directory / f"{phase}.wav"
    if not file.exists():
        say("3", phase, "failed", f"no recording; {last}")
        return
    result = subprocess.run(
        [sys.executable, "-I", str(Path(__file__).with_name("clip.py")), str(file), "single" if show == "single" else "clip"],
        capture_output=True, text=True,
    ).stdout.strip().split("\t")
    if len(result) != 4:
        say("3", phase, "failed", f"clip.py said {result}; {last}")
        return
    say("3", phase, result[3].split(":")[0], f"peak {result[0]}, largest step {result[1]}, {result[3]}; {last}")


def clamp(directory):
    for name in ("s16", "s24", "s24p", "s32", "float"):
        for show in ("single", "two", "boost"):
            clipped(directory, f"{name}-{show}", show)


def tsv(path):
    row = (lines(path) or [""])[0].split("\t")
    return [None if v in ("-", "") else float(v) for v in row]


def view(directory, phase):
    """Judges the keys of the outputs view and the Levels tab in a log."""
    keys, updates, _ = log(directory / f"{phase}.log")
    states = []
    for _, _, _, fields in updates:
        outputs = fields.get("outputs", "[]")
        state = (fields.get("shown") == "true", outputs.count("OutputId"))
        if not states or states[-1] != state:
            states.append(state)
    want = [(False, 2), (True, 2), (True, 3), (True, 2), (True, 3), (False, 3)]
    at = 0
    for state in states:
        if at < len(want) and state == want[at]:
            at += 1
    say("5", phase + " outputs view", "passed" if at == len(want) else "failed",
        f"(shown, outputs) after each change: {states}; keys seen: {''.join(k if len(k) == 1 else '<' + k + '>' for _, k in keys)}")
    # The last line may be cut where the process was stopped while writing it
    with_sends = [f["sends"] for _, _, _, f in updates if "Some" in f.get("sends", "") and f["sends"].rstrip().endswith("})")]
    names = {}
    for _, _, _, f in updates:
        names.update({i: n for i, n in re.findall(r"OutputId\((\d+)\)=([^@,\]]+)@", f.get("outputs", ""))})
    last = re.findall(r"OutputId\((\d+)\): (-?[\d.]+)", with_sends[-1]) if with_sends else []
    sends = {names.get(i, i): float(v) for i, v in last}
    say("5", phase + " Levels tab send", "passed" if sends.get("Right") == -12.0 and sends.get("Left") == 0.0 else "failed",
        f"sends of the cue after the last update: {sends}")
    before, after = tsv(directory / f"{phase}-before.levels.tsv")[:2], tsv(directory / f"{phase}-after.levels.tsv")[:2]
    playing = all(v is not None and v > -40 for v in before) if before else False
    say("5", phase + " one output plays", "passed" if playing else "failed",
        f"dBFS of channels 1 and 2 on the device before the change: {before}")
    delta = None if None in after or len(after) < 2 or None in before else (after[1] - after[0]) - (before[1] - before[0])
    say("5", phase + " send heard", "passed" if delta is not None and abs(delta + 12) < 1.5 else "failed",
        f"channels 1 and 2 after: {after}, change of their difference: {delta}")


def clip_phases(directory):
    for name in ("Int16", "Int24", "Int32"):
        for show in ("single", "two", "boost"):
            clipped(directory, f"asio-{name}-{show}", show)


def windows(directory):
    for phase in ("default", "cable"):
        view(directory, phase)
    clip_phases(directory)


def darwin(directory):
    for phase in ("default", "explicit"):
        view(directory, phase)
    panic(directory, ["two-house", "two-booth"], "two")


def pulse(directory):
    panic(directory, ["house", "booth"], "two")
    slow(directory)
    view(directory, "keys")


kind, directory = sys.argv[1], Path(sys.argv[2])
{"jack": jack, "pulse": pulse, "clamp": clamp, "windows": windows, "darwin": darwin}[kind](directory)
