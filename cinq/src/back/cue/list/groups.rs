// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Group cues, which hold the cues below them that lie one level deeper,
//! and the chain of cues that one GO fires through them
//!
//! A list keeps its cues in one row each, in order, and each cue keeps how
//! deep it lies in groups.  A cue lies in the nearest group cue above it
//! that is one level less deep, so that a group holds the rows below it up
//! to the first that lies no deeper than the group.  The depth that a cue
//! keeps counts only as far as the rows above it allow: no deeper than the
//! row above, or one level deeper where that row is a group.  So a cue
//! moved, pasted, or left behind by a group that goes keeps its depth, and
//! takes it up again once a group is above it again, as undo brings one
//! back.
//!
//! Working out the depths takes a walk of the whole list, so [`Groups`]
//! does it once and answers every question about the structure from it.
//! The methods of the list that ask one question build one for it.

mod collapsing;
mod firing;

use std::{collections::BTreeSet, ops::Range};

use super::CueList;
use crate::back::{Cue, CueId, Kind};

/// List with the depths of its cues worked out, to ask what groups hold
#[derive(Debug, Clone)]
pub struct Groups<'a> {
    list: &'a CueList,
    depths: Vec<u8>,
}

impl CueList {
    /// Works out how the cues of the list lie in groups, to ask more than
    /// one question of it
    pub fn groups(&self) -> Groups<'_> {
        Groups {
            list: self,
            depths: self.depths(),
        }
    }

    /// Returns how deep each cue lies in groups, in order, as far as the
    /// rows above it allow
    pub fn depths(&self) -> Vec<u8> {
        Self::depths_within(&self.cues)
    }

    /// Returns how deep each of `cues` lies in groups as a list of its own
    /// would hold them, which takes the first as the top
    pub(in crate::back) fn depths_within(cues: &[Cue]) -> Vec<u8> {
        cues.iter()
            .scan(None, |above: &mut Option<(u8, bool)>, cue| {
                let allowed = above.map_or(0, |(depth, group)| depth.saturating_add(group.into()));
                let depth = cue.depth.min(allowed);
                *above = Some((depth, cue.kind == Kind::Group));
                Some(depth)
            })
            .collect()
    }

    /// Returns the cues of `ids` together with the cues that each of them
    /// holds, each once, in the order of the list.  A copy of a group takes
    /// every cue that it holds, as a deletion does.  An identity that no
    /// cue of the list has is left out.
    pub fn with_members(&self, ids: &[CueId]) -> Vec<CueId> {
        self.groups().with_members(ids)
    }

    /// Returns the rows that the cue at `index` takes together with the cues
    /// that it holds, which is the row alone for a cue that is no group
    pub fn extent(&self, index: usize) -> Range<usize> {
        self.groups().extent(index)
    }

    /// Returns the group that holds a cue directly, if one does
    pub fn group_of(&self, id: CueId) -> Option<&Cue> {
        self.groups().group_of(id)
    }

    /// Returns the cues one GO on `start` fires, each paired with its delay
    /// after the GO, as [`Groups::chain`] tells
    pub fn chain(&self, start: CueId) -> impl Iterator<Item = (&Cue, f32)> {
        self.groups().chain(start).into_iter()
    }
}

impl<'a> Groups<'a> {
    /// Returns the list that the groups are of
    pub fn list(&self) -> &'a CueList {
        self.list
    }

    /// Returns how deep each cue lies in groups, as [`CueList::depths`] tells
    pub fn depths(&self) -> &[u8] {
        &self.depths
    }

    /// Returns the cues of `ids` together with the cues that each of them
    /// holds, as [`CueList::with_members`] tells
    pub fn with_members(&self, ids: &[CueId]) -> Vec<CueId> {
        let rows: BTreeSet<usize> = ids
            .iter()
            .filter_map(|&id| self.list.position(id))
            .flat_map(|index| self.extent(index))
            .collect();
        rows.into_iter().map(|row| self.list.cues[row].id).collect()
    }

    /// Returns the rows that the cue at `index` takes, as [`CueList::extent`]
    /// tells
    pub fn extent(&self, index: usize) -> Range<usize> {
        let end = self.depths.get(index).map_or(index, |&depth| {
            self.depths[index + 1..]
                .iter()
                .position(|&below| below <= depth)
                .map_or(self.depths.len(), |offset| index + 1 + offset)
        });
        index..end
    }

    /// Returns the cues that a group holds, at any depth, in order, or none
    /// for a cue that is no group
    pub fn members(&self, id: CueId) -> &'a [Cue] {
        self.list
            .position(id)
            .map_or(&[], |index| &self.list.cues[self.extent(index)][1..])
    }

    /// Returns the group that holds a cue directly, if one does
    pub fn group_of(&self, id: CueId) -> Option<&'a Cue> {
        let index = self.list.position(id)?;
        let depth = self.depths[index].checked_sub(1)?;
        let above = self.depths[..index]
            .iter()
            .rposition(|&above| above == depth)?;
        self.list.cues.get(above)
    }

    /// Returns whether the group `group` holds the cue `id`, at any depth
    pub fn holds(&self, group: CueId, id: CueId) -> bool {
        self.members(group).iter().any(|member| member.id == id)
    }
}
