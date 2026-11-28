// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of the run of a group: which cues it holds, which runs start
//! which, and when it ends

use crate::back::{CueId, Groups, Workspace};

impl Workspace {
    /// Returns a cue together with the cues that it holds, at any depth,
    /// which are none for a cue that is no group
    pub(super) fn with_members(&self, id: CueId) -> Vec<CueId> {
        self.list_holding(id)
            .map_or_else(|| vec![id], |list| list.with_members(&[id]))
    }

    /// Drops the runs of groups whose actions have begun and whose cues
    /// have no run left.  Ending a group stops nothing.  A group that holds
    /// groups ends once theirs have, so the search goes on until it finds no
    /// more.
    pub(super) fn without_ended_groups(self) -> Self {
        let ended = self.active_where(|active| {
            active.holds()
                && active.started
                && self
                    .with_members(active.id)
                    .iter()
                    .skip(1)
                    .all(|&member| self.active_of(member).is_none())
        });
        if ended.is_empty() {
            self
        } else {
            self.without_active_where(|active| ended.contains(&active.id))
                .without_ended_groups()
        }
    }
}

/// Returns the run that starts each run of a chain, given in the order of
/// the chain.  A cue that a group holds directly is started by the run of
/// that group, and any other cue by the run of the cue before it that no
/// group holds, which for the cue after a group is the group.
pub(super) fn started_by(groups: &Groups, chain: &[(CueId, u32)]) -> Vec<Option<u32>> {
    chain
        .iter()
        .scan(None, |before: &mut Option<u32>, &(id, run)| {
            let group = groups.group_of(id).and_then(|group| {
                chain
                    .iter()
                    .find(|&&(held, _)| held == group.id)
                    .map(|&(_, run)| run)
            });
            let after = group.or(*before);
            *before = group.is_none().then_some(run).or(*before);
            Some(after)
        })
        .collect()
}
