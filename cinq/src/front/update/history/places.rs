// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The history of places as a model, which knows nothing of the interface
//! but whether a cue, a cue list, or an output is still there

#[cfg(test)]
mod tests;

use crate::back::{CueId, OutputId, Workspace};

/// Longest history, as in Vim
const LIMIT: usize = 100;

/// A place in the show.  A cue names its place by its identity, so that the
/// place follows the cue as it moves between cue lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spot {
    Cue(CueId),
    /// Cue list with no selected cue, which only its index names
    List(usize),
    /// Output that the outputs view shows selected, which follows it as the
    /// outputs move
    Output(OutputId),
    /// Outputs view with no selected output
    Outputs,
}

/// Places that jumps left, the latest last, and the places that going back
/// passed, the nearest last
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::front) struct History {
    behind: Vec<Spot>,
    ahead: Vec<Spot>,
}

impl History {
    /// Remembers `from`, the place that a jump left, once and as the latest.
    /// The places ahead, which going back passed, move behind it, the
    /// nearest to `from` first, as the list of Vim does when a jump follows
    /// going back (`:help jumplist`), so that going back again still visits
    /// them.
    pub(super) fn jumped(self, from: Spot) -> Self {
        let places = self.behind.into_iter().chain(self.ahead.into_iter().rev());
        Self {
            behind: remembered(places.collect(), from),
            ahead: Vec::new(),
        }
        .capped()
    }

    /// Goes `count` places back from `here`, or to the oldest place when
    /// there are fewer, and returns it with the history that remembers
    /// `here`, so that going forward returns to it.  Places that are gone
    /// or equal `here` are skipped and dropped.
    pub(super) fn back(
        &self,
        here: Spot,
        count: usize,
        alive: impl Fn(Spot) -> bool,
    ) -> Option<(Spot, Self)> {
        let (spot, behind, ahead) = shifted(&self.behind, &self.ahead, here, count, alive)?;
        Some((spot, Self { behind, ahead }.capped()))
    }

    /// Goes `count` places forward from `here`, as [`History::back`] goes
    /// back
    pub(super) fn forward(
        &self,
        here: Spot,
        count: usize,
        alive: impl Fn(Spot) -> bool,
    ) -> Option<(Spot, Self)> {
        let (spot, ahead, behind) = shifted(&self.ahead, &self.behind, here, count, alive)?;
        Some((spot, Self { behind, ahead }.capped()))
    }

    /// Remembers `here` as a place to go forward to, for a way back that no
    /// jump recorded
    pub(super) fn left(self, here: Spot) -> Self {
        Self {
            ahead: remembered(self.ahead, here),
            ..self
        }
        .capped()
    }

    #[cfg(test)]
    pub(super) fn can_go_back(&self, here: Spot, alive: impl Fn(Spot) -> bool) -> bool {
        self.behind.iter().any(|&spot| spot != here && alive(spot))
    }

    #[cfg(test)]
    pub(super) fn can_go_forward(&self, here: Spot, alive: impl Fn(Spot) -> bool) -> bool {
        self.ahead.iter().any(|&spot| spot != here && alive(spot))
    }

    /// Drops the oldest places past the limit, those behind first
    fn capped(self) -> Self {
        let excess = (self.behind.len() + self.ahead.len()).saturating_sub(LIMIT);
        let from_ahead = excess.saturating_sub(self.behind.len());
        Self {
            behind: self.behind.into_iter().skip(excess).collect(),
            ahead: self.ahead.into_iter().skip(from_ahead).collect(),
        }
    }
}

/// Returns `places` with `spot` as the latest, and no other of it
fn remembered(places: Vec<Spot>, spot: Spot) -> Vec<Spot> {
    places
        .into_iter()
        .filter(|&place| place != spot)
        .chain([spot])
        .collect()
}

/// Walks `count` places down `from`, whose latest place is last, and returns
/// the place reached with what remains of `from` and `to` with `here` and
/// the places passed on top, the nearest to `here` first, so that walking
/// down `to` retraces the way
pub(super) fn shifted(
    from: &[Spot],
    to: &[Spot],
    here: Spot,
    count: usize,
    alive: impl Fn(Spot) -> bool,
) -> Option<(Spot, Vec<Spot>, Vec<Spot>)> {
    let walked: Vec<(usize, Spot)> = from
        .iter()
        .copied()
        .enumerate()
        .rev()
        .filter(|&(_, spot)| spot != here && alive(spot))
        .take(count.max(1))
        .collect();
    let &(reached, spot) = walked.last()?;
    let passed = walked.iter().map(|&(_, spot)| spot).take(walked.len() - 1);
    let to = [here]
        .into_iter()
        .chain(passed)
        .fold(to.to_vec(), remembered);
    // Place equal to `here` is on `to` now, so that it stays once
    let from = from[..reached]
        .iter()
        .copied()
        .filter(|&place| place != here)
        .collect();
    Some((spot, from, to))
}

/// Returns whether the cue, the cue list, or the output that `spot` names
/// is there
pub(super) fn alive(workspace: &Workspace, spot: Spot) -> bool {
    match spot {
        Spot::Cue(id) => workspace.list_of(id).is_some(),
        Spot::List(list) => list < workspace.lists().len(),
        Spot::Output(id) => workspace.output(id).is_some(),
        Spot::Outputs => true,
    }
}
