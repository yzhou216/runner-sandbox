<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: converting sample rates before and after rubato

This branch compares how two builds of cinq play a file whose sample rate
differs from that of the device, as a file of 44.1 kHz does on a Dante
network of 48 kHz.  `before/` holds the tree of commit e278505 of the
`audio-outputs` branch of yzhou216/cinq, whose sounds kira converts by
cubic interpolation without a filter, and `after/` that of commit 6e43900
of its `resampler-prototype` branch, which converts them by rubato's
windowed sinc interpolation.

## What runs

`.github/workflows/resampler.yml` builds both trees with `nix build`, and
runs `resample.sh`, then `rest.sh`.

- `resample.sh` writes tones at half of full scale as WAV files in 32-bit
  floating point: 10 and 15 kHz at 44.1 kHz, and 10 kHz at 48 kHz, which
  needs no conversion.  It starts jackd with its dummy backend at 48 kHz,
  plays each tone with each build on `jack:output`, and records the
  playback ports with jack_capture.
- `rest.sh` prints the RMS level of the left channel, and that of what is
  left once four notches at the frequency of the tone take it away.

## Results

Run 43550681949 on `ubuntu-latest` passed, with these levels in dBFS.

| Build  | File            | Tone  | Rest  | Rest below the tone |
| ------ | --------------- | ----- | ----- | ------------------- |
| before | 10 kHz, 44.1 kHz | -30.5 | -56.8 | 26.3 dB            |
| before | 15 kHz, 44.1 kHz | -31.6 | -45.5 | 14.0 dB            |
| before | 10 kHz, 48 kHz   | -30.1 | -70.6 | 40.5 dB            |
| after  | 10 kHz, 44.1 kHz | -30.1 | -70.6 | 40.5 dB            |
| after  | 15 kHz, 44.1 kHz | -30.1 | -68.6 | 38.5 dB            |
| after  | 10 kHz, 48 kHz   | -30.1 | -70.6 | 40.5 dB            |

- The file of 48 kHz, which neither build converts, sets the floor of the
  measurement: the notches leave the rest 40.5 dB below the tone.
- Before rubato, cubic interpolation took 0.4 dB off the tone of 10 kHz
  and 1.5 dB off that of 15 kHz, and left the rest 26.3 and 14.0 dB below
  them, as a simulation of kira's formula foretold: 26.4 and 13.8 dB.
- After rubato, both tones kept their level, and the rest lay at the
  floor of the measurement.  cinq's unit test, which needs no notch, puts
  it 89 dB below a tone of 10 kHz.
