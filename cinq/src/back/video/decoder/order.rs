// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Order in which the pictures of a pass go out: the order they show in
//!
//! A decoder finishes the pictures of a pass in the order that their
//! samples decode, which with B-frames differs from the order they show
//! in.  Each picture takes the time of its own sample, so that one that
//! fails to decode costs no other picture its time, and a picture goes out
//! once every sample that shows before it has decoded, to a picture or to
//! none.

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use itertools::Itertools;

/// Pictures of a pass that wait for those that show before them, each by
/// the place of its sample in the order of decoding
pub(super) struct Order<T> {
    /// Times at which the samples of the pass show, in the order they
    /// decode
    shows: Vec<f64>,
    /// Places of the samples in the order they show
    showing: Vec<usize>,
    /// How many places of `showing` have had their turn
    next: usize,
    /// Pictures that have decoded and wait for their turn
    held: BTreeMap<usize, T>,
}

impl<T> Order<T> {
    /// Returns the order of the pictures of samples that show at `shows`,
    /// which go in the order they decode, before any has decoded
    pub(super) fn new(shows: impl IntoIterator<Item = f64>) -> Self {
        let shows: Vec<f64> = shows.into_iter().collect();
        let showing = (0..shows.len())
            .sorted_by(|&a, &b| shows[a].total_cmp(&shows[b]))
            .collect();
        Self {
            shows,
            showing,
            next: 0,
            held: BTreeMap::new(),
        }
    }

    /// Returns the pictures that go out once the sample at `place` in the
    /// order of decoding has decoded to `picture`, or to none, each with
    /// the time it shows and in the order they show, along with the order
    /// of those left.  Samples decode in order, so every sample before
    /// `place` has decoded by then, and once the last has, every picture
    /// has gone out.
    pub(super) fn decoded(self, place: usize, picture: Option<T>) -> (Vec<(f64, T)>, Self) {
        // Samples that show next, up to the first that has yet to decode
        let ready = self.showing[self.next..]
            .iter()
            .take_while(|&&sample| sample <= place)
            .count();
        let turn = &self.showing[self.next..self.next + ready];
        let (due, held): (BTreeMap<usize, T>, _) = self
            .held
            .into_iter()
            .chain(picture.map(|picture| (place, picture)))
            .partition(|(sample, _)| turn.contains(sample));
        let due = due
            .into_iter()
            .map(|(sample, picture)| (self.shows[sample], picture))
            .sorted_by(|(a, _), (b, _)| a.total_cmp(b))
            .collect();
        let next = self.next + ready;
        (due, Self { next, held, ..self })
    }
}
