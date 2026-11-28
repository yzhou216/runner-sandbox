// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes to the cue lists, which [`super::history`] records for undo

mod cue_edits;
mod grouping;
mod numbering;
mod pasting;

use std::collections::HashSet;

use itertools::Itertools;

use self::numbering::Numbers;
use super::{
    Workspace,
    history::{Change, Kept, Placed, Standby},
};
use crate::back::{CueId, CueList, Edit, Kind, Place};

impl Workspace {
    /// Puts `standby` on standby in a list without recording the move, as
    /// GO moves it
    pub(super) fn with_standby(self, list: usize, standby: Option<CueId>) -> Self {
        self.with_list(list, |cues| cues.with_standby(standby))
    }

    /// Moves standby as the operator asks, recording the move unless
    /// standby is there already
    pub(super) fn moved_standby(self, list: usize, cue: Option<CueId>) -> Self {
        let Some(before) = self.lists.get(list).map(|cues| cues.standby) else {
            return self;
        };
        let workspace = self.with_standby(list, cue);
        let moved = workspace.standby_since(list, before);
        if moved.before == moved.after {
            workspace
        } else {
            workspace.recorded(Change::Standby(moved))
        }
    }

    /// Adds cues with the next identities, each after the one before and
    /// numbered after the highest cue of any list, and records them as one
    /// change at the place of the first, with the standby of their list before
    /// and after, which the first of them can take.  Every cue takes the depth
    /// that a cue added at the place of the first takes, so that a row of
    /// groups lies side by side.  Once the identities have run out, no more
    /// cues are added.
    pub(super) fn added(self, list: usize, kind: Kind, at: Place, count: usize) -> Self {
        let ids = self.identities(count);
        let before = self.lists.get(list).and_then(|cues| cues.standby);
        let sends = self.new_sends().filter(|_| kind.plays_file());
        let numbers = Numbers::of(self.cues().map(|cue| cue.number.as_str()));
        let depth = self.lists.get(list).map_or(0, |cues| cues.depth_at(at));
        let (workspace, ..) =
            ids.iter()
                .fold((self, at, numbers), |(workspace, at, numbers), &id| {
                    let number = numbers.next();
                    let numbers = numbers.with_taken(&number);
                    let added = Self {
                        next_id: workspace.next_id + 1,
                        ..workspace
                            .with_list(list, |cues| cues.with_new_cue(id, number, kind, at, depth))
                    }
                    .changed(id, Edit::Sends(sends.clone()));
                    (added, Place::After(id), numbers)
                });
        let cues: Vec<Placed> = ids
            .iter()
            .filter_map(|&id| workspace.place_of(id))
            .collect();
        if cues.is_empty() {
            return workspace;
        }
        let standby = workspace.standby_since(list, before);
        workspace.recorded(Change::Added { cues, at, standby })
    }

    /// Returns the next `count` identities, as many of them as have not run
    /// out
    fn identities(&self, count: usize) -> Vec<CueId> {
        (self.next_id..)
            .take(count)
            .map_while(|id| u32::try_from(id).ok().map(CueId))
            .collect()
    }

    /// Returns the move of the standby of a list from `before` to where it
    /// is now
    fn standby_since(&self, list: usize, before: Option<CueId>) -> Standby {
        Standby {
            list,
            before,
            after: self.lists[list].standby,
        }
    }

    /// Deletes cues as one change, with the places they had and where
    /// standby was before and after, and whether they were cut.  A group
    /// goes with every cue that it holds, which undo brings back as well.
    pub(super) fn deleted(self, ids: &[CueId], cut: bool) -> Self {
        // Cues in the order of their places, which is the order undo puts
        // them back in
        let cues: Vec<Placed> = self
            .lists
            .iter()
            .flat_map(|list| list.with_members(ids))
            .unique()
            .filter_map(|id| self.place_of(id))
            .sorted_by_key(|placed| (placed.list, placed.index))
            .collect();
        if cues.is_empty() {
            return self;
        }
        // Standby of each list that loses a cue, before it does
        let before: Vec<(usize, Option<CueId>)> = cues
            .iter()
            .map(|placed| placed.list)
            .dedup()
            .map(|list| (list, self.lists[list].standby))
            .collect();
        let workspace = self.removed(cues.iter().map(|placed| placed.cue.id));
        let standby = before
            .into_iter()
            .map(|(list, before)| workspace.standby_since(list, before))
            .collect();
        workspace.recorded(Change::Deleted { cues, standby, cut })
    }

    /// Moves a cue to the place `to` in its list without recording the
    /// move, as undo and redo move it
    pub(super) fn with_cue_moved(self, id: CueId, to: usize) -> Self {
        self.with_list_of(id, |cues| cues.with_moved(id, to))
    }

    /// Puts `kept` in place of the lists, outputs, and patch of the show as
    /// one change
    pub(super) fn restored(self, kept: Kept) -> Self {
        let before = Kept {
            lists: self.lists.clone(),
            outputs: self.outputs.clone(),
            patch: self.patch.clone(),
        };
        self.with_kept(kept.clone()).recorded(Change::Restored {
            before,
            after: kept,
        })
    }

    /// Puts kept lists, outputs, and patch in place of those of the show
    /// without recording a change, as undo and redo do.  Kept lists without
    /// outputs leave the outputs as they are, since a show keeps one, while
    /// a kept show without instruments has an empty patch.
    pub(super) fn with_kept(self, kept: Kept) -> Self {
        let workspace = self.with_lists(kept.lists).with_patch(kept.patch);
        if kept.outputs.is_empty() {
            workspace
        } else {
            workspace.with_outputs(kept.outputs)
        }
    }

    /// Puts `lists` in place of the cue lists without recording a change,
    /// as undo and redo do.  The cues that have gone stop, and new cues get
    /// identities that none of the lists holds.
    fn with_lists(self, lists: Vec<CueList>) -> Self {
        let lists = super::listed(lists);
        let workspace = Self {
            next_id: super::next_id(&lists).max(self.next_id),
            lists,
            ..self
        };
        let gone = workspace.active_where(|active| workspace.cue(active.id).is_none());
        gone.into_iter().fold(workspace, Self::stopped)
    }

    /// Removes cues from their lists and stops those that are running,
    /// without recording a change, as undo and redo remove them
    pub(super) fn removed(self, ids: impl IntoIterator<Item = CueId>) -> Self {
        let ids: Vec<CueId> = ids.into_iter().collect();
        let gone: HashSet<CueId> = ids.iter().copied().collect();
        let workspace = Self {
            lists: self
                .lists
                .into_iter()
                .map(|list| list.without(&gone))
                .collect(),
            ..self
        };
        ids.into_iter().fold(workspace, Self::stopped)
    }

    fn place_of(&self, id: CueId) -> Option<Placed> {
        self.lists.iter().enumerate().find_map(|(list, cues)| {
            cues.position(id).map(|index| Placed {
                list,
                index,
                cue: cues.cues[index].clone(),
            })
        })
    }
}
