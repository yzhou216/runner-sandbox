// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of GO on a group and of the run that it makes

mod advancing;
mod lasting;
mod panicking;
mod starting;

use super::*;

/// Returns the group `id` with the setting that goes on to the next cue
fn then(id: u32, advance: Advance, pre_wait: f32, post_wait: f32) -> Cue {
    Cue {
        advance,
        pre_wait: pre_wait.into(),
        post_wait: post_wait.into(),
        ..group(id)
    }
}

pub(super) fn run_of(workspace: &Workspace, id: u32) -> u32 {
    workspace.active_of(CueId(id)).expect("the cue runs").run
}
