<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: rust_h264 in place of openh264

Tested cinq with its video cues decoded by rust_h264 0.6.1, a decoder of
H.264 written in Rust, in place of Cisco's openh264, which it built from
C++.  `cinq/` holds the change uncommitted on cinq's master at 01c6d31: each
sample decodes to its own picture, which goes out in the order of the times
of the samples, cinq turns every picture into RGBA itself, and the tests
make their clips with H.264 that `stream.rs` writes bit by bit.

- `compare/` decodes clips of x264 with rust_h264, as cinq does, and with
  openh264, as cinq did, checks each picture against the MD5 that FFmpeg's
  decoder gives it, and times both, along with rust_h264's decoder on four
  threads, which cinq does not use.  `clips.sh` makes the clips: Baseline at
  360p, Main at 540p, High at 720p with noise, at 1080p, at 1080p with six
  references and weighted prediction, at 2160p, and interlaced at 1080i.
- `play.sh` on GNU/Linux and riscv64, `play.ps1` on Windows, and the steps
  of the `darwin` job play `video.cinq.toml`: GO starts a chord, and GO
  fades it out, after which a video of ten seconds follows, whose picture
  shows its own time to the hundredth of a second.  Screenshots of the
  stage, which the video makes fullscreen, tell how far its picture has
  come at each step.

## Outcome

Run 42337020892 (329a578):

- Nix built cinq with its 235 tests passing on GNU/Linux (x86_64) and on
  Darwin (Apple silicon, where rust_h264 decodes with NEON), and
  cross-built it for Windows on x86_64 and on Arm.  The GNU/Linux program
  needs libasound, libgcc_s, libm, and libc, and no longer libstdc++.
- The picture of the video kept to its time on GNU/Linux, Darwin, and
  Windows on x86_64, as the screenshots read: on Darwin, for example, the
  picture showed 0.93, 2.56, 4.26, 5.96, 7.66, and 9.43 seconds at steps
  about 1.7 seconds apart.  On Windows on Arm it fell behind, which run
  42338925460 tells the cause of.  The sound of the show on GNU/Linux went
  as written: the chord from the first GO, its fade from the second, and
  the 440 Hz tone of the video for ten seconds after the fade.
- On the RISE RISC-V runner, cargo built cinq as it is in 47 minutes 35
  seconds, with no patch, and its 235 tests passed.  The show played with
  its sound as written, but the picture of the video fell behind, which run
  42356188001 tells the cause of.
- rust_h264 decoded every picture of every clip as FFmpeg does, on all six
  kinds of runner, on four threads as well, NEON on Arm and the x86 kernels
  included.  openh264 decoded the Baseline clip as FFmpeg does, 114 of 120
  pictures of the Main one, 4 to 34 of the pictures of each High one, and
  none of the interlaced one; on riscv64 it built from the branch riscv64
  of yzhou216/openh264-rs.

| Pictures a second, one thread | rust_h264 1080p | openh264 1080p | rust_h264 2160p | openh264 2160p |
| ----------------------------- | --------------- | -------------- | --------------- | -------------- |
| ubuntu-latest                 | 74              | 80             | 18              | 26             |
| ubuntu-24.04-arm              | 92              | 120            | 24              | 36             |
| macos-latest                  | 115             | 126            | 31              | 39             |
| windows-latest                | 60              | 81             | 16              | 26             |
| windows-11-arm                | 84              | 104            | 22              | 32             |
| ubuntu-24.04-riscv            | 9.0             | 16             | 2.2             | 4.9            |

On four threads, rust_h264 decoded 2160p at 33 to 58 pictures a second, and
at 5.1 on riscv64, whose board has no vector extension.

Run 42338925460 (1bc9546) played the show on windows-11-arm with cinq
cross-built from `cinq/` and from cinq's master at 01c6d31, which decodes
with openh264, each with iced's default renderer, which draws through WARP
there, and with `ICED_BACKEND=tiny-skia`, twice each.  With WARP, the
picture went on at 0.54 to 0.59 seconds a second with either decoder.  With
tiny-skia, it kept to its time with rust_h264, 8.76 seconds in 8.77 and
8.77 in 8.76, and went on at 0.83 to 0.85 with openh264, which
openh264-sys2 builds for Windows without its assembly for Arm.  So the
picture falls behind there for drawing through WARP, not for rust_h264.

Run 42356188001 (8e26f96) played the show on the RISE RISC-V runner with
cinq built from `cinq/` and from cinq's master at 01c6d31, whose lock file
`cargo update -p openh264-sys2 --precise 0.9.8` points at the branch
riscv64 of yzhou216/openh264-rs; run 42353108467 had failed without that,
as the 0.9.8 of the branch replaced nothing in a lock file that asked for
0.9.9.  With `ICED_BACKEND=tiny-skia`, the picture went on at 0.38 seconds
a second with rust_h264 and at 0.37 with openh264, while the main thread
of cinq, which draws, took 95% and 93% of a core and the thread that
decodes 34% and 25%: the stage, which goes fullscreen at 1280x720 with each
GO of a video since cinq 0.1.1, is more than tiny-skia draws thirty times a
second there.  Through llvmpipe, whose four threads took half a core each,
it went on at 0.59 with rust_h264, and at 0.73 in run 42337020892, and at
0.77 with openh264.

## Notes

- `nix build github:yzhou216/cinq/<commit>#aarch64-windows-cross` builds
  an earlier cinq without a copy of its tree here.
- A `[patch]` that `--config` gives cargo is used only when the lock file
  asks for its version, so for a lock file that holds a later version, run
  `cargo update` with the same `--config` and `-p <crate> --precise <its
  version>` first.  Without a lock file, as for `compare/`, it is used at
  once.
- `actions/download-artifact` takes `run-id` and `github-token`, with the
  `actions: read` permission, to fetch what an earlier run made.
- The prompt to update WSL on windows-11-arm covered the stage in two
  screenshots of the first run, so `play.ps1` closes it before each one.
