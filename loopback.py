# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Records what a playback device plays, through WASAPI's loopback, which
needs no access to the microphone.

Usage: loopback.py NAME SECONDS FILE, with NAME part of the name of the
device, and FILE the WAV file to write, in 16-bit samples.
"""

import sys
import wave

import numpy
import soundcard

name, seconds, path = sys.argv[1], float(sys.argv[2]), sys.argv[3]
rate = 48000
speaker = next(s for s in soundcard.all_speakers() if name in s.name)
loop = soundcard.get_microphone(id=str(speaker.name), include_loopback=True)
print(f"recording {loop.name} with {loop.channels} channels", flush=True)
data = loop.record(samplerate=rate, numframes=int(rate * seconds))
samples = (numpy.clip(data, -1.0, 1.0) * 32767).astype("<i2")
with wave.open(path, "wb") as out:
    out.setnchannels(samples.shape[1])
    out.setsampwidth(2)
    out.setframerate(rate)
    out.writeframes(samples.tobytes())
