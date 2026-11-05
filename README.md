<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: video cues on GNU/Linux, Darwin, and Windows

Recorded cinq playing a video cue in its stage window and then fading it out
with a fade cue, with the main window and the stage side by side, on GNU/Linux
(headless sway on `ubuntu-latest`), Darwin (`macos-latest`), and Windows
(`windows-latest` and `windows-11-arm`).  The `build` job makes a 1080p and a
360p test pattern with ffmpeg and cross-builds `cinq.exe` and the test
programs for both Windows targets from `cinq/`, a snapshot of cinq's tree;
the other jobs record the screen while `video.cinq.toml` plays, with
`video.ps1` on Windows.  Changes in each recording were counted per window
with ffmpeg's scene filter on the region of the cue's clock and of the stage.

## Outcome

The last run, 42072969384 (e13020e), records the code that went into cinq:
both windows played and faded out on every platform.  On Windows runners,
which have no GPU and draw through WARP, x64 at 360p and arm64 at both sizes
updated both windows 8 to 15 times a second; x64 at 1080p, on a runner where
the stage managed about 3 pictures a second, updated the clock throughout,
apart from one gap of a second at the GO of the fade.

How it got there:

- Run 41976455045 (100424e): GNU/Linux and Darwin played the video and faded
  it out.  The stage stayed black on Windows, while the video tests passed
  there: iced_wgpu uploads images of 2 MiB or more in the background, so
  each picture of a 1080p video was replaced before it showed.  cinq then
  drew each cue from a texture of its own, which each picture fills.
- Run 42021230548 (8568571): every platform showed the pictures.  On Windows
  arm64 the main window froze while the stage played, and on x64 both froze
  for about five seconds.
- Run 42025757147 (34f5b17): with cinq's own software renderer
  (`ICED_BACKEND=tiny-skia`), x64 played 1080p with both windows live; with
  WARP, the main window froze even at 360p on x64.  This is winit#3648:
  winit's Windows loop draws one window per wake and iced redraws every
  window after each update, so a slow drawing lets one window starve the
  other.  A WSL update prompt on arm64 took the first GO.
- Run 42058451487 (0b2a9cc) failed to install Nix:
  `wimpysworld/nothing-but-nix@main` had moved TMPDIR under `/nix/tmp` that
  day (#56), which the installer of Nix 2.35.4 cannot use, so the workflow
  pins the commit before it.
- Run 42059144455 (2726527, cancelled once the Windows jobs had finished):
  ticks of the clock that wait for both windows to draw made both windows
  play at 360p, and at 1080p apart from pauses of about a second.
  `video.ps1` now closes the WSL prompt before each key.
- Run 42063378014 (a1634b8): ticks timed from the last tick, which also wait
  for a drawing since the update before them.  GNU/Linux, Darwin, arm64, and
  x64 at 360p played smoothly; x64 at 1080p, on a runner where the stage
  managed 3 pictures a second, froze the main window for 4.6 seconds.
- Run 42068343028 (1ca2407, logging each drawing and tick to `err.txt`):
  on a slow arm64 runner, a stream that had ticked went on ticking before the
  update of its tick came, and ticks went before any window had drawn, so
  the stage drew once in eight seconds.
- Runs 42072872564 (430854b, logging) and 42072969384 (e13020e): a single
  tick for each update, which waits up to a second for the first drawing and
  then 50 ms for the other window.  Both windows drew 2 to 5 times a second
  on slow runners at 1080p, and every tick came with an update of its own.
