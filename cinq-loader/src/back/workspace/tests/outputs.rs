// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the outputs of a show, of the sends of its cues to them, and of
//! both in show files
mod files;
mod lifecycle;
mod silencing;

use super::*;
use crate::back::{Channels, OutputEdit, OutputId, Sends};

const MAIN: OutputId = OutputId(1);

pub(super) fn names(workspace: &Workspace) -> Vec<&str> {
    workspace
        .outputs()
        .iter()
        .map(|output| output.name.as_str())
        .collect()
}

pub(super) fn edit_output(id: u32, edit: OutputEdit) -> Command {
    Command::EditOutput(OutputId(id), edit)
}

pub(super) fn rename_output(id: u32, name: &str) -> Command {
    edit_output(id, OutputEdit::Name(name.to_owned()))
}

pub(super) fn delete_outputs(ids: &[u32]) -> Command {
    Command::DeleteOutputs(ids.iter().copied().map(OutputId).collect())
}

pub(super) fn sends(pairs: &[(u32, f32)]) -> Sends {
    pairs
        .iter()
        .map(|&(id, level)| (OutputId(id), level))
        .collect()
}

/// Returns a show with House on the first two channels and Sub on the
/// third, to which new cues send at 0 and -6 dB
fn house_and_sub() -> Workspace {
    workspace(vec![audio(1)])
        .apply(rename_output(1, "House"))
        .apply(Command::add_output(1, 1))
        .apply(rename_output(2, "Sub"))
        .apply(edit_output(2, OutputEdit::Channels(Channels::Mono(2))))
        .apply(edit_output(2, OutputEdit::NewCues(Some(-6.0))))
}

/// Returns the command that makes cue 1 send at the levels of `pairs`
pub(super) fn send_on(pairs: &[(u32, f32)]) -> Command {
    edit(1, Edit::Sends(Some(sends(pairs))))
}

/// Returns where cue 1 sends, by output
pub(super) fn sends_of_cue(workspace: &Workspace) -> Sends {
    workspace.sends(workspace.cue(CueId(1)).expect("the cue is there"))
}
