// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a change to the show says of itself, as the status bar tells undo and redo

use std::cmp::Ordering;

mod edit;
mod patch;

use super::{consequences, number, of_cues, short, the_output};
use crate::back::{Change, CueId, Output, OutputId, Workspace, output_moved_between};

/// Describes a change to the show in few words, starting in lower case, as
/// the status bar tells what undo or redo acts on
pub fn change(workspace: &Workspace, change: &Change) -> String {
    let named = |id| workspace.cue(id).map_or_else(String::new, number);
    match change {
        Change::Added { cues, .. } => {
            let told = of_cues("added", cues.iter().map(|placed| &placed.cue));
            // Every cue of one add takes the same depth
            match cues.first() {
                Some(placed) if placed.cue.depth > 0 => format!("{told} in a group"),
                _ => told,
            }
        }
        Change::Pasted { cues, .. } => of_cues("pasted", cues.iter().map(|placed| &placed.cue)),
        Change::Deleted { cues, cut, .. } => of_cues(
            if *cut { "cut" } else { "deleted" },
            cues.iter().map(|placed| &placed.cue),
        ),
        Change::Moved { cue, .. } => format!("moved cue {}", named(*cue)),
        Change::Regrouped {
            cues,
            moved: true,
            before,
            after,
            ..
        } => cues.first().map_or_else(String::new, |&cue| {
            moved_across(
                &named(cue),
                Change::depth_in(after, cue).cmp(&Change::depth_in(before, cue)),
            )
        }),
        Change::Regrouped { cues, deeper, .. } => regrouped(&named, cues, *deeper),
        Change::Edited { cue, redo, .. } => edit::edited(&named(*cue), redo),
        Change::Standby(standby) => match standby.after.and_then(|id| workspace.cue(id)) {
            Some(cue) => format!("moved standby to cue {}", number(cue)),
            None => "moved standby past the end".to_owned(),
        },
        Change::Patched {
            edit,
            before,
            after,
        } => patch::patched(edit, before, after),
        Change::Restored { .. } => "brought back the unsaved changes".to_owned(),
        Change::Outputs {
            before,
            after,
            cut,
            pasted,
            anchor,
            ..
        } => outputs(workspace, before, after, (*cut, *pasted), *anchor),
    }
}

/// Describes a change to the outputs by what differs between them before
/// and after it, leaving the words that name an output as one where the
/// change did something to the cues, which [`consequences()`] tells first
fn outputs(
    workspace: &Workspace,
    before: &[Output],
    after: &[Output],
    (cut, pasted): (bool, bool),
    selected: Option<OutputId>,
) -> String {
    let terse = consequences(workspace, before, after).is_some();
    let the = |name: &str| the_output(name, terse);
    let removed = if cut { "cut" } else { "deleted" };
    let added = if pasted { "pasted" } else { "added" };
    let pairs = || {
        before.iter().filter_map(|was| {
            let now = after.iter().find(|now| now.id == was.id)?;
            Some((was, now))
        })
    };
    let edited = pairs().find_map(|(was, now)| {
        let name = &short(&now.name);
        if was.name != now.name {
            Some(format!("renamed {} to {name}", short(&was.name)))
        } else if was.device != now.device {
            Some(format!("changed the device of {name}"))
        } else if was.channels != now.channels {
            Some(format!("changed the channels of {name}"))
        } else if was.new_cues != now.new_cues {
            Some(format!("changed what new cues send to {name}"))
        } else {
            None
        }
    });
    let moved = || {
        output_moved_between(before, after, selected).map(|now| format!("moved {}", the(&now.name)))
    };
    match (
        missing(after, before).as_slice(),
        missing(before, after).as_slice(),
    ) {
        ([output], []) => Some(format!("{added} {}", the(&output.name))),
        ([], [output]) => Some(format!("{removed} {}", the(&output.name))),
        (put, []) if !put.is_empty() => Some(format!("{added} {} outputs", put.len())),
        ([], gone) if !gone.is_empty() => Some(format!("{removed} {} outputs", gone.len())),
        (put, deleted) if !put.is_empty() || !deleted.is_empty() => {
            Some("changed the outputs".to_owned())
        }
        _ => edited.or_else(moved),
    }
    .unwrap_or_else(|| "changed the outputs".to_owned())
}

fn missing<'a>(from: &'a [Output], of: &[Output]) -> Vec<&'a Output> {
    from.iter()
        .filter(|output| of.iter().all(|other| other.id != output.id))
        .collect()
}

/// Tells that the cue numbered `number` moved, and whether it went into a
/// group or out of one by how its depth `compared` after the move
fn moved_across(number: &str, compared: Ordering) -> String {
    match compared {
        Ordering::Greater => format!("moved cue {number} into a group"),
        Ordering::Less => format!("moved cue {number} out of a group"),
        Ordering::Equal => format!("moved cue {number}"),
    }
}

/// Tells that `cues` went `deeper` into groups or out of them, by the number
/// of the cue, or else by how many cues
fn regrouped(named: &dyn Fn(CueId) -> String, cues: &[CueId], deeper: bool) -> String {
    let way = if deeper { "into" } else { "out of" };
    match cues {
        [cue] => format!("moved cue {} {way} a group", named(*cue)),
        cues => format!("moved {} cues {way} groups", cues.len()),
    }
}
