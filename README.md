<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq/2026-11-09-threaded-decoder

Does cinq's video output play in real time when it decodes on several
threads?  Two builds of cinq play the same clips in real time on five hosted
runners:

- `base`: cinq master at 798c620, which decodes with rust_h264's serial
  `Decoder`.
- `threaded`: cinq 0ecefe8 (on top of 798c620), which decodes with
  `ThreadedDecoder`, one thread for each core up to eight, built against a
  rust_h264 whose `ThreadedDecoder` carries per-picture metadata (2401e00 on
  top of v0.6.1, 6436335).  The metadata gives each picture the index of its
  sample, so the picture's time needs no ordering by hand.

Neither change is pushed upstream.  The workflow clones both projects at
their base commits and applies the `git format-patch` output in
`patches/`; the last cinq patch only points `rust_h264` at `../rust_h264`
with a matching `Cargo.lock`, because `run.sh` builds with `--locked`.

## What runs

`.github/workflows/threaded.yml` builds two clips with `bench/clips.sh`: 10 s
of 1080p at 60 pictures a second (600 pictures) and 10 s of 2160p at 30 (300
pictures), both at 20 Mbit/s H.264 with B-frames.  On each runner,
`realtime/run.sh` builds both trees, adds `realtime/realtime.rs` as a test
of each, and plays each clip through each tree twice (rounds 1 and 2) the
way the interface does: the cue is made ready, one second passes, GO is
given, and the video output is polled every 2 ms for pictures.  Each play
prints the pictures shown, the processor time of the process, and the seconds
from the GO to the first picture.  The workflow also runs `cargo test
--release` in `rust_h264/` and in `cinq/` (the threaded tree).

Runners: `ubuntu-latest` (amd64), `ubuntu-24.04-arm`, `macos-latest`
(aarch64), `windows-latest` (amd64), and `windows-11-arm`.

## Results

Each cell is round 1 / round 2.  Pictures shown are out of the clip's count;
CPU is user plus system seconds of the test process for the play (clip
length is 10 s); first is seconds from the GO to the first picture.

### 1080p60 (600 pictures)

| runner | base shown | threaded shown | base CPU | threaded CPU | base first | threaded first |
| --- | --- | --- | --- | --- | --- | --- |
| ubuntu-latest | 468 / 460 | 600 / 600 | 13.32 / 13.19 | 20.24 / 19.86 | 0.002 / 0.002 | 0.002 / 0.002 |
| ubuntu-24.04-arm | 519 / 525 | 600 / 600 | 11.00 / 10.95 | 12.32 / 12.29 | 0.002 / 0.002 | 0.002 / 0.002 |
| macos-latest | 539 / 538 | 559 / 552 | 9.17 / 9.11 | 9.69 / 9.31 | 0.017 / 0.018 | 0.018 / 0.018 |
| windows-latest | 322 / 326 | 600 / 599 | 14.19 / 14.19 | 31.50 / 31.84 | 0.003 / 0.003 | 0.003 / 0.002 |
| windows-11-arm | 440 / 429 | 600 / 600 | 11.61 / 11.73 | 13.86 / 14.17 | 0.003 / 0.003 | 0.003 / 0.003 |

### 2160p30 (300 pictures)

| runner | base shown | threaded shown | base CPU | threaded CPU | base first | threaded first |
| --- | --- | --- | --- | --- | --- | --- |
| ubuntu-latest | 171 / 173 | 300 / 300 | 15.33 / 15.36 | 30.42 / 29.95 | 0.002 / 0.002 | 0.002 / 0.002 |
| ubuntu-24.04-arm | 184 / 185 | 300 / 300 | 12.11 / 12.03 | 17.53 / 17.47 | 0.002 / 0.002 | 0.002 / 0.002 |
| macos-latest | 200 / 242 | 299 / 292 | 11.20 / 11.38 | 12.24 / 12.66 | 0.019 / 0.018 | 0.018 / 0.018 |
| windows-latest | 108 / 103 | 204 / 201 | 15.52 / 15.29 | 33.66 / 33.72 | 0.003 / 0.003 | 0.003 / 0.003 |
| windows-11-arm | 171 / 171 | 300 / 300 | 12.54 / 12.42 | 19.99 / 20.52 | 0.003 / 0.003 | 0.003 / 0.003 |

Reading: the threaded decoder shows every picture on both Linux runners and
on aarch64 Windows, and nearly every one on amd64 Windows at 1080p60.  It
drops 30 percent of 2160p30 on amd64 Windows and 1080p60 stays at about 93
percent on Darwin, where the base also nears that.  It costs more
processor time, up to twice as much on amd64 hosted runners, which have two
cores.  The first picture is ready within 20 ms of the GO everywhere, with
or without threads, because the cue's first picture is made ready ahead.
Darwin takes 17 ms with both builds, so the decoder is not the cause (the
cause was not investigated).

## Tests

Both suites passed on every runner.

| runner | rust_h264 (`cargo test --release`, patched) | cinq (threaded tree) |
| --- | --- | --- |
| ubuntu-latest | 260 + 6 passed, 2 ignored | 264 passed |
| ubuntu-24.04-arm | 253 + 6 passed, 2 ignored | 264 passed |
| macos-latest | 253 + 6 passed, 2 ignored | 264 passed |
| windows-latest | 260 + 6 passed, 2 ignored | 262 passed |
| windows-11-arm | 253 + 6 passed, 2 ignored | 262 passed |

## Runs

- 42695417284: round one, base only (smoke run, green).
- 42697382924: threaded build added, green.
- 42697404542: macos-latest failed in the patch step (see pitfalls), the
  others green.
- 42701179761: final, all jobs green; the tables come from its artifacts.

## Pitfalls

- A 57 MB `testdata/` makes committing rust_h264 impractical; patches
  applied to a fresh clone at a pinned commit keep the branch small.
  `hashFiles` evaluates when its step runs, so the clone step placed before
  the guarded steps is seen by them.
- `rm -rf` of a freshly cloned `.git` failed on macOS with "Directory not
  empty" in one of two runs; the step now keeps the directories.
- Test steps run in `cinq/` and `rust_h264/`, which `run.sh` copies, so they
  use `CARGO_TARGET_DIR` outside the trees; otherwise `target/` would be
  copied too.
- `git am` needs an identity, which `-c user.name -c user.email` supplies.
- `--locked` needs a `Cargo.lock` without the registry source for
  `rust_h264`, as `cargo update -p rust_h264 --offline` writes it.
