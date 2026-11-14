<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the quirks of ALSA devices and JACK servers

This branch plays cinq on devices that stand in for the quirks of sound
cards on GNU/Linux, and on JACK servers of other rates, periods, and
lifetimes.  `cinq/` holds the tree of commit 8d260b2 of the
`audio-outputs` branch of yzhou216/cinq, and `cinq-rt/` the same tree
built with cpal's feature `realtime`.

## What runs

`.github/workflows/linux-quirks.yml` has three jobs.  Each plays a tone
of 1 kHz, and `glitch.sh` prints the level of a channel, the lowest level
of any quarter of a second, and how far below the tone the rest lies once
notches take the tone away: a dropout lowers the lowest quarter, and a
click, a gap, or a tone of the wrong pitch raises the rest.

- alsa: the kernels of the runners have no sound at all, as the branch
  `cinq/2026-11-13-kernel-sound` found, so `vm.nix` runs `alsa.sh` in a
  NixOS virtual machine with the kernel's loopback card.  There ALSA's hw
  plugin holds the card to one format of samples, one count of channels,
  or one rate, as the hw devices of sound cards are, under ALSA's copy
  plugin, which passes that on and, unlike a hw device of the
  configuration, lists the device for cpal.  arecord records the other end
  of the loopback.
- jack: `jack.sh` plays on jackd's dummy backend, and jack_capture
  records its playback ports.
- realtime: `vm-rt.nix` runs `rt.sh` in the virtual machine, which plays
  both trees on the hw device of the loopback card while four loops keep
  the two processors busy and grim takes screenshots.

## Results

Runs 43578194032 and 43581149548 passed.  The tone is -30.1 dBFS at
0 dB.

| ALSA device held to             | Channels 1 to 3       | Rest below the tone |
| ------------------------------- | --------------------- | ------------------- |
| nothing                         | -30.1, -33.1, -36.1   | 79.9 dB             |
| 16-bit samples                  | -30.1, -33.1, -36.1   | 70.0 dB             |
| 32-bit samples                  | -30.1, -33.1, -36.1   | 79.9 dB             |
| 32-bit floating point           | -30.1, -33.1, -36.1   | 79.9 dB             |
| packed 24-bit samples           | silent                |                     |
| packed 24-bit, through plug     | -30.1, -33.1, -36.1   | 80.0 dB             |
| six channels                    | -30.1, -33.1, -36.1   | 65.2 dB             |
| six, with an output on 7        | -30.1, -33.1, -36.1   | 65.2 dB             |
| 44.1 kHz                        | -30.1, -33.1, -36.1   | 64.1 dB             |
| 96 kHz                          | -30.1, -33.1, -36.1   | 65.2 dB             |

- cpal writes no packed 24-bit samples, which some USB interfaces alone
  take on their hw devices.  cinq warned with cpal's words, that the
  requested stream configuration is not supported by the device, until
  commit 8d260b2 named the plughw device of the card, which plays them.
- With an output on channel 7 of the device of six channels, the other
  three played, and that output alone warned, as commit b4e792d made it.
- The takes that arecord records through plughw from six channels, or at
  another rate, lie closer to the floor, since plughw converts them.
- In some runs, one or two takes of the first rows showed a dip at the
  instant that grim took its screenshot, which costs the two processors of
  the virtual machine a burst of work.  arecord reported no overrun, so
  the thread of cinq that writes to ALSA fell behind.

| JACK server                      | Channels 1-2 | Rest below the tone |
| -------------------------------- | ------------ | ------------------- |
| 44.1 kHz, 96 kHz                 | -30.1        | 80.8, 81.1 dB       |
| periods of 64, 1000, 4096 frames | -30.1        | 79.9 dB             |
| started after the cue fired      | -30.1        | 79.9 dB             |
| stopped and started mid-cue      | -30.1        | 79.9 dB             |
| two ports under four channels    | -30.1        | 79.9 dB             |

- On a server of two playback ports, cinq opened four and left the two
  past them unconnected, as cpal does for a patchbay to connect, with no
  warning, which commit 0b5607b tells in cinq(5).

| Build on busy processors | Thread of ALSA     | Quarters spoiled |
| ------------------------ | ------------------ | ---------------- |
| usual                    | normal scheduling  | 8 of 34          |
| realtime                 | normal scheduling  | 6 of 34          |

- Both builds fell behind under four busy loops on two processors.  The
  feature realtime of cpal did not change the scheduling of its thread of
  ALSA: on GNU/Linux it asks for real-time scheduling through RTKit, over
  D-Bus, only with its feature realtime-dbus, so the two rows measure the
  same build twice.
- Runs 43571702262 to 43577011379 came before.  The kernels of the
  runners lack snd-aloop, official Nix gives a flake of a path no date of
  its last change, which the flake of cinq takes, and plain hw devices of
  ALSA's configuration never list.
