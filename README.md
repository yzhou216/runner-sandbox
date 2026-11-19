<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# milestone-1: scoring gameplay video

`milestone-1/` is a `git archive` of the `score` branch of the Bevy 0.18
block-dropping game, commit `066a4e11755a55898c3b1d4b4a87b2e85dd6a5cc`, which
enables `config`, `collision` and `score`.  It keeps the license of its
project.  All jobs run on `ubuntu-24.04`, rendering with Mesa's llvmpipe
through Vulkan.

## Gameplay video

The `video` job builds the game with the `test` feature and replays the four
`score-deterministic` recordings of the instructor in its `--test` mode, with
score checks on, which compares the game state against the recorded snapshots
at every fixed step.  `record.sh` makes Bevy's CI testing plugin save a
screenshot of every second frame under Xvfb and joins them into a 32 fps
video at normal speed.  Each frame advances the game by exactly 1/64 s, so the
timing in the videos is exact and does not depend on how fast the runner
renders.

## Checks

The `checks` job runs the course CI's commands (`cargo fmt --check`,
`cargo clippy --no-default-features --features ci -- -D warnings`, and
`cargo nextest run --no-default-features --features ci --retries 2`).
