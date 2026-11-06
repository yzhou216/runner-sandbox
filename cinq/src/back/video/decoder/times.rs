// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Times at which the pictures that a decoder hands out show
//!
//! openh264 hands out pictures in the order they show, but not when each
//! shows, so each picture takes the next of the times at which the samples
//! that the pass decodes show, in order.  A picture that fails to decode
//! would leave every picture after it a picture early, so the times start
//! again from the first picture of each closed group of pictures, which
//! owes nothing to the pictures before it.

#[cfg(test)]
mod tests;

use itertools::Itertools;

/// Times at which the pictures of a pass show, in order, which the
/// pictures that the decoder hands out take in turn
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Times {
    shows: Vec<f64>,
    /// Index of the time that the next picture takes
    next: usize,
}

impl Times {
    /// Returns the times `shows`, in any order, for the pictures of a pass
    pub(super) fn new(shows: impl IntoIterator<Item = f64>) -> Self {
        Self {
            shows: shows.into_iter().sorted_by(f64::total_cmp).collect(),
            next: 0,
        }
    }

    /// Returns the time of the next picture that the decoder hands out, or
    /// none past the last time, along with the times after it
    pub(super) fn next(self) -> (Option<f64>, Self) {
        let shows = self.shows.get(self.next).copied();
        let next = self.next + 1;
        (shows, Self { next, ..self })
    }

    /// Returns the times from the picture that shows at `shows` on, for a
    /// closed group of pictures that starts there
    pub(super) fn from(self, shows: f64) -> Self {
        let next = self.shows.partition_point(|&time| time < shows);
        Self { next, ..self }
    }
}
