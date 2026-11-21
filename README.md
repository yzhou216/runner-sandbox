<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq group cues

Session of 2026-11-21 that records cinq on a hosted runner while a group cue
fires the cues that it holds, the keys that indent and outdent a cue move one
in and out of the group, and a stop cue stops the group.  The build is
`github:yzhou216/cinq/6e13597f02d0b324812e6a2184f96ab66b090f17` (branch
`wip/groups`), and the tools come from the nixpkgs of the same commit.  It
reuses the recording scripts of the branch `cinq/2026-11-20-devamp`.

## What was tested

- Runner: `ubuntu-latest`, with a headless sway, `wtype` for every key,
  `wf-recorder` for the screen, and a private PulseAudio whose null sink
  `parec` records.  `live/compose.sh` joins the screen, a graph of the heard
  RMS level, and captions.
- Show (`live/make-show.sh`): the rain is 4 s of 440 Hz at -30 dBFS and loops,
  and the thunder is 3 s of 660 Hz at -14 dBFS.  List Main: 1 group "Scene",
  then at `depth = 1` the audio "Rain" with `loop = true` and the audio
  "Thunder" with `pre_wait = 2.0`, then the stop cue "End the scene" that
  targets the group, at depth 0.  The cue list numbers them 1, 1.1, 1.2, 2.
- Keys (`live/record.sh`), one `wtype -s 300` call each, in the vi keymap that
  is the default: `G` selects the stop cue, `>` `>` indents it into the group,
  `<` `<` outdents it, `g` `g` and `Enter` put the group on standby, and GO and
  Panic are F5 and F12, bound in `config.toml`, so no key is Space or Escape.
  F5 fires the group, and F5 again 6.6 s later fires the stop cue.

## Runs

- 44884498816: passed, first push.  Its artifact `groups` holds `groups.mp4`,
  the screenshots, and the logs.

## Result (run 44884498816)

Times are seconds in `groups.mp4`; the key stamps are taken when `wtype`
returns, which is about 0.3 s after the key.  GO is at 12.0 s and the stop
cue at 18.6 s.

| Time (s) | RMS (dB) | What |
| ---: | ---: | --- |
| 0 to 11.75 | silence | list, `>>`, `<<` (the stop cue moves in under the thunder and out again) |
| 12.25 to 13.75 | -33.0 | the rain alone, as the group is running |
| 14.25 to 16.75 | -16.9 | the rain and the thunder, from 2 s after GO for 3 s |
| 17.25 to 18.25 | -33.0 | the rain alone again |
| 18.75 to the end | silence | after the stop cue |

Standby moved to the stop cue at GO, the State column of the group read
`running`, and the Active list was empty after the stop.
