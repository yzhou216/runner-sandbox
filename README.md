<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: where more threads would help

Looked for the parts of cinq, at its master ec61ead, that use the processor
most, and timed whether more threads, with rayon or otherwise, or tokio,
would make them faster, on every kind of runner: `ubuntu-latest` and
`windows-latest` (AMD EPYC, 4 cores), `ubuntu-24.04-arm` (Neoverse-N2, 4
cores), `windows-11-arm` (Cobalt 100, 4 cores), `macos-latest` (Apple M1
virtual machine, 3 cores), and RISE's `ubuntu-24.04-riscv` (4 cores, no
vector extension).  Everything that plays video goes through three stages:
rust_h264 decodes each sample on the thread of a pass, cinq converts the
picture from YUV to RGBA there, with the strips of a high-definition
picture on scoped threads spawned for it, and iced draws it, which
iced's tiny-skia renderer does on the thread of the interface.

- `bench/` times each stage the way cinq does it and the ways it could:
  `convert` the conversion, serial, on scoped threads, and on rayon's pool;
  `kernels` loops of the conversion that vectorize, and the conversion of
  the snapshot in `cinq/` against that of ec61ead, byte for byte; `draw`
  what tiny-skia does on the thread of the interface for each new picture;
  `pass` whole passes through clips that `bench/clips.sh` makes with x264,
  as cinq decodes them, with the conversion pipelined on a thread of its
  own, and with rust_h264's `ThreadedDecoder`.  `results/<run>/` holds the
  tables of each run.
- `cinq/` is cinq's master at ec61ead with its pictures converted in words
  of two pixels, which vectorize.  `realtime/run.sh` plays each clip in real
  time through cinq's video output, by the test in `realtime/realtime.rs`,
  with ec61ead, with the snapshot, and with the snapshot decoding through
  `ThreadedDecoder`, as `realtime/decoder-threaded.rs` prototypes it.

## Outcome

Runs 42422493908, 42423884592, and 42425289237 (rounds 1 to 3) timed the
stages with `bench/`; their tables are in `results/`.  Run 42426043066
timed the final form of the snapshot's conversion and ran its tests on
GNU/Linux and Darwin, and run 42427358379 ran the tests beside ec61ead's
and the plays in real time on all five GitHub runners.  Runs 42425692064
and the cinq jobs of 42425289237 failed on a test of mine that did not
compile, and then on one that took the planes of an odd width in a layout
that rust_h264 does not use.

### Decoding is the limit, and threads lift it

rust_h264 decodes on the one thread of a pass.  At the bitrates of video
made for a show, it falls short of real time for 1080p at 60 pictures a
second and for 2160p on every runner, and rust_h264's `ThreadedDecoder`,
which decodes several pictures at once, doubles to triples it (round 3,
pictures a second, best of two passes):

| Runner | 1080p30, 20 Mbit/s | 1080p60, 20 Mbit/s | 2160p30, 20 Mbit/s | 2160p30, 52 Mbit/s |
| --- | --- | --- | --- | --- |
| ubuntu-latest | 33.5 / 66.9 | 48.4 / 93.6 | 17.7 / 36.6 | 11.0 / 21.3 |
| ubuntu-24.04-arm | 42.6 / 104.7 | 59.8 / 151.4 | 21.2 / 57.4 | 14.0 / 32.6 |
| macos-latest | 48.6 / 71.5 | 59.6 / 122.6 | 22.1 / 44.2 | 15.7 / 30.4 |
| windows-latest | 35.8 / 72.3 | 48.0 / 100.1 | 17.0 / 36.9 | 11.6 / 23.5 |
| windows-11-arm | 38.8 / 95.8 | 53.3 / 134.9 | 18.5 / 51.3 | 12.4 / 29.4 |
| ubuntu-24.04-riscv | 4.0 / 8.9 | | | |

Each cell gives cinq at ec61ead, then the threaded decoder with the
snapshot's conversion.  Clips under heavier grain, at 55 to 425 Mbit/s,
went three times as fast threaded as well (round 1).

`ThreadedDecoder` hands out pictures in the order they show, reordering up
to sixteen deep, with no way to say which sample each came from, while
cinq times each picture by its own sample.  The prototype in
`realtime/decoder-threaded.rs` gives each picture the next of the times of
the samples in order, reorders four deep, and so holds only while every
sample decodes to one picture, as in these clips; a damaged sample would
shift the times of those after it.

### Real-time plays through cinq's video output

Pictures shown of those due, in two rounds, by the build of ec61ead, of
the snapshot, and of the threaded prototype (run 42427358379):

| Runner | 1080p60, 20 Mbit/s, of 240 | 2160p30, 20 Mbit/s, of 120 | 2160p30, 52 Mbit/s, of 90 |
| --- | --- | --- | --- |
| ubuntu-latest | 188, 190 / 194, 196 / 240, 240 | 70, 70 / 74, 74 / 120, 120 | 42, 42 / 42, 42 / 69, 69 |
| ubuntu-24.04-arm | 219, 220 / 217, 218 / 240, 240 | 77, 77 / 80, 80 / 120, 120 | 47, 48 / 48, 48 / 90, 90 |
| macos-latest | 221, 204 / 222, 193 / 235, 191 | 85, 112 / 79, 120 / 120, 120 | 42, 44 / 38, 44 / 87, 90 |
| windows-latest | 174, 175 / 178, 182 / 240, 240 | 59, 63 / 62, 66 / 120, 120 | 39, 38 / 39, 37 / 65, 65 |
| windows-11-arm | 185, 188 / 187, 194 / 240, 240 | 74, 75 / 80, 80 / 120, 120 | 46, 46 / 47, 46 / 90, 90 |

