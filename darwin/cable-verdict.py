# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Check the cues of cable.toml against what the stand-in console heard.

Usage: cable-verdict.py OUT
"""

import re
import sys

out = sys.argv[1]
marks = [line.rstrip("\n") for line in open(f"{out}/marks.txt")]
gos = []
for line in marks:
    stamp, _, rest = line.partition(" ")
    if rest.startswith("GO "):
        gos.append((float(stamp), rest[3:]))
arrived = {}
for line in open(f"{out}/console.txt"):
    match = re.match(r"(\S+) tcp (/\S+) (.*)", line)
    if match:
        arrived.setdefault(" ".join(re.findall(r"'([^']*)'", match[3])), float(match[1]))
never = {"while out", "blip"}
wrong = False
for when, name in gos:
    delay = arrived.get(name)
    shown = "never" if delay is None else f"{(delay - when) * 1000:.0f} ms after GO"
    good = delay is None if name in never else delay is not None and delay - when < 1
    wrong |= not good
    print(f"{'ok' if good else 'WRONG'}: {name}: {shown}")
print("marks:")
for line in marks:
    if " cable " in line or " nc " in line or "Allow" in line:
        print(" ", line)
print("warnings on standard error, seconds after the last GO:")
try:
    stamped = [line.rstrip("\n") for line in open(f"{out}/cinq.log.stamped")]
except OSError:
    stamped = []
for line in stamped:
    stamp, _, text = line.partition(" ")
    last = max((g for g in gos if g[0] <= float(stamp)), default=None, key=lambda g: g[0])
    print(f"  +{float(stamp) - last[0]:.3f} s after GO {last[1]}: {text}" if last else f"  {line}")
sys.exit(1 if wrong else 0)
