<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: the device of a JACK server

This branch plays cinq on `jack:output`, the device of a JACK server, which
connects each channel that cinq opens to the playback port with its number,
where PulseAudio spreads a stream of fewer channels than its sink over all
of them.  `cinq/` holds the tree of the `audio-outputs` branch of
yzhou216/cinq at the commit that made its JACK host list no input device,
before ASIO.

## What runs

`.github/workflows/jack.yml` builds cinq with `nix build ./cinq` and runs
`jack.sh` with jackd, jack_lsp, and jack_capture from Nixpkgs, and with
LD_LIBRARY_PATH naming Nixpkgs' libjack, which cinq loads at run time.

- `jack.sh` starts jackd with its dummy backend of eight playback ports and
  plays two shows on `jack:output`: `partial.cinq.toml`, whose outputs need
  three channels, and `full.cinq.toml`, whose outputs fill eight, House on
  1-2, Sub on 3, Monitors on 5-6, and Lobby on 7-8, at 0, 0, -6, and -12 dB.
- For each, `jack_lsp -c` tells where the ports of cinq connect, and
  `jack_capture --port 'system:playback*'` records what reaches the eight
  playback ports.

## Results

Run 43464407715 on `ubuntu-latest` passed.  Run 43462392972 before it gave
jack_capture options that it does not take, and listed `jack:input` among
the devices, which cinq now leaves out.

- `cinq --audio-devices` listed `alsa:null` and `jack:output`.
- For the partial show, cinq opened three ports, `out_0` to `out_2`,
  connected to `system:playback_1` to `system:playback_3`; for the full
  show, eight, each to the playback port of its number.
- Levels in dBFS RMS, with `-` for silence:

| Show    | 1     | 2     | 3     | 4 | 5     | 6     | 7     | 8     |
| ------- | ----- | ----- | ----- | - | ----- | ----- | ----- | ----- |
| partial | -16.5 | -16.5 | -16.5 | - | -     | -     | -     | -     |
| full    | -16.5 | -16.5 | -16.5 | - | -22.5 | -22.5 | -28.5 | -28.5 |
