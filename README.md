<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the quirks of Windows audio devices

This branch plays cinq on VB-CABLE, a virtual WASAPI device, and on
FlexASIO, an open ASIO driver that plays on it, under the quirks that the
Dante Virtual Soundcard and other devices can show on Windows.  `cinq/`
holds the tree of commit 226e193 of the `audio-outputs` branch of
yzhou216/cinq, which the workflow cross-builds with ASIO and
`open-timing.patch`, which logs on standard error how long each opening of
a device takes.

## What runs

`.github/workflows/windows-quirks.yml` cross-builds `cinq.exe` on
GNU/Linux, installs VB-CABLE, ffmpeg, soundcard, and FlexASIO on
`windows-latest`, and runs `quirks.ps1`, which plays a tone of 1 kHz, Left
at 0 dB and Right at -6 dB, and records VB-CABLE through WASAPI's loopback
with `loopback.py`.  `glitch.sh` prints the level of a channel, the lowest
level of any quarter of a second, and how far below the tone the rest lies
once notches take the tone away.

- asio-*: FlexASIO's configuration file changes midway through the cue,
  to each type of sample and to buffers of 64, 441, 480, and 4096 frames.
  FlexASIO reloads it and asks cinq to reset, as the Dante Virtual
  Soundcard's driver does when its settings change.
- asio-listing: the list of devices of an output opens and closes, with
  clicks, while FlexASIO plays.  cpal lists ASIO drivers by loading each,
  the one that plays among them.
- exclusive: FlexASIO in WASAPI's exclusive mode, held by another cinq
  until that one quits.
- overrun: Left and Right on VB-CABLE, which has two channels, beside an
  output on its channel 3.
- fallback: a show made on another machine, whose outputs name an endpoint
  that this one lacks, along with the name of VB-CABLE's.
- stopped: cinq started while the audio service of Windows is stopped,
  which then starts.

## Results

Run 43573972183 passed.  The tone is -30.1 dBFS at 0 dB, and the 16-bit
recordings of WASAPI's loopback set the floor of the rest at about 70 dB.

| Take                                  | Left, Right  | Rest below the tone |
| ------------------------------------- | ------------ | ------------------- |
| asio-Int32, Float32                   | -30.1, -36.1 | 70.0 dB             |
| asio-Int24                            | -30.1, -36.1 | 68.4 dB             |
| asio-Int16                            | -30.1, -36.1 | 64.5 dB             |
| asio-buffer-64, 441, 480, 4096        | -30.1, -36.1 | 70.0 dB             |
| asio-listing                          | -30.1, -36.1 | 70.0 dB             |
| overrun                               | -30.1, -36.1 | 70.0 dB             |
| fallback                              | -30.1, -36.1 | 70.0 dB             |
| stopped                               | -30.1, -36.1 | 70.0 dB             |

- Each of the seven changes of FlexASIO's configuration made cinq open the
  driver again, in 37 to 55 ms, with no click in any quarter.  Int16 lies
  closer to the floor, as cinq plays 16-bit samples there.
- The list of devices opened over the playing driver and named it, and
  the tone stayed clean.
- overrun: Left and Right played, and the output on channel 3 alone
  warned, as commit b4e792d of cinq made it.
- exclusive: the second cinq warned that the hardware of FlexASIO
  malfunctions, which is how cpal tells ASIO's error, and took the driver
  two seconds after the first quit.  FlexASIO takes exclusive mode on
  VB-CABLE only in 16 bits: in 32 bits neither cinq could open it, and
  cinq blamed its samples, which commit b4e792d reworded.
- fallback: cinq found VB-CABLE by its name and played on it.
- stopped: cinq warned that the device is not there, tried again every two
  seconds, and played once the service started.
- Runs 43572076799 and 43573106919 came before.  ffmpeg read their WAV
  files of a steady tone as MPEG-TS until the scripts named the format.
