// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes in a row that make one change: edits of one setting of one cue,
//! and moves of one cue, whether or not they change its depth

mod moves;

use super::Change;
use crate::back::CueId;

impl Change {
    /// Returns whether `next` edits the setting that this change edited, or
    /// moves the cue that it moved, so that the two make one change
    pub(super) fn takes(&self, next: &Change) -> bool {
        match (self, next) {
            (last, next) if last.mover().is_some() => last.mover() == next.mover(),
            (
                Change::Edited { cue, redo, .. },
                Change::Edited {
                    cue: next,
                    redo: again,
                    ..
                },
            ) => cue == next && redo.same_setting(again),
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
            (Change::Patched { edit, .. }, Change::Patched { edit: next, .. }) => {
                edit.same_setting(next)
            }
            _ => false,
        }
    }

    /// Returns the cue that this change moved up or down alone, if it did
    fn mover(&self) -> Option<CueId> {
        match self {
            Self::Moved { cue, .. } => Some(*cue),
            Self::Regrouped {
                cues, moved: true, ..
            } => match cues.as_slice() {
                [cue] => Some(*cue),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns this change followed by `next` as one change, which takes
    /// back to before this one and makes again what `next` made
    pub(super) fn then(self, next: Self) -> Self {
        match (self, next) {
            (Self::Edited { cue, undo, .. }, Self::Edited { redo, .. }) => {
                Self::Edited { cue, undo, redo }
            }
            (
                Self::Outputs { before, .. },
                Self::Outputs {
                    after,
                    setting,
                    cut,
                    pasted,
                    anchor,
                    ..
                },
            ) => Self::Outputs {
                before,
                after,
                setting,
                cut,
                pasted,
                anchor,
            },
            (Self::Patched { before, .. }, Self::Patched { edit, after, .. }) => Self::Patched {
                edit,
                before,
                after,
            },
            (last, next) => last.moved_on(next),
        }
    }

    /// Returns whether the change leaves the show as it found it, as edits
    /// or moves in a row that end where they began do
    pub(super) fn changes_nothing(&self) -> bool {
        match self {
            Self::Edited { undo, redo, .. } => undo == redo,
            Self::Moved { from, to, .. } => from == to,
            Self::Regrouped { before, after, .. } => before == after,
            Self::Outputs { before, after, .. } => before == after,
            Self::Patched { before, after, .. } => before == after,
            _ => false,
        }
    }
}
