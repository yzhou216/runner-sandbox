<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: a Core Audio device of sixteen channels

This branch plays cinq on BlackHole 16ch, a virtual Core Audio device that
stands in for the Dante Virtual Soundcard, which is a Core Audio device of
up to 64 channels on Darwin, and takes the device away and gives it back
in the middle of a cue, as the soundcard does when it restarts.  `cinq/`
holds the tree of commit 9bd4e16 of the `audio-outputs` branch of
yzhou216/cinq, which first opened devices that stop again on their own.

## What runs

`.github/workflows/coreaudio.yml` installs BlackHole 16ch and ffmpeg with
Homebrew, builds cinq with `nix build ./cinq`, and runs `coreaudio.sh`.

- `coreaudio.sh` writes a show whose outputs play on BlackHole: House on
  channels 1-2, Sub on 3, Monitors on 5-6, and Lobby on 15-16, with a cue
  that sends to them at 0, 0, -6, and -12 dB.  It records the sixteen
  channels that BlackHole loops back with ffmpeg's avfoundation input, which
  the job may use since the runner grants its agents the microphone.
- After GO, it moves BlackHole's driver out of the plug-ins of Core Audio
  and restarts `coreaudiod`, waits, puts the driver back, restarts
  `coreaudiod` again, and records once more.
- `levels.sh` prints the RMS level of each channel.

## Results

Run 43418928719 on `macos-latest` passed.  Run 43416739809 before it
started cinq by a relative path from another directory, so cinq never ran.

| Take                        | 1-2   | 3     | 4 | 5-6   | 7-14 | 15-16 |
| --------------------------- | ----- | ----- | - | ----- | ---- | ----- |
| Last 4 s before the device went | -18.4 | -18.4 | - | -24.4 | -    | -30.4 |
| After the device came back  | -16.5 | -16.5 | - | -22.5 | -    | -28.5 |

- Each output reached its own channels and no others, at the levels of its
  sends; the take before reads 1.9 dB low on every channel alike, since its
  last seconds include the recorder stopping.
- While the device was gone, cinq showed a warning on each output and the
  cue's clock ran on, from 0:48.5 to 0:43.1 left five seconds apart, rather
  than holding.  Once the device was back, the warnings cleared and the cue
  played on there from where it was, with no action from the operator.
- ffmpeg's audiotoolbox output, meant to check the recorder with tones,
  wrote nothing, which the run did not need in the end.
