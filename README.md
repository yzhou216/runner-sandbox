<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq start, pause, and GoTo cues

Session of 2026-11-20 that records cinq on a hosted runner while keys fire a
pause cue, a start cue, and a GoTo cue.  The build is
`github:yzhou216/cinq/631ebe10cf16318d29e6bd9fce80cf7e82f32c99` (branch
`wip/control-cues`), and the tools come from the nixpkgs of the same commit.
It reuses the recording scripts of the branch `cinq/2026-11-19-live-edits`.

## What was tested

- Runner: `ubuntu-latest`, with a headless sway, `wtype` for every key,
  `wf-recorder` for the screen, and a private PulseAudio whose null sink
  `parec` records.  `live/compose.sh` joins the screen, a graph of the heard
  RMS level, and captions.
- Show (`live/make-show.sh`): a 440 Hz tone that rises from -40 dBFS to
  -6 dBFS in 20 s and does not loop.  List Main: 1 audio "Rising tone", 2 pause
  "Hold the tone", 3 start "Resume the tone", 4 GoTo "Back to the tone", each
  of the last three on cue 1.  List Effects: 1 start "Start the tone from
  Effects", on cue 1 of Main.
- Keys (`live/record.sh`): GO is F5 and Panic is F12, bound in `config.toml`,
  so that no key is Space or Escape.  F5 on 1, 2, 3, 4, 1, then `l` to Effects
  and F5, then `h` back to Main and F12.

## Runs

- 44821543525: passed.  One caption, of the pause step, was wider than the
  video and was cut off.
- 44823978540: passed, with the caption shortened.  Its artifact
  `control-cues` holds `control-cues.mp4`, the screenshots, and the logs.

## Result (run 44823978540)

The tone peaks at the level shown, and its RMS is 3.01 dB lower.

| End of the wait | RMS (dB) | Tone peak (dBFS) |
| --- | ---: | ---: |
| GO on 1: tone has risen for 4 s | -37.2 | -34.2 |
| GO on 2: paused, 3 s later | silence | silence |
| GO on 3: first second after the resume | -34.4 | -31.4 |
| GO on 3: 4 s after the resume | -29.8 | -26.8 |
| GO on 4: standby on 1, tone goes on | -25.8 | -22.8 |
| GO on 1: first second after the restart | -41.8 | -38.8 |
| GO on 1: 3 s after the restart | -38.9 | -35.9 |
| Start cue of Effects: first second after the restart | -41.8 | -38.8 |
| Start cue of Effects: 3 s after the restart | -38.9 | -35.9 |

The pause is silent, the resume carries on from the held level, and both the
GO on cue 1 and the start cue of Effects drop the tone back to the bottom.
The State column and the sidebar read "paused" during the pause, standby sits
on cue 1 after the GoTo cue, and standby of Main stays on cue 2 after the
start cue of Effects.
