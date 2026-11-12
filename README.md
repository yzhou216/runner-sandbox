<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# milestone-1: gameplay video and checks without hold

`milestone-1/` is a `git archive` of the `hold` branch of the Bevy 0.18
block-dropping game, commit `b0d075fc693e077281665bd34eadf8e2e8fa8cfa`, which
has every required feature enabled.  It keeps the license of its project.
All jobs ran on `ubuntu-24.04`, rendering with Mesa's llvmpipe through Vulkan.

## Gameplay video

The `video` job builds the game with the `test` feature and replays two of the
instructor's recordings in its `--test` mode, which checks the game state
against the recorded snapshots at every fixed step.  `record.sh` makes
Bevy's CI testing plugin save a screenshot of every second (or eighth) frame
under Xvfb and joins them into a 32 fps video.  Each frame advances the game
by exactly 1/64 s, so the timing in the videos is exact and does not depend on
how fast the runner renders.

- `hard-drop-hold-0.mp4`: seeded random bag (`config-0.json`), hard drop and
  hold, at normal speed (51 s).
- `level-up.mp4`: deterministic bag, scoring and two level-ups, at four
  times normal speed (78 s for 5 min 10 s of play).

Both replays logged `Passed with 0 mismatched states` and end with "Test
Passes!" on screen.  The `test` feature compiles out the game's
`setup_window`, so the script resizes the window to the 900 by 900 that
`setup_window` sets with `xdotool`, and drops the frames drawn before the
resize (none and one).

## Checks without hold

The `checks` matrix runs the course CI's commands (`cargo fmt --check`,
`cargo clippy --no-default-features --features ci -- -D warnings`, and
`cargo nextest run --no-default-features --features ci --retries 2`), once as
committed and once with `hold` removed from `enabled_features`.

- fmt and clippy pass both ways.
- Without hold, 56 of 58 tests pass; with hold, 60 or 61 of 62.
- In both, the failures are `basic_stacking` and `gravity_and_input`, which
  pace themselves with `thread::sleep` and fail on these runners whether or
  not hold is enabled.  The course CI's container passed every test on the same
  commit.

## Runs

- 43117491682: checks without hold as above; video failed because
  `libxkbcommon-x11-0` was missing.
- 43119594492: both checks as above; videos recorded but cropped to Bevy's
  default 1280 by 720 window.
- 43127369809: both checks as above; final videos, in the `videos` artifact.
