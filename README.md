<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq demo and tutorial built by the flake

Tests of [cinq](https://github.com/yzhou216/cinq)'s `nix build .#demo`,
which generates the demo shows' sounds, title card, and film with ffmpeg,
and `nix build .#doc`, which takes the tutorial's screenshots of cinq in a
headless sway session inside the Nix sandbox and typesets the PDF.  The
commits under test, on cinq's 6f5bc2e, were local and unpushed, so the
branch holds them as `cinq.tar.gz`, a `git archive` snapshot that the
workflow unpacks into a repository of its own, committed at the snapshot's
date, so that the flake is a Git one with the same `lastModified`.

## What ran

`test.yml` runs on `ubuntu-latest` and `ubuntu-24.04-arm`:

- `nix build` of `.#cinq`, `.#demo`, and `.#doc` from the snapshot.
- `run.sh` copies the demo out of the store, as the tutorial has readers
  do, and runs its three shows in cinq with the vi keys in a headless sway
  session with two 4:3 outputs: the storm through both acts and its
  effects, the chains with one GO and then the rain that a stop cue ends,
  and the lists from both lists.  It takes screenshots along the way and
  records each show from the monitor of a PulseAudio null sink, then fails
  if a recording is silent.
- `check-pdf.sh` fails on a reference that LaTeX left as `??`, or on
  screenshots other than ten at 2560x1920, and lays out the screenshots
  and the pages in contact sheets.
- The demo's files are hashed and their durations probed.

On `macos-latest` it builds `.#demo`, the only one of the two that builds
on Darwin, and hashes and probes its files.

## Results

Run 42957227622 passed on `ubuntu-24.04-arm` and `macos-latest`, and
failed on `ubuntu-latest` when `run.sh` started its session: `swaymsg
create_output` gave up after three seconds with "Unable to receive IPC
response", since the script took sway's socket appearing for sway being
ready.  `run.sh` and cinq's `doc/tutorial/shots.sh` now wait until sway
answers, and until cinq's window can be focused.

Run 42959632951 passed everywhere, but the film that `ubuntu-latest` built
differed from the one built locally on x86_64-linux: x264 takes its number
of threads from the processor's cores, and its output depends on it.
cinq's `demo/build.sh` now passes `-threads 4`.

Run 42962268975, of cinq's commits 89d9d52, c931f88, ddeeabb, and
94cdb99, passed on all three runners in about 25 minutes:

- On both GNU/Linux runners, the three shows played: each recording peaks
  between -6 and -1.6 dB, with a mean near -25 dB, and every screenshot
  shows its step, with no warnings in the sidebar.  The PDF has 26 pages,
  ten screenshots at 2560x1920, and no `??`.
- The demo that `ubuntu-latest` built is the same, byte for byte, as the
  one built locally.  Between the architectures, `bell.mp3`,
  `thunder.wav`, and `film.mp4` differ in bytes, from floating-point
  rounding, while every file lasts the same on all three runners: 12, 8,
  5, 4, 0.7, and 20 seconds.

## Learned

- Sway's Wayland and IPC sockets appear before it answers on them, and on
  `ubuntu-latest`, outside the Nix sandbox, starting took longer than the
  three seconds that `swaymsg` waits.  Polling `swaymsg -t get_version`
  until it answers covers it.
- x264's output depends on its thread count, which it takes from the
  cores, so a fixed `-threads` keeps a film the same across machines of
  one architecture.  The Ogg muxer picks a random serial number unless
  `-fflags +bitexact`, and the `gradients` source needs a `seed`.
- `gh run watch` polls every few seconds, and with other sessions of the
  same user it used up the API's 5000 requests an hour; waiting on
  `gh api rate_limit`, which does not count, and polling every two
  minutes did not.
