// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue numbers that are taken, and the next free one

use std::collections::HashSet;

/// Cue numbers that are taken, kept so that a paste of many cues finds
/// whether a number is held, and the next free one, without going over all
/// the numbers for each cue
pub(super) struct Numbers {
    held: HashSet<String>,
    highest: f64,
}

impl Numbers {
    pub(super) fn of<'a>(numbers: impl Iterator<Item = &'a str>) -> Self {
        numbers.fold(
            Self {
                held: HashSet::new(),
                highest: 0.0,
            },
            |mut numbers, number| {
                numbers.take(number);
                numbers
            },
        )
    }

    pub(super) fn holds(&self, number: &str) -> bool {
        self.held.contains(number.trim())
    }

    /// Returns the next whole number above every numeric cue number, which
    /// keeps new cue numbers apart across all lists as QLab does.  Numbers
    /// too large to count up from leave the next number at the largest
    /// there is.
    pub(super) fn next(&self) -> String {
        (self.highest.floor() as u64).saturating_add(1).to_string()
    }

    pub(super) fn take(&mut self, number: &str) {
        let number = number.trim();
        self.highest = number
            .parse::<f64>()
            .map_or(self.highest, |parsed| parsed.max(self.highest));
        self.held.insert(number.to_owned());
    }
}
