<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: scheduling the thread of sound on Windows

This branch tells whether cinq on WASAPI, as on the Dante Virtual
Soundcard in WDM mode, needs its thread of sound scheduled for audio.
`cinq/` holds the tree of commit 8d260b2 of the `audio-outputs` branch of
yzhou216/cinq, whose thread keeps its usual priority, and `cinq-rt/` the
same tree built with cpal's feature `realtime`, which asks Windows to
schedule that thread for audio, through MMCSS or else at time-critical
priority.

## What runs

`.github/workflows/realtime.yml` cross-builds both trees for Windows on
GNU/Linux, installs VB-CABLE, and runs `stress.ps1`, which plays a tone of
1 kHz on VB-CABLE with each build for ten seconds while eight loops keep
the four processors of the runner busy, and records it through WASAPI's
loopback at high priority, so that the recorder itself keeps up.
`glitch.sh` and a count of the quarters of a second whose rest rises
above -80 dBFS tell how clean the tone stayed.

## Results

Run 43583510112 passed.

| Build    | Thread of sound          | Left, Right  | Rest below the tone | Quarters spoiled |
| -------- | ------------------------ | ------------ | ------------------- | ---------------- |
| usual    | normal priority          | -30.1, -36.1 | 70.0 dB             | 0 of 34          |
| realtime | time-critical, priority 21 | -30.1, -36.1 | 70.0 dB           | 0 of 34          |

- The feature works on Windows: one thread of the realtime build ran at
  time-critical priority, where every thread of the usual build ran at
  normal priority.
- Neither build fell behind under eight loops of normal priority, so this
  load cannot tell them apart.  The branch `cinq/2026-11-13-linux-quirks`
  saw the thread of ALSA fall behind on two busy processors in a virtual
  machine, where the feature changes nothing without RTKit.
