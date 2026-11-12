// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs of the show, which the main window shows in place of a cue list,
//! and the settings that apply once the text typed into them reads as a
//! value
//!
//! The keys that move through and edit cues move through and edit the
//! outputs while they show, so that adding, deleting, moving, and renaming
//! an output takes the keys of a cue.

use crate::{
    back::{Channels, Command, Edit, OutputEdit, OutputId, parse_level},
    front::{App, Setting, step},
};

impl App {
    pub(super) fn showing_outputs(self) -> Self {
        let output = self
            .output
            .filter(|id| self.workspace.output(*id).is_some())
            .or_else(|| self.workspace.outputs().first().map(|output| output.id));
        Self {
            outputs_shown: true,
            output,
            ..self
        }
    }

    /// Returns the place of the output that the inspector shows
    pub(in crate::front) fn output_row(&self) -> Option<usize> {
        let outputs = self.workspace.outputs();
        self.output
            .and_then(|id| outputs.iter().position(|output| output.id == id))
    }

    fn with_output_at(self, row: Option<usize>) -> Self {
        let output = row
            .and_then(|row| self.workspace.outputs().get(row))
            .map(|output| output.id);
        Self { output, ..self }
    }

    pub(super) fn stepped_output(self, offset: isize) -> Self {
        let row = step(self.output_row(), offset, self.workspace.outputs().len());
        self.with_output_at(row)
    }

    /// Selects the output on a row counted from one, or the last one past
    /// the end, as the keys of the first and last cue do
    pub(super) fn output_at_row(self, row: usize) -> Self {
        let last = self.workspace.outputs().len().saturating_sub(1);
        self.with_output_at(Some(row.saturating_sub(1).min(last)))
    }

    /// Adds an output below the one shown, or above it, and shows the new
    /// one
    pub(super) fn with_new_output(self, above: bool) -> Self {
        // Show mode adds nothing, so the selection stays where it is
        if !self.editable() {
            return self;
        }
        let at = self
            .output_row()
            .map_or(usize::MAX, |row| if above { row } else { row + 1 });
        let app = self.send(Command::AddOutput { at });
        let row = at.min(app.workspace.outputs().len().saturating_sub(1));
        app.with_output_at(Some(row))
    }

    pub(super) fn with_output_moved(self, by: isize) -> Self {
        match self.output {
            Some(output) => self.send(Command::MoveOutput { output, by }),
            None => self,
        }
    }

    /// Deletes the output shown and shows the one that takes its place
    pub(super) fn without_output(self) -> Self {
        let (Some(output), row) = (self.output, self.output_row()) else {
            return self;
        };
        let app = self.send(Command::DeleteOutput(output));
        let last = app.workspace.outputs().len().saturating_sub(1);
        app.with_output_at(row.map(|row| row.min(last)))
    }

    /// Applies what has been typed into the field of `setting` once it reads
    /// as a value, and keeps the text for the field to show meanwhile
    pub(super) fn typed_setting(self, setting: Setting, text: String) -> Self {
        // Level that the text gives, none for an empty field, or no level
        // at all for text that is no level yet
        let level = || match text.trim() {
            "" => Some(None),
            typed => parse_level(typed).map(Some),
        };
        let command = match setting {
            Setting::Send(cue, output) => level().and_then(|level| {
                let sends = self.workspace.sends(self.workspace.cue(cue)?);
                let sends = sends
                    .into_iter()
                    .filter(|(id, _)| *id != output)
                    .chain(level.map(|level| (output, level)))
                    .collect();
                Some(Command::Edit(cue, Edit::Sends(Some(sends))))
            }),
            Setting::OutputName(id) => Some(edit_output(id, OutputEdit::Name(text.clone()))),
            Setting::Channels(id) => Channels::parse(&text)
                .map(|channels| edit_output(id, OutputEdit::Channels(channels))),
            Setting::NewCues(id) => {
                level().map(|level| edit_output(id, OutputEdit::NewCues(level)))
            }
        };
        Self {
            typed: Some((setting, text)),
            ..self
        }
        .send_all(command)
    }
}

fn edit_output(id: OutputId, edit: OutputEdit) -> Command {
    Command::EditOutput(id, edit)
}
