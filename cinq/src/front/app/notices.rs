// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the latest action told, which the status bar shows until the
//! operator does something else

use crate::{
    back::{CueId, one_line},
    front::{keymap::Action, update::Clipped},
};

#[derive(Default)]
pub(in crate::front) struct Notices {
    /// What the latest opening or saving told, which the status bar shows
    /// until the show changes
    pub(in crate::front) told: Option<String>,
    /// What the latest copy or cut told, until the operator does anything
    /// but move the selection or switch lists
    pub(in crate::front) clipped: Option<Clipped>,
    /// What the latest key found to say that is no change, such as an empty
    /// clipboard, until the operator does anything else
    pub(in crate::front) remark: Option<&'static str>,
    /// Why the text that the operator left in a field was not applied, which
    /// the field forgets once the keyboard has left it, until the operator
    /// does anything else
    pub(in crate::front) unapplied: Option<String>,
    /// Cues that the latest edit stopped by taking them away or giving them
    /// another file, which the notice of the change tells
    pub(in crate::front) stopped: Vec<CueId>,
    /// Changes that the latest undo or redo took back or made again, which
    /// its notice counts where there are several
    pub(in crate::front) turns: usize,
}

impl Notices {
    /// Returns the notices with `told` as what the latest opening or saving
    /// told, on one line, since the name of a file in it can hold a line break
    pub(in crate::front) fn with_told(self, told: Option<String>) -> Self {
        Self {
            told: told.as_deref().map(one_line),
            ..self
        }
    }

    pub(in crate::front) fn with_clipped(self, clipped: Option<Clipped>) -> Self {
        Self { clipped, ..self }
    }

    pub(in crate::front) fn with_remark(self, remark: Option<&'static str>) -> Self {
        Self { remark, ..self }
    }

    /// Returns the notices with `remark` in place of the remark, unless it
    /// is none, which leaves the remark as it was
    pub(in crate::front) fn preferring_remark(self, remark: Option<&'static str>) -> Self {
        Self {
            remark: remark.or(self.remark),
            ..self
        }
    }

    /// Returns the notices with `unapplied` in place of the reason that the
    /// text of a field was not applied, unless it is none, which leaves the
    /// reason as it was
    pub(in crate::front) fn preferring_unapplied(self, unapplied: Option<String>) -> Self {
        Self {
            unapplied: unapplied.or(self.unapplied),
            ..self
        }
    }

    pub(in crate::front) fn with_turns(self, turns: usize) -> Self {
        Self { turns, ..self }
    }

    /// Returns the notices that outlast a key that completed `action`, if
    /// any, which drops the remark and the unapplied text, and the notice of
    /// a copy or cut too, unless the action only moves
    pub(in crate::front) fn after_key(self, action: Option<Action>) -> Self {
        Self {
            clipped: self
                .clipped
                .filter(|_| action.is_none_or(Action::only_moves)),
            remark: self.remark.filter(|_| action.is_none()),
            unapplied: self.unapplied.filter(|_| action.is_none()),
            ..self
        }
    }

    /// Returns the notices that outlast the latest message, which keep the
    /// copy or cut, the remark, and the unapplied text only while `typing`
    pub(in crate::front) fn while_typing(self, typing: bool) -> Self {
        Self {
            clipped: self.clipped.filter(|_| typing),
            remark: self.remark.filter(|_| typing),
            unapplied: self.unapplied.filter(|_| typing),
            ..self
        }
    }
}
