// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the cues that act on a group and of what waits inside it

mod control;
mod readying;
mod stopping;
mod targeting;

use super::{groups_playing::run_of, *};

/// Returns a show whose group 1 holds a cue that plays, one that waits two
/// seconds, and a group, with a cue after it that waits for its post-wait
fn grouped() -> Vec<Cue> {
    vec![
        Cue {
            advance: Advance::Continue,
            post_wait: 4.0.into(),
            ..group(1)
        },
        within(1, audio(2)),
        pre_waiting(2.0, within(1, group(3))),
        within(2, audio(4)),
        audio(5),
    ]
}

fn all_paused(workspace: &Workspace) -> Vec<bool> {
    workspace
        .active()
        .iter()
        .map(|active| active.paused)
        .collect()
}
