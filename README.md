<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: a native build and run on a RISE RISC-V runner

Tested whether cinq, from a snapshot of its master at 90cd7ce in `cinq/`,
builds natively on `ubuntu-24.04-riscv` and runs there.  The runner was a
`scaleway-em-rv1` board with Ubuntu 24.04.5 on Linux 5.10.113-scw1, 4
`rv64imafdcsu` CPUs, 15 GiB of memory, and the rustc 1.99.0 of its image.

- `.github/workflows/riscv.yml` builds cinq with cargo, first as it is and
  then with openh264-sys2 taught riscv64, runs its tests, and plays
  `riscv.cinq.toml` by `play.sh` twice: with iced's default renderer, and
  with `ICED_BACKEND=tiny-skia`.
- `media.sh` makes the files of the show with ffmpeg: a chord that loops,
  and a video of ten seconds at 960x540 and 30 pictures a second, whose
  picture shows its own time to the hundredth of a second and whose sound is
  a 440 Hz tone.
- `play.sh` runs cinq in headless sway, with a PulseAudio null sink as its
  sound device.  GO starts the chord, and GO fades it out over two seconds,
  after which the video follows.  The script records the sink, takes a
  screenshot at each step, and notes the processor time of cinq and of
  each of its threads.  It ran on GNU/Linux on x86_64 before it ran here.

## Outcome

- Run 42300722925 (3982c61): the `nix` job passed, and the `cargo` job
  failed at once, since apt could not install `libasound2-dev`.
- Run 42301438697 (f136eaa): passed, with every crate built from scratch.
- Run 42313563402 (569a5f5): passed, from the cache of the run before, and
  played the show with both renderers.

### Build

cinq as it is does not build on riscv64.  openh264-sys2 0.9.9, the latest
release, panics in its build script with `Unknown target arch: riscv64`
(build.rs:110), as its master does: it names each architecture it knows
and panics on any other.  The name only picks the assembly of x86 and Arm,
so named, riscv64 builds the C++ of OpenH264 alone, as every other
architecture does.  With `--keep-going` the other 354 crates compiled, iced,
wgpu, and kira among them, in 3106 seconds.

With two lines added to that build script, which name riscv64, cinq built
in another 454 seconds: an ELF for RISC-V with the double-float ABI, which
links only libstdc++, libasound, libgcc_s, libm, and libc, and whose
`--version` gives `cinq 0.1.0`.  All 223 tests passed in 2.48 seconds, after
1625 seconds of building the tests.  From the cache, the three steps took
24, 439, and 250 seconds.

The image holds `liboss4-salsa-asound2`, the ALSA library of OSS 4, which
conflicts with ALSA's own library that `libasound2-dev` needs.  Nothing
installed depends on it but through the virtual `libasound2`, so apt
replaced it once told to remove it, by `liboss4-salsa-asound2-` in the same
`apt-get install`.

### Nix

`nix build ./cinq --dry-run` took 89 seconds to evaluate on the runner and
listed 1331 derivations to build and 824 paths to fetch (2.3 GiB), since
cache.nixos.org holds next to nothing for riscv64-linux.  They begin with
the bootstrap of the standard environment and include GCC 16.2.0, glibc
2.44, Python 3.14.8, LLVM 21.1.8, and rustc 1.99.0.  The build itself was
not tried.

### Running

Both plays opened the main window and the stage, took the keys, and played
the show as written.  The chord began within a tenth of a second of the
first GO, the fade with the second, and the 440 Hz tone of the video two
seconds later, for its ten seconds.  Sound went through ALSA's PulseAudio
plugin, and the video was decoded by openh264.  Two gaps of silence broke
the tone in run 42313563402, of 11 ms in the default play and of 41 ms in
the tiny-skia one, and none in run 42301438697; whether cinq fell behind
or the PulseAudio of the test, which ran without real-time priority, was
not found.

| Run 42313563402                        | Default renderer | tiny-skia |
| -------------------------------------- | ---------------- | --------- |
| Seconds until the windows open         | 7.42             | 0.56      |
| Cores busy while the chord plays       | 2.07             | 0.35      |
| Cores busy during the fade and video   | 2.08             | 1.45      |
| Seconds of picture a second, shots 2-5 | 0.80             | 1.00      |

- Without `ICED_BACKEND`, wgpu drew through Mesa's EGL on llvmpipe, whose
  threads `llvmpipe-0` to `llvmpipe-3` took about 1.5 cores between them
  while the video played.  llvmpipe was in the image already, and EGL came
  with the packages that play the show.  The picture of the video fell
  behind its sound, about 3 seconds behind by the end of the cue, so that it
  never showed the last seconds of the video, as in run 42301438697.
- With `ICED_BACKEND=tiny-skia`, the main thread of cinq took 98% of a core
  while the video played, and the picture kept to its time: from the second
  screenshot of the video to the fifth, it went on 5.77 seconds in 5.78.
  cinq's main window was the same as on x86_64 to the pixel.
- In the tiny-skia play, no thread of cinq but its main one took more than
  4% of a core while the video played, so decoding kept up.  Each run's
  channel of pictures holds two (`AHEAD` in `src/back/video/decoder.rs`),
  and each update takes those that have come due on iced's main thread,
  between drawings.  That slow drawing, as through llvmpipe, lets fewer
  pictures through than the video has is a reading of the code, not a
  measurement.

## Notes

- The jobs of this account ran one at a time: the `nix` job of the first
  run waited until its `cargo` job had ended, and the job of the second
  run until that `nix` job had.  A runner took about two minutes to start a
  job.
- On x86_64, the first sound that `parecord` recorded came about 1.7
  seconds after it started, so `play.sh` times its steps from the first
  sound written.
- The sway of Nixpkgs, used to try `play.sh` on x86_64, is a wrapper that
  forks sway, which goes on running when the wrapper is killed, so the
  script ends sway with `swaymsg exit`.
