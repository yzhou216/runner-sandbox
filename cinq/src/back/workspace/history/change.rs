// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a change to the show records, which is only what it touched

use std::mem;

use crate::back::{Cue, CueId, CueList, Edit, Output, OutputEdit, OutputId, Place};

#[derive(Debug, Clone)]
pub enum Change {
    /// Cues added at a place, with the places they took in order, and the
    /// standby of their list before and after, which the first of them can
    /// take.  The place is where the first of them went.
    Added {
        cues: Vec<Placed>,
        at: Place,
        standby: Standby,
    },
    /// Cues pasted at a place, with the places they took in order, and the
    /// standby of their list before and after, which the first of them can
    /// take
    Pasted {
        cues: Vec<Placed>,
        at: Place,
        standby: Standby,
    },
    /// Cues deleted, with the places they had in order, and the standby of
    /// each list they left, and whether they were cut to the clipboard
    Deleted {
        cues: Vec<Placed>,
        standby: Vec<Standby>,
        cut: bool,
    },
    /// Cue moved within its list, from one place to another
    Moved { cue: CueId, from: usize, to: usize },
    /// One setting of a cue edited, with the edit that takes it back and
    /// the edit that makes it again
    Edited { cue: CueId, undo: Edit, redo: Edit },
    /// Standby of a list moved by the operator
    Standby(Standby),
    /// Cue lists and outputs kept from before put in place of those of the
    /// show, with those before and after
    Restored { before: Kept, after: Kept },
    /// Outputs added, deleted, moved, or edited, with all of them before
    /// and after, whether they were cut to the register of outputs, and
    /// the output and the setting that an edit changed, by which edits of
    /// one setting in a row make one change
    Outputs {
        before: Vec<Output>,
        after: Vec<Output>,
        setting: Option<(OutputId, mem::Discriminant<OutputEdit>)>,
        cut: bool,
    },
}

/// Cue lists and outputs of a show as they were or became
#[derive(Debug, Clone)]
pub struct Kept {
    pub lists: Vec<CueList>,
    pub outputs: Vec<Output>,
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
            Change::Added { cues, .. }
            | Change::Pasted { cues, .. }
            | Change::Deleted { cues, .. } => cues.first().map(|placed| placed.cue.id),
            Change::Moved { cue, .. } | Change::Edited { cue, .. } => Some(*cue),
            Change::Standby(_) | Change::Restored { .. } | Change::Outputs { .. } => None,
        }
    }

    /// Returns the cue that an addition or a paste went next to, if it did,
    /// which undo returns the selection to, as Vim's undo returns the cursor
    /// to where the change was made, whether the cues went below it or above
    /// it, where the selection was on the last of them
    pub fn anchor(&self) -> Option<CueId> {
        match self {
            Change::Added {
                at: Place::After(cue) | Place::Before(cue),
                ..
            }
            | Change::Pasted {
                at: Place::After(cue) | Place::Before(cue),
                ..
            } => Some(*cue),
            _ => None,
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
            ) => cue == next && mem::discriminant(redo) == mem::discriminant(again),
            (
                Change::Outputs {
                    setting: Some(setting),
                    ..
                },
                Change::Outputs {
                    setting: Some(again),
                    ..
                },
            ) => setting == again,
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
            (
                Self::Outputs { before, .. },
                Self::Outputs {
                    after,
                    setting,
                    cut,
                    ..
                },
            ) => Self::Outputs {
                before,
                after,
                setting,
                cut,
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
            Self::Outputs { before, after, .. } => before == after,
            _ => false,
        }
    }

    /// Returns the change without the move of the standby of `list` that
    /// adding a cue made, so that redo puts no cue that has played back
    /// on standby
    pub(super) fn keeping_standby_of(self, list: usize) -> Self {
        match self {
            Self::Added { cues, at, standby } if standby.list == list => Self::Added {
                cues,
                at,
                standby: Standby {
                    after: standby.before,
                    ..standby
                },
            },
            Self::Pasted { cues, at, standby } if standby.list == list => Self::Pasted {
                cues,
                at,
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
