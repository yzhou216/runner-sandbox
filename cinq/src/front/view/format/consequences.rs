// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a change to the outputs did to the cues, as the status bar tells it
//!
//! Cues that name no sends play on the first output, and cues that send only
//! to outputs that have gone play on none, so that a change to the outputs can
//! move the sound of cues that it never touched.  The notice puts that first
//! and short, since it is what the operator cannot see, and the bar cuts the
//! end of a notice that does not fit.

use itertools::Itertools;

use super::count;
use crate::{
    back::{Change, Output, Workspace, one_line},
    front::view::widget::shortened,
};

/// Most columns of the name of an output that a notice keeps, so that what it
/// tells fits in a bar of 1024 pixels with the key of undo after it, beside the
/// shortest name of the show
const NAME_ROOM: usize = 10;

/// Tells how many cues send to no output and where the cues that name no
/// outputs play, when a change to the outputs, from `before` to `after`,
/// changed either, the worse first, in lower case
pub fn consequences(workspace: &Workspace, before: &[Output], after: &[Output]) -> Option<String> {
    let was = came_from(workspace, before, after);
    let told = [stranding(workspace, was), rerouting(workspace, was)]
        .into_iter()
        .flatten()
        .join(", ");
    (!told.is_empty()).then_some(told)
}

/// Tells what `change` did to the cues, if it is a change to the outputs
/// that did
pub fn consequences_of(workspace: &Workspace, change: &Change) -> Option<String> {
    match change {
        Change::Outputs { before, after, .. } => consequences(workspace, before, after),
        _ => None,
    }
}

/// Puts `facts`, if there are any, before `told` and capitalizes the first
/// letter of what comes first
pub fn leading(facts: Option<String>, told: &str) -> String {
    let text = facts.map_or_else(|| told.to_owned(), |facts| format!("{facts}, {told}"));
    text.chars()
        .take(1)
        .flat_map(char::to_uppercase)
        .chain(text.chars().skip(1))
        .collect()
}

/// Cuts the name of an output to the room that a notice keeps for it
pub(super) fn short(name: &str) -> String {
    shortened(&one_line(name), NAME_ROOM)
}

/// Returns the outputs that the change from `before` to `after` started
/// from, which are `after` while the show holds `before`, as it does once the
/// change is undone
fn came_from<'a>(workspace: &Workspace, before: &'a [Output], after: &'a [Output]) -> &'a [Output] {
    if workspace.outputs() == before {
        after
    } else {
        before
    }
}

fn rerouting(workspace: &Workspace, was: &[Output]) -> Option<String> {
    let first = workspace.outputs().first()?;
    let moved = was.first().is_none_or(|was| was.id != first.id);
    let unrouted = workspace.cues().any(|cue| cue.sends.is_none());
    (moved && unrouted).then(|| format!("cues without sends play on {}", short(&first.name)))
}

/// Tells how many cues send to no output now, when the change added to them
fn stranding(workspace: &Workspace, was: &[Output]) -> Option<String> {
    let now = workspace.stranded().count();
    (now > workspace.stranded_among(was)).then(|| {
        let verb = if now == 1 { "sends" } else { "send" };
        format!("{} now {verb} to no output", count(now, "cue"))
    })
}
