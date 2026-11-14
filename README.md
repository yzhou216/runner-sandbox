<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: sixteen loops on 64 channels

This branch plays sixteen looping cues at once on all 64 channels of a
JACK server, as many channels as the Dante Virtual Soundcard offers, for
five minutes with each of two builds of cinq, and counts what the server
missed.  `before/` holds the tree of commit e278505 of the `audio-outputs`
branch of yzhou216/cinq, and `after/` that of commit 6e43900 of its
`resampler-prototype` branch, which converts sounds of another rate than
the device by rubato's windowed sinc interpolation.

## What runs

`.github/workflows/soak.yml` builds both trees with `nix build`, and runs
`soak.sh`, then `levels.sh`.

- `soak.sh` starts jackd with its dummy backend at 48 kHz, with periods of
  256 frames and 64 playback ports, and writes a show of 32 stereo outputs
  over the 64 channels and sixteen cues that loop a file of 44.1 kHz, each
  sending to two outputs at 0 and -6 dB, which one GO fires.
- For each build it fires the cue, waits five minutes, and prints the
  share of one processor that cinq took meanwhile, window and all, and the
  xruns that jackd logged, then records the 64 ports with jack_capture.
- `levels.sh` prints the RMS level of each port of that recording.

## Results

Run 43552711060 on `ubuntu-latest` passed.

| Build  | Processor | Xruns |
| ------ | --------- | ----- |
| before | 23.1%     | 0     |
| after  | 31.3%     | 0     |

- Neither build missed a period of the server in five minutes, at 5.3 ms
  a period.
- Converting sixteen sounds by windowed sinc interpolation took 8.2% more
  of a processor, about half a percent for each sound.
- At the end, every port played at its level with both builds: -16.5 dB
  on those of the outputs sent at 0 dB, and -22.5 dB on those sent at
  -6 dB.
