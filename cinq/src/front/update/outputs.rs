// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs of the show, which the main window shows in place of a cue list,
//! and the settings that apply once the text typed into them reads as a
//! value
//!
//! The keys that move through cues move through the outputs by the code that
//! the two share, and the keys that edit cues edit the outputs while they
//! show, so that adding, deleting, moving, and renaming an output takes the
//! keys of a cue.

mod clipboard;
mod editing;
mod undoing;

use iced::Task;

use crate::{
    back::{Channels, Command, CueId, Edit, OutputEdit, OutputId, Place, parse_level},
    front::{App, Message, Setting, view},
};

impl App {
    /// Shows the outputs in place of the cue list, closing a search line,
    /// which would otherwise search the list out of sight
    pub(super) fn showing_outputs(self) -> Self {
        let output = self
            .output
            .filter(|id| self.workspace.output(*id).is_some())
            .or_else(|| self.workspace.outputs().first().map(|output| output.id));
        Self {
            outputs_shown: true,
            output,
            ..self.without_search()
        }
    }

    /// Shows the cue list again in place of the outputs
    pub(super) fn showing_cues(self) -> Self {
        Self {
            outputs_shown: false,
            ..self
        }
    }

    /// Applies a message that acts on the rows of the cue list alone or
    /// changes them while the outputs show: the keys that add, move, copy, cut,
    /// paste, delete, undo, and redo do the same to outputs, each by the code
    /// of outputs, and every other such message does nothing, since the cue
    /// list is out of sight
    pub(super) fn in_outputs(self, message: Message) -> (Self, Task<Message>) {
        let above = |place: fn(CueId) -> Place| matches!(place(CueId(0)), Place::Before(_));
        let app = match message {
            Message::Add(place, count) => self.with_new_outputs(above(place), count),
            Message::Move(by) => self.with_output_moved(by),
            Message::Copy(count) => self.copied_outputs(count),
            Message::Cut(count) => self.cut_outputs(count),
            Message::Paste(place, count) => self.with_pasted_outputs(above(place), count),
            Message::Delete(count) => self.without_outputs(count),
            Message::Across(operator, motion, count) => self.across(operator, motion, count),
            Message::Undo(count) => self.undone_outputs(count),
            Message::Redo(count) => self.redone_outputs(count),
            _ => return (self, Task::none()),
        };
        app.revealed()
    }

    /// Returns the place of the output that the inspector shows
    pub(in crate::front) fn output_row(&self) -> Option<usize> {
        let outputs = self.workspace.outputs();
        self.output
            .and_then(|id| outputs.iter().position(|output| output.id == id))
    }

    /// Applies the channels typed into the field of an output once the
    /// operator has left it.  They wait until then, since a device would
    /// open for each of `1`, `12`, and `12-13` on the way to the last.  The
    /// other settings apply as they read, for they cost no device.
    pub(super) fn settled(self) -> Self {
        let command = match &self.typed {
            Some((Setting::Channels(id), text)) => Channels::parse(text)
                .filter(|channels| {
                    self.workspace
                        .output(*id)
                        .is_some_and(|output| output.channels != *channels)
                })
                .map(|channels| edit_output(*id, OutputEdit::Channels(channels))),
            _ => None,
        };
        Self {
            typed: self
                .typed
                .filter(|(setting, _)| !matches!(setting, Setting::Channels(_))),
            ..self
        }
        .send_all(command)
    }

    /// Settles the text of the field that the operator was in, unless it is
    /// the field of `setting`
    pub(super) fn settled_but(self, setting: Setting) -> Self {
        match &self.typed {
            Some((typed, _)) if *typed != setting => self.leaving_unapplied().settled(),
            _ => self,
        }
    }

    /// Returns why the text that the operator typed into the field and did
    /// not take back does not apply, as the status bar tells it
    pub(super) fn unapplied(&self) -> Option<String> {
        let (setting, text) = self.typed.as_ref()?;
        view::rejection(self, *setting, text).map(|reason| format!("Not applied: {reason}"))
    }

    /// Keeps the reason that the text of the field about to be left does not
    /// apply, since the field shows the value that holds once it is left
    pub(super) fn leaving_unapplied(self) -> Self {
        Self {
            unapplied: self.unapplied().or(self.unapplied.clone()),
            ..self
        }
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
                // Start from the sends as the cue holds them, which keep the
                // sends to outputs that have gone for an undo to bring back
                let cue = self.workspace.cue(cue)?;
                let sends = cue
                    .sends
                    .clone()
                    .unwrap_or_else(|| self.workspace.sends(cue))
                    .into_iter()
                    .filter(|(id, _)| *id != output)
                    .chain(level.map(|level| (output, level)))
                    .collect();
                Some(Command::Edit(cue.id, Edit::Sends(Some(sends))))
            }),
            Setting::OutputName(id) => Some(edit_output(id, OutputEdit::Name(text.clone()))),
            Setting::Channels(_) => None,
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
