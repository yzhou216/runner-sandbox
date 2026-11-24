# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Writes a show of COUNT audio cues into DIRECTORY, each playing a copy of
# a one second tone under a name of its own (sounds/f001.wav, ...), and a
# five second tone in spare/long.wav: mkshow.py DIRECTORY COUNT
import math
import os
import shutil
import struct
import sys
import wave


def tone(path, hz, seconds):
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(44100)
        w.writeframes(b"".join(
            struct.pack("<h", int(8000 * math.sin(2 * math.pi * hz * i / 44100)))
            for i in range(44100 * seconds)))


directory, count = sys.argv[1], int(sys.argv[2])
os.makedirs(f"{directory}/sounds", exist_ok=True)
os.makedirs(f"{directory}/spare", exist_ok=True)
tone(f"{directory}/spare/base.wav", 440, 1)
tone(f"{directory}/spare/long.wav", 330, 5)
cues = []
for i in range(1, count + 1):
    shutil.copyfile(f"{directory}/spare/base.wav", f"{directory}/sounds/f{i:03d}.wav")
    cues.append(
        f'[[list.cue]]\nid = {i}\nnumber = "{i}"\nkind = "audio"\n'
        f'name = "Cue {i}"\nfile = "sounds/f{i:03d}.wav"\nlevel = -12.0\n')
with open(f"{directory}/show.cinq.toml", "w") as f:
    f.write('[[list]]\nname = "Main"\n\n' + "\n".join(cues))
