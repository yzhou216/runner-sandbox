// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a cue lands when it is added, pasted, or moved: its index and its
//! depth
//!
//! Wherever a cue lands, it takes the depth that a cue added there would.
//! A cue below a row takes the deepest depth that the row allows, which is
//! one level deeper than the row where that is a group, so that a cue added
//! below a group is the first cue in it.  A cue above a row takes the depth
//! of that row.  A collapsed group is one row, as the cue list shows it, so
//! that a cue added below it lands below the cues that it holds, at its own
//! depth, outside it.

use super::CueList;
use crate::back::{Cue, Kind, Place};

impl CueList {
    /// Returns the index that a cue put at `at` takes
    pub(in crate::back) fn index_at(&self, at: Place) -> usize {
        match at {
            Place::After(cue) => self.position(cue).map(|row| self.index_below(row)),
            Place::Before(cue) => self.position(cue),
            Place::End => None,
        }
        .unwrap_or(self.cues.len())
    }

    /// Returns the depth that a cue put at `at` takes
    pub(in crate::back) fn depth_at(&self, at: Place) -> u8 {
        match at {
            Place::Before(cue) => self.position(cue).map(|row| self.depth_above(row)),
            Place::After(cue) => self.position(cue).map(|row| self.depth_below(row)),
            Place::End => None,
        }
        .unwrap_or_else(|| {
            self.groups()
                .shown_as()
                .last()
                .map_or(0, |&row| self.depth_below(row))
        })
    }

    /// Returns the index just below the row `row` and the cues that it hides
    fn index_below(&self, row: usize) -> usize {
        if self.collapsed_at(row) {
            self.extent(row).end
        } else {
            row + 1
        }
    }

    /// Returns whether the row `row` is a collapsed group
    fn collapsed_at(&self, row: usize) -> bool {
        self.cues
            .get(row)
            .is_some_and(|cue| cue.kind == Kind::Group && cue.collapsed)
    }

    /// Returns the depth of a cue put just above the row `row`
    fn depth_above(&self, row: usize) -> u8 {
        self.depths().get(row).copied().unwrap_or(0)
    }

    /// Returns the deepest depth of a cue put just below the row `row`, which
    /// is outside a group that is collapsed
    fn depth_below(&self, row: usize) -> u8 {
        self.depths().get(row).map_or(0, |depth| {
            let opens = self.cues[row].kind == Kind::Group && !self.collapsed_at(row);
            depth.saturating_add(u8::from(opens))
        })
    }

    /// Moves the cue at row `from`, together with the cues that it holds, by
    /// `by` rows of the cue list, downwards when positive, as far as its ends
    /// allow, leaving standby where it is.  Returns the list with the index
    /// that the block took among the cues without it.  The depths of the block
    /// shift together so that its first cue takes the depth of a cue added
    /// where the block lands: below the row above it when it moves down, and
    /// above the row below it when it moves up.  Moving the block onto a group
    /// therefore takes it into the group and moving it off the edge of one
    /// takes it out, while moving it past a collapsed group takes it below
    /// the cues that the group holds, outside it.
    pub(in crate::back) fn block_moved(&self, from: usize, by: isize) -> (Self, usize) {
        let depths = self.depths();
        let extent = self.extent(from);
        let lead = depths.get(from).copied().unwrap_or(0);
        let list = self.clone();
        let mut cues = list.cues;
        let block: Vec<(Cue, u8)> = cues
            .drain(extent.clone())
            .zip(depths[extent].iter().copied())
            .collect();
        let rest = Self { cues, ..list };
        let shown = rest.groups().shown();
        let slot = shown
            .partition_point(|&row| row < from)
            .saturating_add_signed(by)
            .min(shown.len());
        let to = shown.get(slot).copied().unwrap_or(rest.cues.len());
        let landing = if to > from {
            slot.checked_sub(1)
                .map_or(0, |above| rest.depth_below(shown[above]))
        } else {
            rest.depth_above(to)
        };
        let moved = block.into_iter().map(|(cue, depth)| Cue {
            depth: u8::try_from((i16::from(depth) - i16::from(lead) + i16::from(landing)).max(0))
                .unwrap_or(u8::MAX),
            ..cue
        });
        let mut cues = rest.cues;
        let tail = cues.split_off(to);
        cues.extend(moved);
        cues.extend(tail);
        (Self { cues, ..rest }, to)
    }
}
