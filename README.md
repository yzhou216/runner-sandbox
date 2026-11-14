<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: WASAPI and ASIO devices that go away and come back

This branch plays cinq on Windows the two ways it can reach the Dante
Virtual Soundcard, which no runner can run, since it is licensed per
machine: on a WASAPI device, as the soundcard's WDM mode offers each pair
of its channels, and on an ASIO driver, as its ASIO mode offers all of
them.  VB-CABLE, a virtual WASAPI device, and FlexASIO, an open ASIO
driver that plays on VB-CABLE in 32-bit integers, stand in for them.
Midway through the cue the device goes away and comes back, as the
soundcard does when it restarts.  `cinq/` holds the tree of commit 70a247e
of the `audio-outputs` branch of yzhou216/cinq.

## What runs

`.github/workflows/wasapi.yml` cross-builds cinq for Windows on GNU/Linux
with `nix build ./cinq#x86_64-windows`, which builds it with ASIO, after
applying `open-timing.patch`, which logs on standard error how long each
opening of a device takes.  On `windows-latest`, LABSN/sound-ci-helpers
installs VB-CABLE, the job starts the audio service and installs FlexASIO
with `FlexASIO.toml`, and `wasapi.ps1` runs twice, once on VB-CABLE and
once on FlexASIO.

- `wasapi.ps1` writes a show whose outputs are Left on channel 1 and Right
  on channel 2, with a cue that sends to them at 0 and -6 dB, fires the cue
  with a key, and records VB-CABLE through WASAPI's loopback with
  `loopback.py`.
- Midway through, it disables the endpoint of VB-CABLE with NirSoft's
  SoundVolumeView, as the Sound control panel would, enables it again, and
  records, then stops the audio service of Windows, starts it again, and
  records once more.
- `levels.sh` prints the RMS level of each channel of each recording.

## Results

Run 43546277276 passed.  The music is -16.5 dBFS on each channel at 0 dB.

| Take                            | WASAPI        | ASIO          |
| ------------------------------- | ------------- | ------------- |
| Before                          | -16.5 / -22.5 | -16.5 / -22.5 |
| After the endpoint came back    | -16.5 / -22.5 | -16.5 / -22.5 |
| After the audio service came back | -16.5 / -22.5 | -16.5 / -22.5 |

- While the device was gone, cinq showed a warning on each output and the
  clock of the cue ran on.  Each time it came back, cinq took it up again
  by itself, and the cue played on there from where it was.
- An opening of a device took 16 to 101 ms when it succeeded, and 0 ms on
  WASAPI and 10 to 85 ms on FlexASIO when the device was gone.  cinq tries
  again every two seconds on the thread of its window.
- While the endpoint of VB-CABLE was disabled, `cinq --audio-devices`
  listed nothing, not even FlexASIO, which could not start without it.
- Run 43545351280 before it passed the same way without the timing patch.
  Runs 43512437619 and earlier removed VB-CABLE's device node with devcon,
  which Windows defers until a reboot while the device plays: the device
  that was playing stayed and kept playing, and a second one appeared
  beside it.
