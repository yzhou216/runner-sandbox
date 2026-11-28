// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a change to the show records, which is only what it touched

use std::mem;

use crate::back::{
    Cue, CueId, CueList, Edit, Instrument, Output, OutputEdit, OutputId, PatchEdit, Place,
};

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
    /// Cues pasted at a place, recorded as an addition is
    Pasted {
        cues: Vec<Placed>,
        at: Place,
        standby: Standby,
    },
    /// Cues deleted, with the places they had in order, the standby of each
    /// list they left, and whether they were cut to the clipboard
    Deleted {
        cues: Vec<Placed>,
        standby: Vec<Standby>,
        cut: bool,
    },
    /// Cue moved within its list, from one place to another
    Moved { cue: CueId, from: usize, to: usize },
    /// Cues moved into groups or out of them, or a cue moved with the cues
    /// that it holds where depths change, with the order and depths of the
    /// cues of their list before and after.  Where `moved` is set the change
    /// was a move up or down, which moves of one cue in a row make one change
    /// of, and `deeper` tells whether the cue came to lie deeper.
    Regrouped {
        list: usize,
        cues: Vec<CueId>,
        deeper: bool,
        moved: bool,
        before: Vec<(CueId, u8)>,
        after: Vec<(CueId, u8)>,
    },
    /// One setting of a cue edited, with the edit that takes it back and
    /// the edit that makes it again
    Edited { cue: CueId, undo: Edit, redo: Edit },
    /// Standby of a list moved by the operator
    Standby(Standby),
    /// Cue lists, outputs, and light patch kept from before put in place of
    /// those of the show, with those before and after
    Restored { before: Kept, after: Kept },
    /// Outputs added, deleted, moved, or edited, with all of them before
    /// and after.  The `setting` is the output and the setting that an edit
    /// changed, by which edits of one setting in a row make one change.  The
    /// `anchor` is the output that was selected when outputs were added,
    /// pasted, or moved, which undo selects again, as Vim's undo returns the
    /// cursor to where the change was made.  Where an output moved it is the
    /// output that moved, which two neighbors that swap do not tell by
    /// themselves.
    Outputs {
        before: Vec<Output>,
        after: Vec<Output>,
        setting: Option<(OutputId, mem::Discriminant<OutputEdit>)>,
        cut: bool,
        pasted: bool,
        anchor: Option<OutputId>,
    },
    /// Light patch edited, with the patch before and after
    Patched {
        edit: PatchEdit,
        before: Vec<Instrument>,
        after: Vec<Instrument>,
    },
}

/// Cue lists, outputs, and light patch of a show as they were or became
#[derive(Debug, Clone)]
pub struct Kept {
    pub lists: Vec<CueList>,
    pub outputs: Vec<Output>,
    pub patch: Vec<Instrument>,
}

#[derive(Debug, Clone)]
pub struct Placed {
    pub list: usize,
    pub index: usize,
    pub cue: Cue,
}

#[derive(Debug, Clone, Copy)]
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

    /// Returns the move that leaves standby where it was
    fn unmoved(self) -> Self {
        Self {
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
            Change::Regrouped { cues, .. } => cues.first().copied(),
            Change::Standby(_)
            | Change::Restored { .. }
            | Change::Outputs { .. }
            | Change::Patched { .. } => None,
        }
    }

    /// Returns the cue that standby is on once a move of standby is taken
    /// back, or made again where `redone` is set, which the view follows
    pub fn standing(&self, redone: bool) -> Option<CueId> {
        match self {
            Change::Standby(standby) if redone => standby.after,
            Change::Standby(standby) => standby.before,
            _ => None,
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

    /// Returns the change without the move of the standby of `list` that
    /// adding a cue made, so that redo puts no cue that has played back
    /// on standby
    pub(super) fn keeping_standby_of(self, list: usize) -> Self {
        match self {
            Self::Added { cues, at, standby } if standby.list == list => Self::Added {
                cues,
                at,
                standby: standby.unmoved(),
            },
            Self::Pasted { cues, at, standby } if standby.list == list => Self::Pasted {
                cues,
                at,
                standby: standby.unmoved(),
            },
            change => change,
        }
    }

    pub(super) fn moves_standby_of(&self, list: usize) -> bool {
        matches!(self, Change::Standby(standby) if standby.list == list)
    }

    /// Returns the depth that `arrangement` gives `cue`, where it holds it
    pub fn depth_in(arrangement: &[(CueId, u8)], cue: CueId) -> Option<u8> {
        arrangement
            .iter()
            .find_map(|&(id, depth)| (id == cue).then_some(depth))
    }
}
