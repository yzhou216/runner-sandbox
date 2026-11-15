<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# milestone-1: cumulative-sum line drops

`milestone-1/` is a `git archive` of the block-dropping game's `hold`
branch, commit `4b27b64800288017a922d3e8ea0b20d9eb6505ad`, after rebasing it onto `main` and changing
`delete_full_lines` to compute how far each line drops with a cumulative sum,
as requested in review.  It keeps the license of its project.

The `checks` job runs the course CI's commands on `ubuntu-24.04`:
`cargo fmt --check`,
`cargo clippy --no-default-features --features ci -- -D warnings`, and
`cargo nextest run --no-default-features --features ci --retries 2`.

Run 43668025827: fmt and clippy pass, and all 62 tests pass, including the
score, level-up and hard-drop replays, which clear lines and compare every
state with the instructor's recordings.  `gravity_and_input`, which paces
itself with `thread::sleep`, passed on its third try.
