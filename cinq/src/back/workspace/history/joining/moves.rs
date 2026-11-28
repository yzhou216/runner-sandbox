// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Joining moves of one cue in a row, within a list or with a new depth

use super::Change;
use crate::back::CueId;

impl Change {
    /// Returns this change followed by the move `next` as one change, where
    /// both are moves of one cue, and `next` alone otherwise
    pub(super) fn moved_on(self, next: Self) -> Self {
        match (self, next) {
            (Self::Moved { cue, from, .. }, Self::Moved { to, .. }) => {
                Self::Moved { cue, from, to }
            }
            (
                Self::Moved { cue, from, .. },
                Self::Regrouped {
                    list,
                    deeper,
                    moved: true,
                    before,
                    after,
                    ..
                },
            ) => Self::Regrouped {
                list,
                cues: vec![cue],
                deeper,
                moved: true,
                before: placed_at(before, cue, from),
                after,
            },
            (
                Self::Regrouped {
                    list,
                    cues,
                    deeper,
                    moved: true,
                    before,
                    after,
                },
                Self::Moved { cue, to, .. },
            ) => Self::Regrouped {
                list,
                cues,
                deeper,
                moved: true,
                before,
                after: placed_at(after, cue, to),
            },
            (
                Self::Regrouped {
                    list,
                    cues,
                    moved: true,
                    before,
                    ..
                },
                Self::Regrouped {
                    after, moved: true, ..
                },
            ) => Self::Regrouped {
                deeper: deeper_than(&before, &after, &cues),
                list,
                cues,
                moved: true,
                before,
                after,
            },
            (_, next) => next,
        }
    }
}

/// Returns whether the first of `cues` lies deeper in the arrangement `after`
/// than in `before`
fn deeper_than(before: &[(CueId, u8)], after: &[(CueId, u8)], cues: &[CueId]) -> bool {
    let depth = |arrangement: &[(CueId, u8)]| {
        cues.first()
            .and_then(|&cue| Change::depth_in(arrangement, cue))
    };
    depth(after) > depth(before)
}

/// Returns the arrangement with the entry of `cue` at the place `to`, as a
/// move of the cue within its list puts it
fn placed_at(arrangement: Vec<(CueId, u8)>, cue: CueId, to: usize) -> Vec<(CueId, u8)> {
    let (moving, rest): (Vec<_>, Vec<_>) = arrangement.into_iter().partition(|&(id, _)| id == cue);
    let (head, tail) = rest.split_at(to.min(rest.len()));
    head.iter().chain(&moving).chain(tail).copied().collect()
}
