<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: a recorded walkthrough of a show on every platform

Recorded cinq 0.1.0 (`cinq/`, a snapshot of fc1bdce, as published on
crates.io) running `usage.cinq.toml` by the steps of `steps.txt`, with the
main window and the stage side by side, on GNU/Linux (headless sway on
`ubuntu-latest`), Darwin (`macos-latest`), and Windows (`windows-latest` and
`windows-11-arm`, with `usage.ps1`).  The `build` job makes the sounds and
960x540 videos of the show with ffmpeg and cross-builds `cinq.exe` for both
Windows targets.

The walkthrough, in about 45 seconds: GO starts looping preshow music; GO
fades it out over 3 seconds, after which the opening titles follow; GO fades
the titles to black, after which a looping scene follows; the help page
opens and closes; GO fades the scene out; the second list, Act 2, comes up,
where GO starts a looping bell and GO a looping video; Escape, the panic,
fades everything out.

## Outcome

Run 42178525506 (6923a77): every job passed, and each recording shows the
whole walkthrough, with the standby, the states, the countdowns, and the
active cues following each step, the stage showing each video and fading to
black, and the panic leaving nothing active.  The GNU/Linux and Windows
runners have no audio device, so cinq shows its warning of no audio output
there, and the sounds run on their clocks without being heard; Darwin played
them, with no warnings.
