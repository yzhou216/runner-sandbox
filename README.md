<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq held output demo

Two short videos of cinq holding what is printed while the problems of its
`config.toml` show on the alternate screen of the terminal, and printing it on
the main screen once the problems clear.  The feature is commit da8cb2b of
the `held-output` branch of yzhou216/cinq, whose tree at that commit is under
`cinq/`.

## Runners and runs

| Video     | Runner | Run | Commit |
| --------- | ------ | --- | ------ |
| GNU/Linux | `ubuntu-latest`, Ubuntu 26.04, image 20261101.186.1 | 42895097311 | f20a9ca |
| Windows   | `windows-latest`, Windows Server 2025 build 26100, image windows-2025-vs2026 20261103.298.1 | 42897059337 | d17aacb |

Runs 42895097311 and 42897059337 are the ones whose artifacts hold the final
videos.  Earlier runs of this branch, which led there:

- 42889811098 (1a3d8c0), GNU/Linux and Windows: the GNU/Linux video was right
  at once.  On Windows the keys typed for the commands console never reached
  it, because the stage window that cinq opens took the focus and Windows
  only flashed the taskbar button of the console, so the video file never
  arrived and no warning came.  The recording was also cut short, since
  killing ffmpeg loses the last seconds of a Matroska file.
- 42892030710 (d3a0466), Windows only: focus fixed with a tap of Alt before
  `SetForegroundWindow` and a check of `GetForegroundWindow`, and cinq's two
  windows moved beyond the right edge of the desktop.  The warning came, but
  the recording was still cut off at 37 seconds.
- 42893774152 (ab8a9f7), Windows only: ffmpeg ends with a `q` on its standard
  input, so the file is complete.
- 42895097311 (f20a9ca), GNU/Linux and Windows: the System Properties window
  that the image leaves open was closed, which left a Performance Options
  window showing for the first seconds.
- 42897059337 (d17aacb), Windows only: the `SystemPropertiesPerformance`
  process ended before recording.

## GNU/Linux

`linux.sh` runs headless sway with two foot terminals, the top one for cinq
and the bottom one for commands, and cinq's window to the right.  The top
terminal runs through `script`, whose typescript (`typescript` in the
artifact) holds what the terminal received, in order.

| Caption from second | What shows |
| ------------------- | ---------- |
| 6.8  | `cinq` is started with `colour = "red"` in `config.toml`, a setting that cinq does not know, and the problem takes the alternate screen |
| 12.9 | `echo ... > /proc/$(pidof cinq)/fd/2` writes a line from another process to the standard error of cinq |
| 20.5 | `aplay -D missing /dev/zero 2> /proc/$(pidof cinq)/fd/2` writes the ALSA error of a C program the same way, and the problems screen stays clean (frames at 24 and 28 seconds) |
| 28.7 | `sed -i /colour/d config.toml` fixes the file: the terminal switches back and the three held lines are on the main screen, below `$ cinq` (frame at 36 seconds) |
| 37.2 | the problem comes back, and one more line is written while the screen shows (frame at 47 seconds) |
| 49.0 | cinq is closed with the problem left: `one more line` comes out first and then the problem (frame at 56 seconds) |

The typescript shows the same order: the lines come after `ESC[?1049l`, the
switch back to the main screen, and none comes between `ESC[?1049h` and it.

The sound system of the machine has a null device in `~/.asoundrc`, so that
cinq's own start prints nothing.  Lines that cinq prints before the problems
screen opens, such as the ALSA error of a missing plugin in `~/.asoundrc`
(cinq opens the sound output before the watch of `config.toml` starts), are
not held: they stay on the main screen as they always did.  So the lines
written meanwhile come from `/proc/PID/fd/2`, which is the descriptor that
the libraries inside cinq write to as well.

## Windows

`windows.ps1` opens two Windows Terminal windows, the top one running `cmd`
for cinq and the bottom one PowerShell for commands, in `C:\demo`, with the
text enlarged by three steps.  Windows only holds what cinq itself prints,
which is its warnings.  The show `held.cinq.toml` has a video cue whose
`clip.mp4` is not there yet, and cinq reads such files again every two
seconds.  A show named on the command line would be read at the start, which
may come before the first reading of `config.toml` draws the problems screen
and so says nothing about holding.  So the clip arrives while the problems
show.

`clip.sh` makes the clip with ffmpeg: six seconds of H.264 with two chapters,
which gives a `text` track for them, and then renames the `stbl` box of that
track to `skip`, so that mp4-atom cannot decode the track while the picture
track stays readable.  The `ffprobe` of the runner lists the picture track
and the chapters as a data stream that it cannot decode, and cinq warns
`missing box: stbl`.

| Caption from second | What shows |
| ------------------- | ---------- |
| 11.7 | `cinq.exe --config-dir conf held.cinq.toml` is typed, cinq starts, and the problem takes the alternate screen, which shows from about 18 seconds |
| 20.3 | `Move-Item incoming\clip.mp4 clip.mp4` is typed and run: the clip arrives, and cinq reads it within two seconds |
| 30.6 | the problems screen is clean while cinq has read the clip (frame at 33 seconds) |
| 34.6 | `Set-Content conf\config.toml ''` fixes the file |
| 45.4 | the main screen is back, with `cinq: C:\demo\clip.mp4: skipped a track, which cannot be read: missing box: stbl` below the command (frames at 41 and 47 seconds) |

The video is Matroska on the runner, with the times of the captions in
`captions.tsv`; `burn.sh` turned it into the MP4 with the captions in a band
under the picture.

## Files

- `linux.sh`, `windows.ps1`: what runs on the runners.
- `clip.sh`: the clip with a chapter track that cannot be read.
- `burn.sh`: captions into a band under the picture of a video; the GNU/Linux
  job runs it, and the Windows video was burned after the download.
- `.github/workflows/held.yml`: builds cinq with Nix (`./cinq` on GNU/Linux,
  `./cinq#x86_64-windows` cross-built for Windows), and runs both scripts.  A
  commit message with `[windows only]` leaves the GNU/Linux job out.
- `cinq/`: the snapshot of cinq at da8cb2b, `git archive`.

Artifacts: `linux` (the video, its captions, the typescript, and logs) and
`windows` (`demo.mkv`, `captions.tsv`, `ffmpeg.txt`, and the damaged
`clip.mp4`).
