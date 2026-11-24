# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Writes two sine tones as WAV files into DIRECTORY: sounds.py DIRECTORY
import math
import struct
import sys
import wave

for name, hz, seconds in (("tone", 440, 4), ("two", 330, 6)):
    with wave.open(f"{sys.argv[1]}/{name}.wav", "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(44100)
        w.writeframes(b"".join(
            struct.pack("<h", int(8000 * math.sin(2 * math.pi * hz * i / 44100)))
            for i in range(44100 * seconds)))
