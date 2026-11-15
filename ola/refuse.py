# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Fire the network cues of refuse.toml, one GO each five seconds apart, and
record what cinq says on standard error, with its time after each GO, and
the window of cinq after each (sidebar warnings).

Usage: refuse.py OUT SHOW WORK, started by gateway.sh in its sway session
"""

import subprocess
import sys
import threading
import time
from pathlib import Path

out, show = Path(sys.argv[1]), sys.argv[2]
lines = []
verdict = open(out / "verdict.txt", "a", buffering=1)


def say(text):
    print(text, flush=True)
    print(text, file=verdict)


proc = subprocess.Popen(["cinq", show], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
threading.Thread(target=lambda: [lines.append((time.time(), line.rstrip())) for line in proc.stderr], daemon=True).start()
time.sleep(8)
subprocess.run(["swaymsg", "-q", '[app_id="cinq"] focus'])
for number in range(1, 6):
    subprocess.run(["swaymsg", "-q", '[app_id="cinq"] focus'])
    before = time.time()
    subprocess.run(["wtype", "-s", "300", " "])
    after = time.time()
    time.sleep(4)
    subprocess.run(["grim", str(out / f"go-{number}.png")])
    new = [(t, text) for t, text in lines if t >= before]
    if not new:
        say(f"GO {number}: nothing on standard error")
    for t, text in new:
        # The effect of a GO came 30 ms after wtype ended in gateway.py, so
        # the time that wtype ended stands for the key
        say(f"GO {number}: {t - after:+.2f} s after wtype ended: {text}")
    lines.clear()
    time.sleep(1)
subprocess.run(["swaymsg", '[app_id="cinq"] kill'])
proc.wait(timeout=10)
