<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: how outputs tell why they cannot play

This branch shows, on GNU/Linux, Darwin, and Windows, how cinq tells why
outputs cannot play: in a few words in the outputs view, in yellow where
cinq tries the device again by itself and in red where the output must
change, in one warning for the outputs of a device, in the names of the
Levels tab, and in the Device list of the inspector.  `cinq/` holds the
tree of commit 3b5aef6 of the `audio-outputs` branch of yzhou216/cinq.

## What runs

`.github/workflows/ui.yml` writes the show of `show.sh` on each system:
House L and House R on channels 1 and 2 of a device there, which play,
Over on channels past those of that device, and Stage L and Stage R on a
Dante Virtual Soundcard that is not there, with a cue of a tone that sends
to all five.

- `linux.sh` plays on jackd's dummy backend of 64 ports, with Over on
  65-66, in a headless sway.
- `darwin.sh` plays on BlackHole 16ch, with Over on 17-18.
- `windows.ps1` plays on VB-CABLE, with Over on channel 3, and opens a
  second show that names an endpoint of another machine, along with the
  name of VB-CABLE's.

Each takes screenshots of the main window, the Levels tab, the outputs
view, and the inspector, and records the device to measure House L and
House R.

## Results

Run 43599976711 passed on all three systems.

- House L and House R played at -30.1 and -36.1 dBFS everywhere, as they
  are sent.
- The outputs view read Past its 64, 16, or 2 channels in red for Over,
  and Not found, retrying in yellow for Stage L and Stage R, each on one
  line.
- The sidebar held two warnings: one for the Dante Virtual Soundcard and
  its two outputs, and one for Over, and the status bar counted two.
- The Levels tab named Over in red and the Stage outputs in yellow, beside
  3 cannot play now.
- The Device list of Stage L read Dante Virtual Soundcard, not found, in
  yellow.
- On Windows, the Device list of House L chose VB-CABLE both when the show
  named its endpoint and when it named an endpoint of another machine
  along with VB-CABLE's name, which commit 3b5aef6 made it do.
- Runs 43596203728 and 43598724393 came before.  The first wrote channels
  65-66, past the 64 that cinq then took, which commit b122c44 raised to
  256, and Git's bash dropped the braces of the endpoint ID until the
  script set MSYS to noglob, which showed that the Device list chose
  devices by ID alone.
