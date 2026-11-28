// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Chain that one GO fires through groups, and how long a group's
//! action takes

use super::Groups;
use crate::back::{Cue, CueId, Kind};

impl<'a> Groups<'a> {
    /// Returns the cues one GO on `start` fires, each paired with its delay
    /// after the GO.  The chain follows auto-continue and auto-follow until
    /// it reaches a cue that waits for GO.  A group fires the cues that it
    /// holds along with itself, each once the pre-waits of the groups that
    /// hold it are over, and the chain goes on from the row below the group,
    /// as its own setting says.  A cue in a group goes on to no cue, since
    /// its group fires the cues beside it.
    pub fn chain(&self, start: CueId) -> Vec<(&'a Cue, f32)> {
        let from = self.list.position(start).unwrap_or(self.list.cues.len());
        let inside = self.depths.get(from).is_some_and(|&depth| depth > 0);
        std::iter::successors(Some((from, 0.0_f32)), |&(index, delay)| {
            let cue = self.list.cues.get(index)?;
            cue.next_delay_in(self)
                .filter(|_| !inside)
                .map(|gap| (self.extent(index).end, delay + gap))
        })
        .take_while(|&(index, _)| index < self.list.cues.len())
        .flat_map(|(index, delay)| self.fired_with(index, delay))
        .collect()
    }

    /// Returns the cue at `index` with the cues that it holds, each with its
    /// delay after a GO that fires the cue `delay` seconds after it
    fn fired_with(&self, index: usize, delay: f32) -> Vec<(&'a Cue, f32)> {
        // Groups that hold the row, each with its depth and its pre-wait,
        // which holds the row back as long as it lasts
        self.extent(index)
            .scan(Vec::<(u8, f32)>::new(), |groups, row| {
                let depth = self.depths[row];
                groups.retain(|&(above, _)| above < depth);
                let held: f32 = groups.iter().map(|(_, wait)| wait).sum();
                let cue = &self.list.cues[row];
                if cue.kind == Kind::Group {
                    groups.push((depth, cue.pre_wait.value()));
                }
                Some((cue, delay + held))
            })
            .collect()
    }

    /// Returns how long the action of a cue takes, which for a group is
    /// until the last of the actions of the cues that it holds directly
    /// ends, each counted from the end of the pre-wait of the group, and
    /// none for a group that holds no cue
    pub fn action_of(&self, cue: &Cue) -> f32 {
        if cue.kind != Kind::Group {
            return cue.action();
        }
        self.held(cue)
            .map(|held| held.pre_wait.value() + self.action_of(held))
            .fold(0.0, f32::max)
    }

    /// Returns whether the action of a cue has no end, as that of a group
    /// has once a cue in it has none
    pub fn endless(&self, cue: &Cue) -> bool {
        if cue.kind == Kind::Group {
            self.held(cue).any(|held| self.endless(held))
        } else {
            cue.endless()
        }
    }

    /// Returns the cues that a group holds directly
    fn held(&self, group: &Cue) -> impl Iterator<Item = &'a Cue> {
        let (rows, depth) = self.list.position(group.id).map_or((0..0, 0), |index| {
            (self.extent(index), self.depths[index].saturating_add(1))
        });
        rows.skip(1)
            .filter(move |&row| self.depths[row] == depth)
            .map(|row| &self.list.cues[row])
    }
}
