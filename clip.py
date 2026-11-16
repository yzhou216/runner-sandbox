# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Judges whether the peaks of a recording of a sine clipped or wrapped.

Usage: clip.py FILE EXPECTED..., which reads the first channel of FILE
from 1 s to 4 s through ffmpeg and prints, tab-separated, the peak in full
scale, the largest step between neighboring samples, the share of samples
within 1 % of the peak, and the verdict.  A sine of 100 Hz at 48 kHz steps
0.013 at most per unit of amplitude, so a step past 0.5 is a wrap, where a
sample that should peak positive reads negative.  EXPECTED is "single" for
a sound that stays under full scale, whose peak must read 0.75 to 0.85, or
"clip" for one past it, whose peak must read 0.99 or more with no wrap.
"""

import array
import subprocess
import sys

path, expected = sys.argv[1], sys.argv[2]
raw = subprocess.run(
    ["ffmpeg", "-loglevel", "error", "-ss", "1", "-t", "3", "-i", path,
     "-af", "pan=mono|c0=c0", "-f", "f32le", "-"],
    check=True, capture_output=True,
).stdout
samples = array.array("f")
samples.frombytes(raw)
if len(samples) < 48000:
    print(f"{len(samples)} samples\t\t\tfailed: too little sound recorded")
    sys.exit(0)
peak = max(abs(sample) for sample in samples)
step = max(abs(b - a) for a, b in zip(samples, samples[1:]))
near = sum(abs(sample) >= peak * 0.99 for sample in samples) / len(samples)
if step > 0.5:
    verdict = "failed: wrapped"
elif expected == "single":
    verdict = "passed" if 0.75 <= peak <= 0.85 else f"failed: peak {peak:.3f}, not 0.8"
elif peak >= 0.99:
    verdict = "passed: clipped, no wrap"
else:
    verdict = "inconclusive: no wrap but peak below full scale"
print(f"{peak:.4f}\t{step:.4f}\t{near:.3f}\t{verdict}")
