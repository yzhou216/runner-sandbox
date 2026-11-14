<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: channels through PipeWire

This branch tells where the channels of cinq's outputs sound when cinq
plays through PipeWire, as a GNU/Linux desktop does, either through
PipeWire's JACK, as `jack:output`, or through PipeWire's plugin of ALSA,
as `alsa:pipewire`.  `cinq/` holds the tree of commit 70a247e of the
`audio-outputs` branch of yzhou216/cinq.

## What runs

`.github/workflows/pipewire.yml` builds cinq with `nix build ./cinq`, and
PipeWire, WirePlumber, sway, wtype, grim, and ffmpeg from the Nixpkgs of
cinq, and runs `pipewire.sh` in a session bus of its own.

- `pipewire.sh` starts PipeWire with a null sink of eight channels as its
  default sink, first on the positions AUX0 to AUX7, as a desk's channels
  are, then on those of 7.1, as a sound card's are.
- On each sink it plays shows of 3, 4, 6, and 8 outputs, Ch K on channel
  K, to which the cue sends 3 (K - 1) dB down, each on `jack:output` and on
  `alsa:pipewire`, and records every channel of the sink with pw-record.
- The workflow prints the level of each channel of the sink for each.

## Results

Run 43547535989 passed, with these levels in dBFS, "-" for silence.  Ch K
should read -16.5 - 3 (K - 1) on channel K.

| Sink, path, outputs | 1     | 2     | 3     | 4     | 5     | 6     | 7     | 8     |
| ------------------- | ----- | ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| desk, JACK, 8       | -16.5 | -19.5 | -22.5 | -25.5 | -28.5 | -31.5 | -34.5 | -37.5 |
| desk, ALSA, 3       | -16.5 | -19.5 | -     | -     | -     | -     | -     | -     |
| desk, ALSA, 4       | -13.8 | -16.8 | -     | -     | -     | -     | -     | -     |
| desk, ALSA, 8       | -12.0 | -14.5 | -     | -     | -     | -     | -     | -     |
| 7.1, JACK, 8        | -16.5 | -19.5 | -22.5 | -25.5 | -28.5 | -31.5 | -34.5 | -37.5 |
| 7.1, ALSA, 3        | -16.5 | -19.5 | -22.5 | -     | -     | -     | -     | -     |
| 7.1, ALSA, 4        | -16.5 | -19.5 | -     | -     | -22.5 | -25.5 | -     | -     |
| 7.1, ALSA, 8        | -16.5 | -19.5 | -28.5 | -31.5 | -22.5 | -25.5 | -34.5 | -37.5 |

- Through JACK every channel sounded where its output put it, on both
  sinks and with 3, 4, and 6 outputs as well.
- Through the plugin of ALSA the channels past the second go to the
  places of surround sound.  On the desk's sink only the first two
  sounded: the third of three was lost, and more were folded into the
  first two, which grew louder.  On the 7.1 sink, the third and fourth of
  four sounded on the fifth and sixth, as ALSA's map of four channels
  puts them behind.  pw-dump showed that cinq opened as many channels as
  its outputs need, which the plugin gave no positions.
- cinq showed no warning either way, since the plugin takes the channels
  and moves them itself.  cinq(5) now tells this, in commit a15abec, and
  points to `jack:output` under PipeWire.
- Runs 43511999784 to 43546468465 came before.  They took the wrong link
  for PipeWire's libjack, lacked the configuration of Nix's dbus, and
  recorded only the first two channels of the sink.
