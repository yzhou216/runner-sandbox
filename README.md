<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: an emulated Dante network

This branch plays cinq through an emulated Dante network, as a theater
routes it from the Dante Virtual Soundcard to its sound board.  The
soundcard is licensed per machine and needs an Audinate account, so no
runner can run it.  Inferno, an unofficial implementation of Dante for
GNU/Linux that Audinate does not endorse, stands in for both ends: one
Inferno device, `cinq`, with eight transmit channels, is what cinq plays
on, and another, `board`, with eight receive channels, is the board.
netaudio, an open controller, subscribes each receive channel of the board
to the transmit channel of cinq with its number, as Dante Controller
would.  `cinq/` holds the tree of commit c835d63 of the `audio-outputs`
branch of yzhou216/cinq.

## What runs

`.github/workflows/dante.yml` builds cinq with `nix build ./cinq`, builds
Inferno with `inferno.sh`, installs netaudio 0.5.7 in a virtual
environment, puts a dummy interface `dante0` on 10.77.0.1 for the devices
to bind to, and runs `dante.sh`, then `measure.sh`.

- `inferno.sh` builds Inferno's ALSA plugin at commit 9767558e, with
  `inferno-prepare-pollout.patch`, and the fake clock server of its tests,
  which stands in for PTP.
- `dante.sh` records the board with arecord for the whole run.  It plays a
  tone on each channel with aplay first, each 3 dB below the one before,
  then plays `desk.cinq.toml` on Inferno's device itself in 32-bit integers
  and routes its cue from the main window with keys, step by step, then
  plays `partial.cinq.toml`, whose outputs need three channels alone,
  through ALSA's `plug`, in floating point.
- `measure.sh` lines the recording up with the marks of each step by the
  onset of the music after the first GO, and prints the RMS level of each
  channel of the board between each step and the next.

## The patch

Without it, cinq's stream never started: the board's flow stayed
"Establishing flow", and Inferno logged that it could not get the start
timestamp.  Inferno's plugin tells a playback stream writable only once it
has started or has moved samples, while cpal, like many programs, polls
before it writes the first samples that start the stream, so both wait.
The patch tells a prepared playback stream writable until it starts.  It
is upstream as teodly/inferno#59.

## Results

Run 43511682488 on `ubuntu-latest` passed, with the table below in dBFS,
"-" for silence.  The music is -16.5 dBFS on each channel at 0 dB.

| Step                          | 1     | 2     | 3     | 4     | 5     | 6     | 7 | 8 |
| ----------------------------- | ----- | ----- | ----- | ----- | ----- | ----- | - | - |
| Tones from aplay              | -11.0 | -14.1 | -17.1 | -20.1 | -23.1 | -26.1 | -29.1 | -32.1 |
| GO: House on 1-2              | -16.5 | -16.5 | -     | -     | -     | -     | - | - |
| Sub on 3 added                | -16.5 | -16.5 | -16.5 | -     | -     | -     | - | - |
| Monitors on 5-6 at -6 dB      | -16.5 | -16.5 | -16.5 | -     | -22.5 | -22.5 | - | - |
| House off                     | -     | -     | -16.5 | -     | -22.5 | -22.5 | - | - |
| Sub moved to 4                | -     | -     | -     | -16.5 | -22.5 | -22.5 | - | - |
| Panic                         | -     | -     | -     | -     | -     | -     | - | - |
| Partial show through `plug`   | -16.5 | -16.5 | -16.5 | -     | -     | -     | - | - |

- Each step reached exactly its channels of the board, at the level of
  its send, and no other.
- The board kept its subscriptions by name while cinq's device was away
  between the shows, as Dante devices do, and they came back by
  themselves once cinq's device showed again.
- Runs 43412756236 to 43508448445 came before.  They found the poll
  deadlock and tried two forms of the patch, and the last of them passed,
  though its `measure.sh` misplaced the steps, since ffmpeg tells times
  after a seek from the point it seeks to.
