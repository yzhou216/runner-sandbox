// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs of the show, and the levels at which its cues send to them
//!
//! Cues send to outputs by their identities, so that renaming an output
//! leaves the cues alone.  Deleting an output leaves the sends to it in the
//! cues too, where they play nowhere and stay out of the show file, so
//! that undo, which brings the output back, brings back what played there.
//! A show keeps at least one output, and no two outputs share a name, since
//! the show file names outputs by their names.

mod change;
mod insertion;

use std::mem;

use self::change::OutputChange;
use super::Workspace;
use crate::back::{Cue, Output, OutputEdit, OutputId, Sends, items::replaced_where};

impl Workspace {
    pub fn outputs(&self) -> &[Output] {
        &self.outputs
    }

    pub fn output(&self, id: OutputId) -> Option<&Output> {
        self.outputs.iter().find(|output| output.id == id)
    }

    /// Returns the levels at which `cue` sends to the outputs that the
    /// show has: those of the cue, or the first output at 0 dB for a cue
    /// that names none
    pub fn sends(&self, cue: &Cue) -> Sends {
        match &cue.sends {
            Some(sends) => sends
                .iter()
                .filter(|(output, _)| self.output(**output).is_some())
                .map(|(output, level)| (*output, *level))
                .collect(),
            None => self
                .outputs
                .iter()
                .take(1)
                .map(|output| (output.id, 0.0))
                .collect(),
        }
    }

    /// Returns the sends of a new cue, which the outputs give, or none
    /// where those are the sends of a cue that names no outputs
    pub(super) fn new_sends(&self) -> Option<Sends> {
        let sends: Sends = self
            .outputs
            .iter()
            .filter_map(|output| Some((output.id, output.new_cues?)))
            .collect();
        let first = self.outputs.iter().take(1).map(|output| (output.id, 0.0));
        (!sends.iter().map(|(id, level)| (*id, *level)).eq(first)).then_some(sends)
    }

    /// Deletes outputs as one change, unless that would leave the show none,
    /// and records whether they were cut to the register of outputs
    pub(super) fn deleted_outputs(self, ids: &[OutputId], cut: bool) -> Self {
        let outputs: Vec<Output> = self
            .outputs
            .iter()
            .filter(|output| !ids.contains(&output.id))
            .cloned()
            .collect();
        if outputs.is_empty() || outputs.len() == self.outputs.len() {
            return self;
        }
        OutputChange::to(outputs).with_cut(cut).recorded_on(self)
    }

    /// Moves an output by a number of places, downwards when positive, as
    /// far as the ends of the outputs allow
    pub(super) fn moved_output(self, id: OutputId, by: isize) -> Self {
        let Some(from) = self.outputs.iter().position(|output| output.id == id) else {
            return self;
        };
        let to = from
            .saturating_add_signed(by)
            .min(self.outputs.len().saturating_sub(1));
        let mut outputs = self.outputs.clone();
        let output = outputs.remove(from);
        outputs.insert(to, output);
        OutputChange::to(outputs)
            .with_anchor(Some(id))
            .recorded_on(self)
    }

    /// Edits one setting of an output, apart from a name that is empty or
    /// that another output has
    pub(super) fn edited_output(self, id: OutputId, edit: OutputEdit) -> Self {
        let taken = |name: &str| {
            name.trim().is_empty()
                || self
                    .outputs
                    .iter()
                    .any(|output| output.id != id && output.name == name)
        };
        if matches!(&edit, OutputEdit::Name(name) if taken(name)) {
            return self;
        }
        let setting = mem::discriminant(&edit);
        let outputs = replaced_where(
            self.outputs.clone(),
            |output| output.id == id,
            |output| output.edited(edit),
        );
        OutputChange::to(outputs)
            .with_setting(id, setting)
            .recorded_on(self)
    }

    /// Puts `outputs` in place of those of the show without recording a
    /// change, as undo and redo do
    pub(super) fn with_outputs(self, outputs: Vec<Output>) -> Self {
        let next_output = outputs
            .iter()
            .map(|output| output.id.0 + 1)
            .max()
            .unwrap_or(1)
            .max(self.next_output);
        Self {
            outputs,
            next_output,
            ..self
        }
    }
}
