// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Universes that leave the patch, which receivers would otherwise hold at
//! their last levels for as long as they hold those of any source that has
//! gone quiet

use std::collections::BTreeMap;

use super::Universes;

/// Frames that send the levels as zeros before the stream says it has ended
const ZEROED: u8 = 3;

/// Packets that carry the terminated option, which E1.31 asks to be three
/// so that one lost packet does not leave the stream running
const TERMINATED: u8 = 3;

/// Frames that remain to each universe that has left the patch
#[derive(Debug, Default, PartialEq)]
pub(super) struct Leaving(BTreeMap<u16, u8>);

impl Leaving {
    /// Adds the universes that `before` holds and `after` lacks, and takes
    /// back those that `after` holds again
    pub(super) fn changed(self, before: &Universes, after: &Universes) -> Self {
        let left = before
            .keys()
            .filter(|universe| !after.contains_key(universe))
            .map(|&universe| (universe, ZEROED + TERMINATED));
        Self(
            self.0
                .into_iter()
                .filter(|(universe, _)| !after.contains_key(universe))
                .chain(left)
                .collect(),
        )
    }

    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns each universe to send as zeros in this frame, with whether
    /// its stream says that it has ended
    pub(super) fn frame(&self) -> impl Iterator<Item = (u16, bool)> {
        self.0
            .iter()
            .map(|(&universe, &frames)| (universe, frames <= TERMINATED))
    }

    /// Returns the frames that remain once this one has gone out
    pub(super) fn next(self) -> Self {
        Self(
            self.0
                .into_iter()
                .filter(|&(_, frames)| frames > 1)
                .map(|(universe, frames)| (universe, frames - 1))
                .collect(),
        )
    }
}
