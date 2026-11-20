<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq live edits of levels, before and after

Session of 2026-11-19 and 2026-11-20 that records cinq on a hosted runner
while keys edit the levels of cues that play, once for master
(`d554f526b0855cdb941ba8797a51628e362dac20`, before) and once for the branch
`wip/live-edits` of github:yzhou216/cinq (`37a17f59735d24a1a832c8da8c1ba6f81faac205`,
after).  Both builds get the same keys.

## What was tested

- Runner: `ubuntu-latest`, cinq built with `nix build github:yzhou216/cinq/<commit>`.
- Recording: headless sway (`sway-unwrapped`, pixman), `wtype` for every key,
  `wf-recorder` for the screen, a private PulseAudio with a null sink whose
  monitor `parec` records.  `live/compose.sh` ties the three clocks together,
  draws a graph of the heard RMS level, and burns captions.  `live/side.sh`
  puts the two videos side by side, before on the left channel and after on
  the right one.
- Show: a looping 440 Hz tone (cue 1, -20 dB), a fade to -30 dB in 2 s (cue 2),
  and a fade to -6 dB in 12 s (cue 3), neither stopping the tone.
- Keys: Space (GO), `k`, `3gt`, Tab into the Level field, `-1`, `2`, Enter,
  Space, `gg`, Tab, `-9`, Enter, Space, `G`, Tab into Fade target to, `-30`,
  Enter, Escape.  See `live/record.sh`.
- Plugging in another build: edit the `after` line of `live/refs.txt`.

## Runs

- 44689787320: failed in both record jobs.  `result-cinq` is the package
  directory, and `record.sh` ran it as a program ("Is a directory").  `run.sh`
  now takes a directory and runs its `bin/cinq`.
- 44695529024: both recordings passed; the side-by-side job failed (ffmpeg out
  link named `ffmpeg`, artifact paths differ).
- 44697609766: all jobs passed.  Videos `before.mp4`, `after.mp4`, and
  `side-by-side.mp4` are its artifacts.

## Result (run 44697609766)

The tone peaks at -3 dBFS, so the RMS heard is the cue's level minus 6.01 dB.

| End of the wait | Before: RMS (dB) | Before: cue level (dB) | After: RMS (dB) | After: cue level (dB) |
| --- | ---: | ---: | ---: | ---: |
| Tone has played 3 s at -20 dB | -26.0 | -20.0 | -26.0 | -20.0 |
| -1 typed, 3 s later, no Enter | -7.0 | -1.0 | -26.0 | -20.0 |
| -12 typed, 3 s later, no Enter | -18.0 | -12.0 | -26.0 | -20.0 |
| Enter pressed, 3 s later | -18.0 | -12.0 | -18.0 | -12.0 |
| Duck ended, 4 s later | -36.0 | -30.0 | -36.0 | -30.0 |
| Level edited to -9, 4 s later | -15.0 | -9.0 | -33.0 | -27.0 |
| Swell has run 4 s | -14.2 | -8.2 | -27.1 | -21.1 |
| Fade target edited to -30, 10 s later | -12.0 | -6.0 | -36.0 | -30.0 |

The after build behaves as expected in every step.  The same table came out of
the first run and of a run on this machine.
