// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the text that describes cues

mod chains;
mod changes;
mod playing;
mod standby;
mod targets;

use super::{activity, chain, change, clipped, moving, paused, phase, target};
use crate::back::{
    Advance, Command, Cue, CueId, CueList, Edit, InstrumentId, Kind, OutputId, PatchEdit, Place,
    Sends, Workspace, fresh, group, loaded, memo, sound,
};

/// Returns what the latest change to `workspace` says of itself
fn told(workspace: &Workspace) -> String {
    let (_, latest) = workspace.latest_change().expect("a command is a change");
    change(workspace, latest)
}

/// Returns the chain that GO fires from the cue `standby` of the first list,
/// while the outputs `failing` have failed
fn chained(workspace: &Workspace, standby: u32, failing: &[OutputId]) -> String {
    let cue = workspace.cue(CueId(standby)).expect("it exists");
    chain(workspace, &workspace.lists()[0], cue, failing)
}
