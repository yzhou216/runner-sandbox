// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a change to the show records, which is only what it touched

use crate::back::{Contents, Cue, CueId, Edit, Instrument, PatchEdit};

#[derive(Debug, Clone)]
pub enum Change {
    /// Cue added, with the standby of its list before and after, which the
    /// new cue can take.  The cue is boxed, since it is far larger than
    /// what the other changes record.
    Added {
        placed: Box<Placed>,
        standby: Standby,
    },
    /// Cues deleted, with the places they had in order, and the standby of
    /// each list they left
    Deleted {
        cues: Vec<Placed>,
        standby: Vec<Standby>,
    },
    /// Cue moved within its list, from one place to another
    Moved { cue: CueId, from: usize, to: usize },
    /// One setting of a cue edited, with the edit that takes it back and
    /// the edit that makes it again
    Edited { cue: CueId, undo: Edit, redo: Edit },
    /// Standby of a list moved by the operator
    Standby(Standby),
    /// Light patch edited, with the patch before and after
    Patched {
        edit: PatchEdit,
        before: Vec<Instrument>,
        after: Vec<Instrument>,
    },
    /// Cue lists and patch kept from before put in place of those of the
    /// show, with both before and after
    Restored { before: Contents, after: Contents },
}

#[derive(Debug, Clone)]
pub struct Placed {
    pub list: usize,
    pub index: usize,
    pub cue: Cue,
}

#[derive(Debug, Clone)]
pub struct Standby {
    pub list: usize,
    pub before: Option<CueId>,
    pub after: Option<CueId>,
}

impl Standby {
    pub(super) fn reversed(self) -> Self {
        Self {
            before: self.after,
            after: self.before,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Done,
    Undone,
    Redone,
}

impl Change {
    pub fn cue(&self) -> Option<CueId> {
        match self {
            Change::Added { placed, .. } => Some(placed.cue.id),
            Change::Deleted { cues, .. } => cues.first().map(|placed| placed.cue.id),
            Change::Moved { cue, .. } | Change::Edited { cue, .. } => Some(*cue),
            Change::Standby(_) | Change::Patched { .. } | Change::Restored { .. } => None,
        }
    }

    /// Returns whether `next` edits the setting that this change edited, or
    /// moves the cue that it moved, so that the two make one change
    pub(super) fn takes(&self, next: &Change) -> bool {
        match (self, next) {
            (Change::Moved { cue, .. }, Change::Moved { cue: next, .. }) => cue == next,
            (
                Change::Edited { cue, redo, .. },
                Change::Edited {
                    cue: next,
                    redo: again,
                    ..
                },
            ) => cue == next && redo.same_setting(again),
            (Change::Patched { edit, .. }, Change::Patched { edit: next, .. }) => {
                edit.same_setting(next)
            }
            _ => false,
        }
    }

    /// Returns this change followed by `next` as one change, which takes
    /// back to before this one and makes again what `next` made
    pub(super) fn then(self, next: Self) -> Self {
        match (self, next) {
            (Self::Edited { cue, undo, .. }, Self::Edited { redo, .. }) => {
                Self::Edited { cue, undo, redo }
            }
            (Self::Moved { cue, from, .. }, Self::Moved { to, .. }) => {
                Self::Moved { cue, from, to }
            }
            (Self::Patched { before, .. }, Self::Patched { edit, after, .. }) => Self::Patched {
                edit,
                before,
                after,
            },
            (_, next) => next,
        }
    }

    /// Returns whether the change leaves the show as it found it, as edits
    /// or moves in a row that end where they began do
    pub(super) fn changes_nothing(&self) -> bool {
        match self {
            Self::Edited { undo, redo, .. } => undo == redo,
            Self::Moved { from, to, .. } => from == to,
            Self::Patched { before, after, .. } => before == after,
            _ => false,
        }
    }

    /// Returns the change without the move of the standby of `list` that
    /// adding a cue made, so that redo puts no cue that has played back
    /// on standby
    pub(super) fn keeping_standby_of(self, list: usize) -> Self {
        match self {
            Self::Added { placed, standby } if standby.list == list => Self::Added {
                placed,
                standby: Standby {
                    after: standby.before,
                    ..standby
                },
            },
            change => change,
        }
    }

    pub(super) fn moves_standby_of(&self, list: usize) -> bool {
        matches!(self, Change::Standby(standby) if standby.list == list)
    }
}
