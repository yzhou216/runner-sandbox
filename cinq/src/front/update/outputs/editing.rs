// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Adding, moving, and deleting outputs around the selected one, as the
//! keys of cues do for cues
//!
//! Show mode locks all of it, as it does for cues.  Cues that name no sends
//! play on the first output, so that a change to the first output changes
//! where they play, which the notice of the change tells.

use crate::{
    back::{Command, Output},
    front::App,
};

/// Most outputs that one add or one paste adds, which keeps a count typed by
/// mistake from filling a show with outputs that no device has the channels
/// for
pub(super) const LIMIT: usize = 64;

/// What the status bar tells when a cut or a delete would take every output
/// away
const KEEPS_ONE: &str = "A show keeps at least one output";

impl App {
    /// Adds `count` outputs in a row below the selected one, or above it,
    /// and selects the last of them, as Vim leaves its cursor on the last
    /// line that `3o` inserts
    pub(super) fn with_new_outputs(self, above: bool, count: usize) -> Self {
        // Show mode adds nothing, so the selection stays where it is
        if !self.editable() {
            return self;
        }
        let at = self.insertion(above);
        let count = count.clamp(1, LIMIT);
        let app = self.send(Command::AddOutput { at, count });
        let last = app.workspace.outputs().len().saturating_sub(1);
        app.selecting(Some(at.saturating_add(count - 1).min(last)))
    }

    /// Returns the row where outputs go below the selected output, or above
    /// it, or past the last without a selection
    pub(super) fn insertion(&self, above: bool) -> usize {
        self.output_row()
            .map_or(usize::MAX, |row| if above { row } else { row + 1 })
    }

    pub(super) fn with_output_moved(self, by: isize) -> Self {
        match self.output {
            Some(output) => self.send(Command::MoveOutput { output, by }),
            None => self,
        }
    }

    /// Deletes the selected output and the outputs below it, as many as
    /// `count` holds, and selects the one that takes its place, leaving the
    /// register of outputs as it is
    pub(in crate::front::update) fn without_outputs(self, count: usize) -> Self {
        self.taking_outputs(count, false).0
    }

    /// Removes the selected output and the outputs below it as
    /// [`App::without_outputs`] does, as a cut when `cut`, and returns the
    /// outputs removed.  A show keeps at least one output, so that a count
    /// that reaches all of them removes none and tells so.  Show mode
    /// removes none.
    pub(super) fn taking_outputs(self, count: usize, cut: bool) -> (Self, Vec<Output>) {
        let outputs = self.workspace.outputs();
        let first = match self.output_row() {
            Some(first) if self.editable() => first,
            _ => return (self, Vec::new()),
        };
        let taken: Vec<Output> = outputs[first..].iter().take(count).cloned().collect();
        if taken.len() == outputs.len() {
            let app = Self {
                remark: Some(KEEPS_ONE),
                ..self
            };
            return (app, Vec::new());
        }
        let ids = taken.iter().map(|output| output.id).collect();
        let app = self.send(if cut {
            Command::CutOutputs(ids)
        } else {
            Command::DeleteOutputs(ids)
        });
        let last = app.workspace.outputs().len().saturating_sub(1);
        (app.selecting(Some(first.min(last))), taken)
    }
}
