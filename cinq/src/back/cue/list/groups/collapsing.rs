// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rows that the cue list shows, which leave out the cues that a
//! collapsed group holds, as Vim shows a closed fold as one line
//!
//! A collapsed group still fires, stops, and pauses the cues that it holds
//! as an expanded one does.  Only the cue list hides them.

use super::Groups;
use crate::back::{CueId, Kind};

impl Groups<'_> {
    /// Returns the row that stands for each cue in the cue list, in order:
    /// its own, or that of the outermost collapsed group that holds it
    pub fn shown_as(&self) -> Vec<usize> {
        self.depths
            .iter()
            .zip(&self.list.cues)
            .enumerate()
            .scan(
                None,
                |hiding: &mut Option<(usize, u8)>, (row, (&depth, cue))| {
                    *hiding = hiding.filter(|&(_, group)| depth > group).or_else(|| {
                        (cue.kind == Kind::Group && cue.collapsed).then_some((row, depth))
                    });
                    Some(hiding.map_or(row, |(group, _)| group))
                },
            )
            .collect()
    }

    /// Returns the rows that the cue list shows, in order
    pub fn shown(&self) -> Vec<usize> {
        self.shown_as()
            .into_iter()
            .enumerate()
            .filter_map(|(row, standing)| (row == standing).then_some(row))
            .collect()
    }

    /// Returns the collapsed groups that hold the cue `id`, from the nearest
    /// out, which must expand for the cue list to show it
    pub fn hiding(&self, id: CueId) -> Vec<CueId> {
        std::iter::successors(self.group_of(id), |group| self.group_of(group.id))
            .filter(|group| group.collapsed)
            .map(|group| group.id)
            .collect()
    }
}
