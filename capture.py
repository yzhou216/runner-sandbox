# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

"""Records every channel of an input device through PortAudio until
stopped, or plays a tone on every channel of a device and records it.

Usage: capture.py record NAME FILE, which writes FILE in 32-bit samples
and prints the time of the host's clock of its first sample, or
capture.py tones NAME, which plays each channel 0.5 dB below the one
before and prints the level that came back on each, with NAME part of
the name of the device.
"""

import signal
import sys
import threading
import time
import wave

import numpy
import sounddevice

RATE = 48000


def device(name, kind):
    found = [
        index
        for index, info in enumerate(sounddevice.query_devices())
        if name in info["name"] and info[f"max_{kind}_channels"] > 0
    ]
    if not found:
        sys.exit(f"no {kind} device named {name}")
    return found[0]


def write(path, data):
    samples = (numpy.clip(data, -1.0, 1.0) * (2**31 - 1)).astype("<i4")
    with wave.open(path, "wb") as out:
        out.setnchannels(samples.shape[1])
        out.setsampwidth(4)
        out.setframerate(RATE)
        out.writeframes(samples.tobytes())


def record(name, path):
    index = device(name, "input")
    channels = sounddevice.query_devices(index)["max_input_channels"]
    blocks, first = [], []
    done = threading.Event()
    signal.signal(signal.SIGINT, lambda *_: done.set())
    signal.signal(signal.SIGTERM, lambda *_: done.set())

    def take(data, frames, when, status):
        if not first:
            first.append(time.time() - frames / RATE)
        blocks.append(data.copy())

    with sounddevice.InputStream(
        device=index, channels=channels, samplerate=RATE, dtype="float32", callback=take
    ):
        print(f"recording {channels} channels", flush=True)
        done.wait()
    write(path, numpy.concatenate(blocks))
    print(f"start {first[0]:.4f}", flush=True)


def tones(name):
    index = device(name, "output")
    channels = sounddevice.query_devices(index)["max_output_channels"]
    t = numpy.arange(RATE * 4) / RATE
    levels = 0.3 * 10 ** (-numpy.arange(channels) * 0.5 / 20)
    played = numpy.outer(numpy.sin(2 * numpy.pi * 440 * t), levels).astype("float32")
    heard = sounddevice.playrec(
        played, samplerate=RATE, device=(device(name, "input"), index), channels=channels, blocking=True
    )
    rms = lambda data: 20 * numpy.log10(numpy.sqrt(numpy.mean(data**2, axis=0)) + 1e-12)
    for channel, (sent, back) in enumerate(zip(rms(played[RATE:-RATE]), rms(heard[RATE:-RATE])), 1):
        print(f"{channel}\t{sent:.1f}\t{back:.1f}")


if sys.argv[1] == "record":
    record(sys.argv[2], sys.argv[3])
else:
    tones(sys.argv[2])
