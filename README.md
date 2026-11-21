<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq loop sections and devamp cues

Session of 2026-11-20 that records cinq on a hosted runner while a sound
loops a section of its file and a devamp cue lets it go on.  The build is
`github:yzhou216/cinq/28022f87bc3b7222d5ebc1bfa6db0f1f16158614` (branch
`wip/devamp`), and the tools come from the nixpkgs of the same commit.  It
reuses the recording scripts of the branch `cinq/2026-11-20-control-cues`.

## What was tested

- Runner: `ubuntu-latest`, with a headless sway, `wtype` for every key,
  `wf-recorder` for the screen, and a private PulseAudio whose null sink
  `parec` records.  `live/compose.sh` joins the screen, a graph of the heard
  RMS level, and captions.
- Show (`live/make-show.sh`): one 10 s sound at 48 kHz in three parts.  The
  introduction (0 to 2 s) is 330 Hz at -30 dBFS.  The vamp (2 to 6 s) is
  440 Hz rising from -36 to -10 dBFS.  The ending (6 to 10 s) is 550 Hz
  falling from -10 dBFS to silence.  List Main: 1 audio "Scene music" with
  `loop = true`, `loop_start = 2.0`, `loop_end = 6.0`; 2 devamp "Leave the
  vamp" on 1.
- Keys (`live/record.sh`): GO is F5 and Panic is F12, bound in `config.toml`,
  so that no key is Space or Escape.  `2gt` shows the Time & Loops tab of cue 1
  (Loop start 2, Loop end 6), F5 fires cue 1, and F5 again 11.8 s later fires
  cue 2, 1.8 s into the third pass.

## Runs

- 44876513842: passed, first push.  Its artifact `devamp` holds
  `devamp.mp4`, the screenshots, and the logs.

## Result (run 44876513842)

Times are seconds after the first GO; levels are the RMS in dB of the video's
sound over 0.25 s, whose tone peak is 3.01 dB higher.

| Time after GO (s) | RMS (dB) | What |
| ---: | ---: | --- |
| 0.25 to 1.75 | -33.0 | introduction, steady |
| 2.25 | -37.2 | start of the first pass |
| 5.75 | -14.4 | top of the first pass |
| 6.25 | -37.2 | start of the second pass (loop end at 6.0) |
| 9.75 | -14.4 | top of the second pass |
| 10.25 | -37.2 | start of the third pass (loop end at 10.0) |
| 11.82 | - | cue 2 fired |
| 13.75 | -14.4 | top of the third pass |
| 14.0 | -13.8 | pass end: the ending starts at the top |
| 14.25 | -17.0 | ending falling |
| 16.0 | -43.3 | ending falling |
| 17.75 | -69.5 | ending nearly silent |
| 18.25 | silence | sound over, 4 s after the pass end |

The cue left the Active list and the State column once the ending was over.
