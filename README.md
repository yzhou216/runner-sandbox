<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the quirks of Core Audio devices

This branch plays cinq on BlackHole 16ch and BlackHole 2ch under the
quirks that a Core Audio device, such as the Dante Virtual Soundcard, can
show while a show runs.  `cinq/` holds the tree of commit 226e193 of the
`audio-outputs` branch of yzhou216/cinq, with `open-timing.patch`, which
logs on standard error how long each opening of a device takes.

## What runs

`.github/workflows/darwin-quirks.yml` installs both BlackHole drivers,
ffmpeg, and sounddevice, builds cinq with `nix build ./cinq`, and runs
`quirks.sh`, which plays a tone of 1 kHz and records it with `capture.py`.
`coreaudio.swift` sets the sample rate of a device, as Dante Controller
does, and the default output of the system, as System Settings does.
`glitch.sh` prints the level of a channel, the lowest level of any quarter
of a second, and how far below the tone the rest lies once notches take
the tone away: a dropout lowers the lowest quarter, and a click, a gap, or
a tone of the wrong pitch raises the rest.

- overrun: Low on channels 1-2 of BlackHole 16ch, and High on 17-18,
  past its channels, as a typo in an output makes.
- fallback: a show made on another Mac, whose outputs name a device ID
  that this one lacks, along with the name of BlackHole 16ch.
- rate: BlackHole 16ch goes from 48 to 96 kHz midway through the cue.
- default: an output on the default device, while the default output of
  the system goes from BlackHole 2ch to BlackHole 16ch midway through.

## Results

Runs 43571889587, 43573633375, and 43574783713 passed.  The tone is -30.1
dBFS at 0 dB.

| Take                     | Channels                       | Rest below the tone |
| ------------------------ | ------------------------------ | ------------------- |
| overrun                  | 1-2 at -30.1, nothing else     | 79.9 dB             |
| fallback                 | 1 to 3 at -30.1, -33.1, -36.1  | 79.9 dB             |
| rate, before             | 1-2 at -30.1                   | 79.9 dB             |
| rate, after              | 1-2 at -30.2                   | see below           |
| default, before          | BlackHole 2ch at -30.1         | 79.9 dB             |
| default, after           | BlackHole 16ch at -30.1        | 79.9 dB             |

- overrun: with the tree of 226e193, Low played and High alone warned.
  Before commit b4e792d of cinq, the device failed to open for both, so
  that a typo in one output silenced every output of its device.
- fallback: cinq found BlackHole 16ch by its name and played on it.
- rate: cinq opened BlackHole 16ch again once, in 32 ms, at 96 kHz.  The
  recordings after the change showed dips of about 100 ms, but always
  about 0.7 s after their recorder started or just before it stopped.
  Run 43574783713 recorded twice, 1.5 s apart: the second recorder's dip
  came while the first heard the tone unbroken, so the dips are those of
  PortAudio's recorder at 96 kHz, not of cinq.
- default: cinq followed the default output of the system to BlackHole
  16ch.