Every build showed all 180 pictures of the 1080p clip at 30 pictures a
second on every runner, and of the one at 20 Mbit/s, but for
`windows-latest`, where ec61ead showed 167 and 170, the snapshot 173 and
173, and the threaded prototype 180 and 180.  The threaded prototype is
the only build that plays 2160p in real time anywhere, and 1080p at 60
pictures a second on all but the Darwin virtual machine of three cores.

### Conversion did not vectorize

cinq wrote each pixel as four separate bytes, which LLVM did not turn into
vector instructions, so that turning a 1080p picture into RGBA took 19 ms
on one EPYC core, against 4.8 ms on a Neoverse-N2; built for x86-64-v3
with AVX2, it took 9% less.  Packing each pixel into a word and storing
each pair of pixels at once gives the same bytes, on every runner, at odd
sizes as well, and vectorizes.  `lanes`, sixteen pixels at once, was 10%
faster still where there are vector units, but 55% slower on riscv64,
which has none here, while `pairs` was 12% faster there.  cinq's own
conversion of a high-definition picture, strips on threads included, at
ec61ead and in the snapshot (run 42426043066, milliseconds, wall and
processor):

| Runner | 1080p | 2160p |
| --- | --- | --- |
| ubuntu-latest | 6.17, 23.91 / 2.92, 10.95 | 24.93, 95.43 / 11.97, 43.86 |
| ubuntu-24.04-arm | 1.45, 5.17 / 0.84, 2.71 | 5.37, 19.61 / 2.94, 9.91 |
| macos-latest | 1.66, 4.74 / 0.50, 1.32 | 6.79, 19.06 / 2.22, 5.46 |
| windows-latest | 4.99, 18.47 / 2.32, 8.20 | 19.33, 74.31 / 9.08, 33.23 |
| windows-11-arm | 2.35, 7.41 / 1.74, 4.34 | 7.34, 23.80 / 4.93, 14.52 |
| ubuntu-24.04-riscv | 22.85, 38.96 / 20.35, 34.24 (run 42425289237) | 75.33, 171.50 / 64.83, 149.29 |

In the real-time plays, the snapshot took 6% to 9% less processor time than
ec61ead on `ubuntu-latest`, 8% to 15% less on `ubuntu-24.04-arm`, and 4% to
31% less on `macos-latest`, for the same pictures; decoding takes most of
the rest.  Bash's `time` gives no processor time for native programs on
Windows.

### What does not need more threads

- Spreading the conversion over rayon's pool rather than scoped threads
  spawned for each picture gained 0% to 10%, within the noise of the
  runners, and the scoped threads never fell behind rayon's in latency
  over several runs (rounds 1 and 2).  rayon's pool would also share its
  threads with `load_files`, which can block on a slow drive.
- Pipelining the conversion on a thread of its own while the next picture
  decodes gained 2% to 20% before the conversion vectorized, and 0% to 10%
  after (round 3).
- tokio: what cinq does off the thread of the interface is decoding, which
  is work for the processor, and a few blocking reads and writes of files,
  each on a thread of its own, none of which waits on many sources at once.

### Drawing with tiny-skia

With no GPU, as on every runner here, iced's tiny-skia renderer turns each
new picture into premultiplied BGRA and draws it scaled, on the thread of
the interface: 15 to 29 ms for a 1080p picture on a 1080p screen, 31 to 37
ms for a 720p one scaled to 1080p, 38 to 72 ms for a 2160p one, and 110 to
560 ms on riscv64 (round 1).  Scaling each picture to the size it shows at
on worker threads would cut that to 7 to 11 ms on a 720p screen and 15 to
29 ms on a 1080p one when it scales, but nothing below the cost of a
picture as large as the screen; with a GPU, cinq's own shader draws each picture.

### Tests

The snapshot's 248 tests, its new test of the conversion included, pass on
GNU/Linux and Darwin, as ec61ead's do.  On Windows, two tests fail with
the snapshot and at ec61ead alike, since they take POSIX paths such as
`/show` for absolute: `relative_sound_files_go_from_wherever_the_file_is`
and
`file_is_in_the_directory_named_on_the_command_line_or_else_that_of_the_platform`.
clippy finds nothing in cinq on any runner.  The step that compares the
tests reported failure on four runners in run 42427358379 for faults of
the step itself, fixed since and checked against that run's logs: its list of failing tests took
the summary line, whose time differs from build to build, and a `grep`
that found nothing ended the step.

## Notes

- Copies of one package in several directories that share a target
  directory share one test program as well: cargo names it by the path of
  the package from its workspace root, which is the same for each copy,
  and once it is built, a copy whose files have not changed since takes
  it as its own.  `realtime/run.sh` copies each program out as it is
  built; run 42426043066 had run the last one built for every copy.
- `CINQ_SHARE_DIR=` with an empty value builds cinq without scdoc.
- `bench/clips.sh` makes clips at realistic bitrates from FFmpeg's test
  pattern with `noise=alls=8:allf=t` and `-b:v`; under `alls=24` the
  bitrate went to 55 Mbit/s at 720p and 425 Mbit/s at 2160p.
