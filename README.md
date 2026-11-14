<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: two Core Audio devices, one of 64 channels

This branch plays one cue of cinq on two Core Audio devices at once:
BlackHole 64ch, which has as many channels as the Dante Virtual
Soundcard offers on Darwin, and BlackHole 2ch beside it, as the
soundcard's WDM mode on Windows offers each pair of its channels as a
device of its own.  `cinq/` holds the tree of commit 70a247e of the
`audio-outputs` branch of yzhou216/cinq.

## What runs

`.github/workflows/coreaudio64.yml` installs both BlackHole drivers and
ffmpeg with Homebrew, sounddevice and numpy with pip, builds cinq with
`nix build ./cinq`, and runs these.

- `capture.py tones` plays a tone on each channel of BlackHole 64ch
  through PortAudio, each 0.5 dB below the one before, and prints the
  level that came back on each, which checks the loopback and the
  recorder without cinq.
- `multi.sh` writes a show whose outputs are House on channels 1-2 of
  BlackHole 2ch, and Sub on 3, Monitors on 33-34, and Lobby on 63-64 of
  BlackHole 64ch, with a cue that sends to them at 0, 0, -6, and -12 dB.
  It records each device with `capture.py record`, each in a process of
  its own, and fires the cue with a key.
- `levels.sh` prints the RMS level of each channel, and `onset.sh` the
  time at which a channel first sounds, from which the workflow prints how
  far apart the two devices started the cue.

## Results

Run 43545083339 on `macos-latest` passed.

- Every one of the 64 tones came back at the level it was sent, from
  -13.5 dB on channel 1 to -45.0 dB on channel 64.
- BlackHole 2ch played House at -16.5 dB on both channels.  BlackHole 64ch
  played Sub at -16.5 dB on channel 3, Monitors at -22.5 dB on 33-34, and
  Lobby at -28.5 dB on 63-64, and nothing on its other 59 channels.
- BlackHole 64ch started the cue 6.4 ms after BlackHole 2ch, about as
  closely as the recorders can tell, since each dates its first sample
  from when its first block of samples arrives.
- Run 43513452443 before it recorded both devices with one ffmpeg, whose
  avfoundation input starved them, keeping 2.3 and 3.9 s of 19, and gave
  every channel of BlackHole 64ch the same level, though the loopback
  keeps the channels apart, as capture.py then showed.
