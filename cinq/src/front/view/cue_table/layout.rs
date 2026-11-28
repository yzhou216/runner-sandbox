// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! How the cues of a list lie in groups and which rows show, worked out once
//! for a view

use std::collections::HashMap;

use super::INDENT;
use crate::back::{Cue, CueId, CueList, Groups, Kind};

pub(super) struct Layout<'a> {
    pub(super) groups: Groups<'a>,
    depths: HashMap<CueId, u8>,
    shown: Vec<usize>,
    standing: Vec<usize>,
    next: Option<CueId>,
    gutter: usize,
}

impl<'a> Layout<'a> {
    pub(super) fn of(list: &'a CueList) -> Self {
        let groups = list.groups();
        let depths = list
            .cues
            .iter()
            .map(|cue| cue.id)
            .zip(groups.depths().iter().copied())
            .collect();
        let standing = groups.shown_as();
        let next = list
            .standby
            .and_then(|id| list.position(id))
            .and_then(|row| list.cues.get(standing[row]))
            .map(|cue| cue.id);
        // Only a list that holds a group needs the mark, so that the names of
        // the others lie where they always did
        let gutter = if list.cues.iter().any(|cue| cue.kind == Kind::Group) {
            INDENT
        } else {
            0
        };
        Self {
            shown: groups.shown(),
            groups,
            depths,
            standing,
            next,
            gutter,
        }
    }

    pub(super) fn rows(&self) -> Vec<&'a Cue> {
        let cues = &self.groups.list().cues;
        self.shown.iter().map(|&row| &cues[row]).collect()
    }

    /// Returns the position among the rows shown of the row that stands for
    /// the cue `id`, which is its own unless a collapsed group hides it
    pub(super) fn row_of(&self, id: CueId) -> Option<usize> {
        let row = self.groups.list().position(id)?;
        self.shown.binary_search(&self.standing[row]).ok()
    }

    /// Returns whether the row of `cue` stands for the standby cue, which GO
    /// fires next
    pub(super) fn is_next(&self, cue: &Cue) -> bool {
        self.next == Some(cue.id)
    }

    /// Returns the columns of the font that a name leaves free before it for
    /// the mark of a group, which are none in a list that holds no group
    pub(super) fn gutter(&self) -> usize {
        self.gutter
    }

    pub(super) fn depth(&self, cue: &Cue) -> u8 {
        self.depths.get(&cue.id).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests;
