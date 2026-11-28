// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of moving cues, which take a group with the cues that it holds, and of
//! moving cues into groups and out of them, each a change that undo takes back
//! at once

use std::collections::HashMap;

use itertools::Itertools;

use super::super::{Workspace, history::Change};
use crate::back::{Cue, CueId, CueList, Kind};

impl Workspace {
    /// Moves each of `ids` into the group above it, together with the cues
    /// that it holds, as one change.  A cue goes one level deeper where the
    /// row above it is a group or lies deeper than the cue, and stays where
    /// it is elsewhere, as at the top of a list.
    pub(in crate::back::workspace) fn indented(self, ids: &[CueId]) -> Self {
        self.regrouped(ids, true)
    }

    /// Moves each of `ids` out of the group that holds it, together with
    /// the cues that it holds, as one change.  A cue with cues of its group
    /// below it goes below them, so that the group keeps them, and comes
    /// next after the group.
    pub(in crate::back::workspace) fn outdented(self, ids: &[CueId]) -> Self {
        self.regrouped(ids, false)
    }

    fn regrouped(self, ids: &[CueId], deeper: bool) -> Self {
        let Some(list) = ids.first().and_then(|&id| self.list_of(id)) else {
            return self;
        };
        let before = self.lists[list].arrangement();
        let workspace = self.with_list(list, |cues| {
            if deeper {
                cues.indented(ids)
            } else {
                cues.outdented(ids)
            }
        });
        let after = workspace.lists[list].arrangement();
        if before == after {
            return workspace;
        }
        workspace.recorded(Change::Regrouped {
            list,
            cues: ids.to_vec(),
            deeper,
            moved: false,
            before,
            after,
        })
    }

    /// Moves a cue within its list by a number of rows of the cue list,
    /// downwards when positive, as far as the ends of the list allow, where a
    /// collapsed group is one row.  A group moves with the cues that it
    /// holds, as one block, whose depths shift so that its first cue takes
    /// the depth of a cue added where the block lands.  A single cue that
    /// keeps its depth is a move of its own, and any other is recorded with
    /// the depths of the list before and after it.
    pub(in crate::back::workspace) fn moved(self, id: CueId, by: isize) -> Self {
        let Some(list) = self.list_of(id) else {
            return self;
        };
        let cues = &self.lists[list];
        let Some(from) = cues.position(id) else {
            return self;
        };
        let rows = cues.extent(from).len();
        let (moved, to) = cues.block_moved(from, by);
        if to == from {
            return self;
        }
        let (before, depth) = (cues.arrangement(), cues.cues[from].depth);
        let workspace = self.with_list(list, |_| moved);
        let after = workspace.lists[list].arrangement();
        let now = workspace.cue(id).map_or(depth, |cue| cue.depth);
        workspace.recorded(if rows == 1 && now == depth {
            Change::Moved { cue: id, from, to }
        } else {
            Change::Regrouped {
                list,
                cues: vec![id],
                deeper: now > depth,
                moved: true,
                before,
                after,
            }
        })
    }

    /// Collapses the groups among `ids`, or expands them, without recording
    /// it, since a collapsed group is a way to look at the list
    pub(in crate::back::workspace) fn collapsed(self, ids: &[CueId], collapsed: bool) -> Self {
        let lists = self
            .lists
            .into_iter()
            .map(|list| CueList {
                cues: list
                    .cues
                    .into_iter()
                    .map(|cue| match cue.kind {
                        Kind::Group if ids.contains(&cue.id) => Cue { collapsed, ..cue },
                        _ => cue,
                    })
                    .collect(),
                ..list
            })
            .collect();
        Self { lists, ..self }
    }

    /// Puts the cues of a list in the order and at the depths of
    /// `arrangement` without recording it, as undo and redo do
    pub(in crate::back::workspace) fn with_arrangement(
        self,
        list: usize,
        arrangement: &[(CueId, u8)],
    ) -> Self {
        self.with_list(list, |cues| cues.arranged(arrangement))
    }
}

impl CueList {
    /// Returns the order of the cues, each with the depth that it keeps
    pub(in crate::back::workspace) fn arrangement(&self) -> Vec<(CueId, u8)> {
        self.cues.iter().map(|cue| (cue.id, cue.depth)).collect()
    }

    /// Returns the list with its cues in the order and at the depths of
    /// `arrangement`, and those that it leaves out after them
    fn arranged(self, arrangement: &[(CueId, u8)]) -> Self {
        let places: HashMap<CueId, (usize, u8)> = arrangement
            .iter()
            .enumerate()
            .map(|(place, &(id, depth))| (id, (place, depth)))
            .collect();
        let cues = self
            .cues
            .into_iter()
            .sorted_by_key(|cue| places.get(&cue.id).map_or(usize::MAX, |&(place, _)| place))
            .map(|cue| match places.get(&cue.id) {
                Some(&(_, depth)) => Cue { depth, ..cue },
                None => cue,
            })
            .collect();
        Self { cues, ..self }
    }

    /// Returns the rows of the cues of `ids` that no other of them holds,
    /// in order
    fn roots(&self, ids: &[CueId]) -> Vec<usize> {
        let rows: Vec<usize> = ids
            .iter()
            .filter_map(|&id| self.position(id))
            .sorted_unstable()
            .dedup()
            .collect();
        let groups = self.groups();
        rows.iter()
            .filter(|&&row| {
                !rows
                    .iter()
                    .any(|&above| above < row && groups.extent(above).contains(&row))
            })
            .copied()
            .collect()
    }

    fn indented(self, ids: &[CueId]) -> Self {
        let roots = self.roots(ids);
        roots.into_iter().fold(self, |list, row| {
            let depths = list.depths();
            let allowed = row.checked_sub(1).map_or(0, |above| {
                depths[above].saturating_add(u8::from(list.cues[above].kind == Kind::Group))
            });
            let extent = list.extent(row);
            // No depth is left for a block that already lies as deep as one can
            if depths[row] >= allowed || depths[extent.clone()].contains(&u8::MAX) {
                return list;
            }
            let cues = list
                .cues
                .into_iter()
                .enumerate()
                .map(|(index, cue)| {
                    if extent.contains(&index) {
                        Cue {
                            depth: depths[index] + 1,
                            ..cue
                        }
                    } else {
                        cue
                    }
                })
                .collect();
            Self { cues, ..list }
        })
    }

    fn outdented(self, ids: &[CueId]) -> Self {
        let roots: Vec<CueId> = self
            .roots(ids)
            .into_iter()
            .map(|row| self.cues[row].id)
            .collect();
        roots.into_iter().rev().fold(self, |list, id| {
            let (Some(row), Some(group)) = (list.position(id), list.group_of(id).map(|g| g.id))
            else {
                return list;
            };
            let depths = list.depths();
            let extent = list.extent(row);
            let mut cues = list.cues;
            let taken: Vec<Cue> = cues
                .drain(extent.clone())
                .zip(&depths[extent])
                .map(|(cue, &depth)| Cue {
                    depth: depth - 1,
                    ..cue
                })
                .collect();
            let rest = Self { cues, ..list };
            let end = rest
                .position(group)
                .map_or(rest.cues.len(), |at| rest.extent(at).end);
            let mut cues = rest.cues;
            cues.splice(end..end, taken);
            Self { cues, ..rest }
        })
    }
}
