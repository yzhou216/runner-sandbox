// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Undoing and redoing changes while the outputs show
//!
//! A change to the outputs selects the output that it touched: one that it
//! brought back or added, one that it edited or moved, or the neighbor of one
//! that it took away, except that undoing an addition or a paste selects the
//! output that was selected when it was made.  A change to a cue would happen
//! out of sight, so that the cue list shows in place of the outputs, as a jump
//! that going back returns from, and the changes that remain go on there.
//! Show mode takes back no change but those of standby, which are out of
//! sight as well, so that undo and redo leave the outputs as they are there.

use std::ops::ControlFlow;

use crate::{
    back::{Change, Command, Output, OutputId, Workspace, output_moved_between},
    front::App,
};

impl App {
    /// Undoes up to `count` changes
    pub(in crate::front::update) fn undone_outputs(self, count: usize) -> Self {
        self.turned_in_outputs(count, &Command::Undo, Workspace::undoable, Self::undone)
    }

    /// Redoes up to `count` changes
    pub(in crate::front::update) fn redone_outputs(self, count: usize) -> Self {
        self.turned_in_outputs(count, &Command::Redo, Workspace::redoable, Self::redone)
    }

    /// Sends `command` once for each of `count` changes that `next` finds,
    /// one at a time so that the selection and the notice follow the last of
    /// them, until one is no change to the outputs, where `in_cues` takes
    /// over for the changes that remain
    fn turned_in_outputs(
        self,
        count: usize,
        command: &Command,
        next: fn(&Workspace) -> Option<&Change>,
        in_cues: fn(Self, usize) -> Self,
    ) -> Self {
        if !self.editable() {
            return self;
        }
        let flow = (0..count).try_fold(self, |app, done| match next(&app.workspace) {
            Some(Change::Outputs { anchor, .. }) => {
                let anchor = *anchor;
                let before = app.workspace.outputs().to_vec();
                let app = app.send(command.clone());
                let touched = touched(
                    app.workspace.outputs(),
                    &before,
                    anchor.or(app.output),
                    anchor,
                );
                ControlFlow::Continue(Self {
                    output: touched.or(app.output),
                    ..app
                })
            }
            Some(_) => {
                let app = app.jumping(Self::showing_cues);
                ControlFlow::Break(in_cues(app, count - done))
            }
            None => ControlFlow::Break(app),
        });
        match flow {
            ControlFlow::Continue(app) | ControlFlow::Break(app) => app,
        }
    }
}

/// Returns the output of `now` that a change touched, `was` being the
/// outputs before the change.  An output that `was` lacked comes first, then
/// where outputs that were added went away, the `anchor` that was selected
/// when they were, as Vim's undo returns the cursor to where the change was
/// made, then the neighbor of the first that `now` lacks, then the first that
/// differs, and then the one that moved, which is `prefer` where two fit.
fn touched(
    now: &[Output],
    was: &[Output],
    prefer: Option<OutputId>,
    anchor: Option<OutputId>,
) -> Option<OutputId> {
    let absent = |from: &[Output], of: &[Output]| {
        from.iter()
            .position(|output| of.iter().all(|other| other.id != output.id))
    };
    if let Some(row) = absent(now, was) {
        return Some(now[row].id);
    }
    if let Some(row) = absent(was, now) {
        let anchored = anchor.filter(|id| now.iter().any(|output| output.id == *id));
        return anchored.or_else(|| now.get(row).or(now.last()).map(|output| output.id));
    }
    let differs = now
        .iter()
        .zip(was)
        .find(|(now, was)| now.id == was.id && now != was);
    if let Some((now, _)) = differs {
        return Some(now.id);
    }
    output_moved_between(was, now, prefer).map(|output| output.id)
}
