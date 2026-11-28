// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Telling which cues an edit silenced, since taking a cue away or giving it
//! another file stops it without a prompt, and undo brings back the cue and
//! not its sound

use crate::back::{Command, CueId, Workspace};

/// What a command can stop, with the cues that were running before it
pub(in crate::front::update) struct Stopping {
    running: Vec<CueId>,
    edits: bool,
    undoes: bool,
}

impl Stopping {
    pub(in crate::front::update) fn of(command: &Command, workspace: &Workspace) -> Self {
        Self {
            running: workspace.active().iter().map(|active| active.id).collect(),
            edits: command.edits(),
            undoes: matches!(command, Command::Undo | Command::Redo),
        }
    }

    /// Returns the cues that the command stopped, where it was an edit that
    /// took them out of the show or gave them another file, or else `kept`,
    /// the cues of the edit before, which the end of a run of edits keeps,
    /// since the field that a typed file leaves ends its run right after the
    /// edit.  An undo, a redo, or an edit that stopped nothing tells nothing.
    pub(in crate::front::update) fn stopped(self, kept: &[CueId], after: &Workspace) -> Vec<CueId> {
        let gone: Vec<CueId> = self
            .running
            .into_iter()
            .filter(|&cue| after.active_of(cue).is_none())
            .collect();
        match (self.edits, self.undoes) {
            (true, _) if !gone.is_empty() => gone,
            (true, _) | (_, true) => Vec::new(),
            _ => kept.to_vec(),
        }
    }
}
