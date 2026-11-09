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

## Final run (42705504855)

The same clips and runners, with `threaded` rebuilt from newer sources:
cinq 4ae3c09 (on 798c620; adds a cap on the reorder depth and corrected
docs) against rust_h264 at 6436335 plus three commits (a0acf09): 3cb90a4
(per-picture metadata, with a panic fix), e53d0f1 (keep ready frames when an
error is reported), and a0acf09 (block on worker results instead of a 20 ms
poll).  `base` is unchanged.  The earlier run, 42701179761, is the first
set of tables above (cinq 0ecefe8, rust_h264 2401e00).  All jobs were green.

### 1080p60 (600 pictures)

| runner | base shown | threaded shown | base CPU | threaded CPU | base first | threaded first |
| --- | --- | --- | --- | --- | --- | --- |
| ubuntu-latest | 389 / 386 | 600 / 600 | 12.77 / 12.59 | 23.03 / 22.65 | 0.002 / 0.002 | 0.002 / 0.002 |
| ubuntu-24.04-arm | 512 / 526 | 600 / 600 | 10.79 / 10.77 | 11.59 / 11.55 | 0.002 / 0.002 | 0.002 / 0.002 |
| macos-latest | 251 / 396 | 557 / 556 | 10.43 / 10.02 | 11.77 / 11.36 | 0.019 / 0.018 | 0.018 / 0.020 |
| windows-latest | 309 / 310 | 591 / 597 | 10.25 / 10.03 | 19.17 / 19.84 | 0.003 / 0.003 | 0.003 / 0.003 |
| windows-11-arm | 468 / 454 | 600 / 600 | 9.62 / 9.62 | 6.78 / 6.16 | 0.003 / 0.003 | 0.003 / 0.003 |

### 2160p30 (300 pictures)

| runner | base shown | threaded shown | base CPU | threaded CPU | base first | threaded first |
| --- | --- | --- | --- | --- | --- | --- |
| ubuntu-latest | 135 / 130 | 275 / 268 | 14.23 / 13.64 | 32.88 / 32.02 | 0.002 / 0.002 | 0.002 / 0.002 |
| ubuntu-24.04-arm | 190 / 191 | 300 / 300 | 10.76 / 10.82 | 15.83 / 15.76 | 0.002 / 0.002 | 0.002 / 0.002 |
| macos-latest | 143 / 126 | 298 / 291 | 10.74 / 10.74 | 16.31 / 17.12 | 0.015 / 0.018 | 0.034 / 0.010 |
| windows-latest | 103 / 107 | 197 / 199 | 11.64 / 11.30 | 23.30 / 23.70 | 0.003 / 0.003 | 0.002 / 0.003 |
| windows-11-arm | 187 / 185 | 300 / 300 | 9.97 / 9.69 | 10.98 / 11.19 | 0.003 / 0.003 | 0.003 / 0.003 |

Reading: every picture still shows on aarch64 Linux and aarch64 Windows, and
on amd64 Linux at 1080p60.  amd64 Linux now drops about 8 percent of
2160p30, which the first run showed complete, and amd64 Windows at 1080p60
shows about 99 percent.  The windows-11-arm threaded CPU at 1080p60 is below
its base, which was not investigated.  The runners were also slower for the
base build than in the first run (amd64 Linux base 389 against 468), so
compare within a run.

### Tests

Both suites passed on every runner.

| runner | rust_h264 (`cargo test --release`, patched) | cinq (threaded tree) |
| --- | --- | --- |
| ubuntu-latest | 262 + 6 passed, 2 ignored | 264 passed |
| ubuntu-24.04-arm | 255 + 6 passed, 2 ignored | 264 passed |
| macos-latest | 255 + 6 passed, 2 ignored | 264 passed |
| windows-latest | 262 + 6 passed, 2 ignored | 262 passed |
| windows-11-arm | 255 + 6 passed, 2 ignored | 262 passed |

## Runs

- 42695417284: round one, base only (smoke run, green).
- 42697382924: threaded build added, green.
- 42697404542: macos-latest failed in the patch step (see pitfalls), the
  others green.
- 42701179761: all jobs green; the first set of tables comes from its artifacts.
- 42705504855: final, cinq 4ae3c09 and rust_h264 a0acf09, all jobs green; the last set of tables.  (42705489833, started by a push that lacked the rust_h264 patches, was cancelled.)

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
