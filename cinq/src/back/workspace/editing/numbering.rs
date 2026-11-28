// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue numbers that are taken, and the next free one

use std::collections::HashSet;

/// Cue numbers that are taken, kept so that a paste of many cues finds
/// whether a number is held, and the next free one, without going over all
/// the numbers for each cue
#[derive(Default)]
pub(super) struct Numbers {
    held: HashSet<String>,
    highest: f64,
}

impl Numbers {
    pub(super) fn of<'a>(numbers: impl Iterator<Item = &'a str>) -> Self {
        numbers.fold(Self::default(), Self::with_taken)
    }

    /// Returns the next whole number above every numeric cue number, which
    /// keeps new cue numbers apart across all lists as QLab does.  Numbers
    /// too large to count up from leave the next number at the largest
    /// there is.
    pub(super) fn next(&self) -> String {
        (self.highest.floor() as u64).saturating_add(1).to_string()
    }

    /// Returns the number that a cue numbered `wanted` takes, which is
    /// `wanted` unless another cue holds it, with the numbers that have it
    /// taken
    pub(super) fn claim(self, wanted: &str) -> (String, Self) {
        let number = if wanted.trim().is_empty() || !self.held.contains(wanted.trim()) {
            wanted.to_owned()
        } else {
            self.next()
        };
        let taken = self.with_taken(&number);
        (number, taken)
    }

    pub(super) fn with_taken(self, number: &str) -> Self {
        let number = number.trim();
        let mut held = self.held;
        held.insert(number.to_owned());
        Self {
            // Words such as inf and NaN parse as numbers, but are text to an operator
            highest: number
                .parse::<f64>()
                .ok()
                .filter(|parsed| parsed.is_finite())
                .map_or(self.highest, |parsed| parsed.max(self.highest)),
            held,
        }
    }
}
