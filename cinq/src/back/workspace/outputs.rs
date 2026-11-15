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

    /// Returns the first name that `name` makes from the numbers from one on
    /// and that neither an output of the show nor one of `also` has
    fn unused_name(&self, also: &[Output], name: impl Fn(usize) -> String) -> String {
        (1..)
            .map(name)
            .find(|name| {
                self.outputs
                    .iter()
                    .chain(also)
                    .all(|output| output.name != *name)
            })
            .expect("the outputs run out before the names do")
    }

    /// Adds `count` outputs in a row at `at` among the outputs, or at the
    /// end past them, as one change, each on the default device and its first
    /// two channels, under the first name of the form Output 1 that no output
    /// has
    pub(super) fn added_output(self, at: usize, count: usize) -> Self {
        let added = (0..count).fold(Vec::<Output>::new(), |added, offset| {
            let id = OutputId(self.next_output + offset as u32);
            let name = self.unused_name(&added, |number| format!("Output {number}"));
            let output = Output {
                name,
                new_cues: None,
                ..Output::main(id)
            };
            [added, vec![output]].concat()
        });
        self.inserted(at, added)
    }

    /// Pastes `outputs` in a row at `at` among the outputs, as one change.
    /// With `restore`, an output whose identity no output of the show has
    /// takes it back, with the level of its new cues, so that the cues that
    /// sent to it send to it again, as a line cut and pasted back in Vim is
    /// the line that was.  Every other output is a copy: it has an identity of
    /// its own, no cue sends to it, and no new cue sends to it either.  A name
    /// that an output has gets the lowest number from 2 on that makes it free.
    pub(super) fn pasted_outputs(self, at: usize, outputs: Vec<Output>, restore: bool) -> Self {
        let (pasted, _) = outputs.into_iter().fold(
            (Vec::<Output>::new(), self.next_output),
            |(pasted, next), output| {
                let back = restore
                    && self.output(output.id).is_none()
                    && pasted.iter().all(|placed| placed.id != output.id);
                let name = self.unused_name(&pasted, |number| match number {
                    1 => output.name.clone(),
                    number => format!("{} {number}", output.name),
                });
                let placed = if back {
                    Output { name, ..output }
                } else {
                    Output {
                        id: OutputId(next),
                        name,
                        new_cues: None,
                        ..output
                    }
                };
                let next = if back { next } else { next + 1 };
                ([pasted, vec![placed]].concat(), next)
            },
        );
        self.inserted(at, pasted)
    }

    /// Puts `added` among the outputs at `at`, or at the end past them, as
    /// one change, and nothing when there is nothing to put
    fn inserted(self, at: usize, added: Vec<Output>) -> Self {
        if added.is_empty() {
            return self;
        }
        let at = at.min(self.outputs.len());
        let outputs = [&self.outputs[..at], &added, &self.outputs[at..]].concat();
        self.with_outputs_recorded(outputs, None)
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
        self.with_outputs_changed(outputs, None, cut)
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
        self.with_outputs_changed(outputs, setting, false)
    }

    fn with_outputs_changed(
        self,
        outputs: Vec<Output>,
        setting: Option<(OutputId, mem::Discriminant<OutputEdit>)>,
        cut: bool,
    ) -> Self {
        let before = self.outputs.clone();
        let change = Change::Outputs {
            before,
            after: outputs.clone(),
            setting,
            cut,
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
