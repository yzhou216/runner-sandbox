// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes that the operator makes to one setting of a cue

#[cfg(test)]
mod tests;

use std::mem;

use super::{Advance, Cue, CueId, Seconds};
use crate::back::InstrumentId;

#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Number(String),
    Name(String),
    File(String),
    /// File of an audio, video, or picture cue together with the name it
    /// gives the cue, as choosing the file makes
    FileAndName {
        file: String,
        name: String,
    },
    Start(Seconds),
    End(Seconds),
    Loop(bool),
    Target(Option<CueId>),
    Notes(String),
    PreWait(Seconds),
    Duration(Seconds),
    PostWait(Seconds),
    Advance(Advance),
    /// Level in dB
    Level(f32),
    StopTarget(bool),
    /// Level in percent that a light cue fades an instrument to, or none
    /// to leave the instrument out
    Light(InstrumentId, Option<f32>),
    Destination(String),
    Message(String),
}

impl Edit {
    /// Returns whether `next` edits the same setting, which for the levels
    /// of a light cue is the level of the same instrument
    pub(in crate::back) fn same_setting(&self, next: &Edit) -> bool {
        match (self, next) {
            (Edit::Light(id, _), Edit::Light(next, _)) => id == next,
            _ => mem::discriminant(self) == mem::discriminant(next),
        }
    }
}

impl Cue {
    /// Returns the cue with one edit applied.  Its media stays as it was,
    /// since only the workspace, which knows where relative file names
    /// start from, can tell whether a new file name leads to another file.
    pub(super) fn edited(self, edit: Edit) -> Self {
        match edit {
            Edit::Number(number) => Self { number, ..self },
            Edit::Name(name) => Self { name, ..self },
            Edit::File(file) => Self { file, ..self },
            Edit::FileAndName { file, name } => Self { file, name, ..self },
            Edit::Start(start) => Self { start, ..self },
            Edit::End(end) => Self { end, ..self },
            Edit::Loop(looping) => Self { looping, ..self },
            Edit::Target(target) => Self { target, ..self },
            Edit::Notes(notes) => Self { notes, ..self },
            Edit::PreWait(pre_wait) => Self { pre_wait, ..self },
            Edit::Duration(duration) => Self { duration, ..self },
            Edit::PostWait(post_wait) => Self { post_wait, ..self },
            Edit::Advance(advance) => Self { advance, ..self },
            Edit::Level(level) => Self { level, ..self },
            Edit::StopTarget(stop_target) => Self {
                stop_target,
                ..self
            },
            Edit::Light(id, level) => Self {
                lights: self
                    .lights
                    .into_iter()
                    .filter(|(lit, _)| *lit != id)
                    .chain(level.map(|level| (id, level)))
                    .collect(),
                ..self
            },
            Edit::Destination(destination) => Self {
                destination,
                ..self
            },
            Edit::Message(message) => Self { message, ..self },
        }
    }

    /// Returns the edit that sets what `edit` changes back to its value in
    /// this cue, which is how undo takes an edit back
    pub(in crate::back) fn reverting(&self, edit: &Edit) -> Edit {
        match edit {
            Edit::Number(_) => Edit::Number(self.number.clone()),
            Edit::Name(_) => Edit::Name(self.name.clone()),
            Edit::File(_) => Edit::File(self.file.clone()),
            Edit::FileAndName { .. } => Edit::FileAndName {
                file: self.file.clone(),
                name: self.name.clone(),
            },
            Edit::Start(_) => Edit::Start(self.start.clone()),
            Edit::End(_) => Edit::End(self.end.clone()),
            Edit::Loop(_) => Edit::Loop(self.looping),
            Edit::Target(_) => Edit::Target(self.target),
            Edit::Notes(_) => Edit::Notes(self.notes.clone()),
            Edit::PreWait(_) => Edit::PreWait(self.pre_wait.clone()),
            Edit::Duration(_) => Edit::Duration(self.duration.clone()),
            Edit::PostWait(_) => Edit::PostWait(self.post_wait.clone()),
            Edit::Advance(_) => Edit::Advance(self.advance),
            Edit::Level(_) => Edit::Level(self.level),
            Edit::StopTarget(_) => Edit::StopTarget(self.stop_target),
            Edit::Light(id, _) => Edit::Light(*id, self.lights.get(id).copied()),
            Edit::Destination(_) => Edit::Destination(self.destination.clone()),
            Edit::Message(_) => Edit::Message(self.message.clone()),
        }
    }
}
