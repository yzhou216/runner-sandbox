// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Limit on how often the interface hears of what was no command

use std::{
    cell::Cell,
    time::{Duration, Instant},
};

use super::Heard;

/// Shortest time between two notes of what was no command, since the
/// sidebar keeps one line and each note runs a whole update of the interface
pub(super) const NOTE: Duration = Duration::from_secs(1);

/// Lets one note through each [`NOTE`], the first at once, and every other
/// kind of message always, since a dropped command is worse than the memory
/// of a flood
#[derive(Debug, Default)]
pub(super) struct Notes {
    last: Cell<Option<Instant>>,
}

impl Notes {
    /// Returns whether `heard` goes on to the interface at `now`
    pub(super) fn allow(&self, heard: &Heard, now: Instant) -> bool {
        !matches!(heard, Heard::Ignored(_))
            || self
                .last
                .get()
                .is_none_or(|at| now.saturating_duration_since(at) >= NOTE)
                && {
                    self.last.set(Some(now));
                    true
                }
    }
}
