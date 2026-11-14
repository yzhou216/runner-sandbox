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

use std::mem;

use super::{Change, Workspace};
use crate::back::{Cue, Output, OutputEdit, OutputId, Sends};

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

    /// Adds an output at `at` among the outputs, or at the end past them,
    /// on the default device and its first two channels, under the first
    /// name of the form Output 2 that no output has
    pub(super) fn added_output(self, at: usize) -> Self {
        let id = OutputId(self.next_output);
        let name = (1..)
            .map(|count| format!("Output {count}"))
            .find(|name| self.outputs.iter().all(|output| output.name != *name))
            .expect("the outputs run out before the names do");
        let output = Output {
            name,
            new_cues: None,
            ..Output::main(id)
        };
        let at = at.min(self.outputs.len());
        let outputs = [&self.outputs[..at], &[output], &self.outputs[at..]].concat();
        Self {
            next_output: self.next_output + 1,
            ..self
        }
        .with_outputs_recorded(outputs, None)
    }

    /// Deletes an output, unless it is the only one
    pub(super) fn deleted_output(self, id: OutputId) -> Self {
        if self.outputs.len() < 2 {
            return self;
        }
        let outputs = self
            .outputs
            .iter()
            .filter(|output| output.id != id)
            .cloned()
            .collect();
        self.with_outputs_recorded(outputs, None)
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
        self.with_outputs_recorded(outputs, None)
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
        let setting = Some((id, mem::discriminant(&edit)));
        let outputs = self
            .outputs
            .iter()
            .map(|output| {
                if output.id == id {
                    output.clone().edited(edit.clone())
                } else {
                    output.clone()
                }
            })
            .collect();
        self.with_outputs_recorded(outputs, setting)
    }

    fn with_outputs_recorded(
        self,
        outputs: Vec<Output>,
        setting: Option<(OutputId, mem::Discriminant<OutputEdit>)>,
    ) -> Self {
        let before = self.outputs.clone();
        let change = Change::Outputs {
            before,
            after: outputs.clone(),
            setting,
        };
        self.with_outputs(outputs).recorded(change)
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
