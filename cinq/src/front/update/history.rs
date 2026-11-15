// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The places that jumps left, which going back and forward walk along, on
//! the model of the jump list of Vim
//!
//! A place is a cue list with a selected cue, or a cue list with none.  The
//! history lives in the interface state only, so that neither the show file
//! nor undo ever holds it.

#[cfg(test)]
mod tests;

use crate::{
    back::{CueId, Workspace},
    front::App,
};

/// Longest history, as in Vim
const LIMIT: usize = 100;

/// A place in the show.  A cue names its place by its identity, so that the
/// place follows the cue as it moves between cue lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::front) enum Spot {
    Cue(CueId),
    /// Cue list with no selected cue, which only its index names
    List(usize),
}

/// Places that jumps left, the latest last, and the places that going back
/// passed, the nearest last
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(in crate::front) struct History {
    behind: Vec<Spot>,
    ahead: Vec<Spot>,
}

impl History {
    /// Remembers `from`, the place that a jump left, once and as the latest
    fn jumped(self, from: Spot) -> Self {
        Self {
            behind: remembered(self.behind, from),
            ..self
        }
        .without_places_ahead()
        .capped()
    }

    /// Forgets the places ahead, as a browser does when it opens a page
    /// after going back.  Vim keeps them by default, as its `jumpoptions`
    /// lacks `stack`: a function that moves the places ahead behind, the
    /// nearest first, in place of this one gives that behavior.
    fn without_places_ahead(self) -> Self {
        Self {
            ahead: Vec::new(),
            ..self
        }
    }

    /// Goes `count` places back from `here`, or to the oldest place when
    /// there are fewer, and returns it with the history that remembers
    /// `here`, so that going forward returns to it.  Places that are gone
    /// or equal `here` are skipped and dropped.
    fn back(&self, here: Spot, count: usize, alive: impl Fn(Spot) -> bool) -> Option<(Spot, Self)> {
        let (spot, behind, ahead) = shifted(&self.behind, &self.ahead, here, count, alive)?;
        Some((spot, Self { behind, ahead }.capped()))
    }

    /// Goes `count` places forward from `here`, as [`History::back`] goes
    /// back
    fn forward(
        &self,
        here: Spot,
        count: usize,
        alive: impl Fn(Spot) -> bool,
    ) -> Option<(Spot, Self)> {
        let (spot, ahead, behind) = shifted(&self.ahead, &self.behind, here, count, alive)?;
        Some((spot, Self { behind, ahead }.capped()))
    }

    fn can_go_back(&self, here: Spot, alive: impl Fn(Spot) -> bool) -> bool {
        self.behind.iter().any(|&spot| spot != here && alive(spot))
    }

    fn can_go_forward(&self, here: Spot, alive: impl Fn(Spot) -> bool) -> bool {
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
fn shifted(
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
    // A place equal to `here` is on `to` now, so that it stays once
    let from = from[..reached]
        .iter()
        .copied()
        .filter(|&place| place != here)
        .collect();
    Some((spot, from, to))
}

/// Returns whether the cue or the cue list that `spot` names is there
fn alive(workspace: &Workspace, spot: Spot) -> bool {
    match spot {
        Spot::Cue(id) => workspace.list_of(id).is_some(),
        Spot::List(list) => list < workspace.lists().len(),
    }
}

impl App {
    fn spot(&self) -> Spot {
        self.selected_here()
            .map_or(Spot::List(self.list), Spot::Cue)
    }

    fn at(self, spot: Spot) -> Self {
        match spot {
            Spot::Cue(id) => self.with_selection(id),
            Spot::List(list) => Self {
                list,
                selected: None,
                ..self
            },
        }
    }

    /// Runs `go`, a command that moves to another place by what it is, and
    /// remembers the place that it left unless it stayed
    pub(super) fn jumping(self, go: impl FnOnce(Self) -> Self) -> Self {
        let from = self.spot();
        let app = go(self);
        if app.spot() == from {
            app
        } else {
            Self {
                history: app.history.jumped(from),
                ..app
            }
        }
    }

    /// Runs `go` as [`App::jumping`] does, but remembers the place only when
    /// the cue list changed, since a click on a cue of the list on screen
    /// points at what is in view
    pub(super) fn crossing(self, go: impl FnOnce(Self) -> Self) -> Self {
        let (from, list) = (self.spot(), self.list);
        let app = go(self);
        if app.list == list {
            app
        } else {
            Self {
                history: app.history.jumped(from),
                ..app
            }
        }
    }

    pub(super) fn went_back(self, count: usize) -> Self {
        let here = self.spot();
        let back = self
            .history
            .back(here, count, |spot| alive(&self.workspace, spot));
        match back {
            Some((spot, history)) => Self { history, ..self }.at(spot),
            None => self,
        }
    }

    pub(super) fn went_forward(self, count: usize) -> Self {
        let here = self.spot();
        let forward = self
            .history
            .forward(here, count, |spot| alive(&self.workspace, spot));
        match forward {
            Some((spot, history)) => Self { history, ..self }.at(spot),
            None => self,
        }
    }

    pub(in crate::front) fn can_go_back(&self) -> bool {
        self.history
            .can_go_back(self.spot(), |spot| alive(&self.workspace, spot))
    }

    pub(in crate::front) fn can_go_forward(&self) -> bool {
        self.history
            .can_go_forward(self.spot(), |spot| alive(&self.workspace, spot))
    }
}
