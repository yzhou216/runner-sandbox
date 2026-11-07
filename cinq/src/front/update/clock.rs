// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Playback clock, which hands the back end the time between its ticks

use std::time::Instant;

use crate::{
    back::{Command, TICK},
    front::App,
};

impl App {
    /// Counts the time since the last tick.  Ticks of the timer and those
    /// that wake for pictures may come out of order, and an earlier one
    /// counts no time.
    pub(super) fn ticked(self, now: Instant) -> Self {
        let elapsed = self
            .last_tick
            .map_or(TICK, |last| now.saturating_duration_since(last));
        Self {
            last_tick: Some(self.last_tick.map_or(now, |last| last.max(now))),
            ..self.send(Command::Tick(elapsed.as_secs_f64()))
        }
    }

    /// Returns the state with the clock going while any cue runs, from now
    /// when it starts, so that its first tick counts only the time since,
    /// and stopped otherwise, so that time spent idle or paused never
    /// reaches the next tick.  It has to wait until the sound and video
    /// output have caught up, since a cue that they cannot play stops then.
    pub(in crate::front) fn clocked(self) -> Self {
        Self {
            last_tick: self
                .running()
                .then(|| self.last_tick.unwrap_or_else(Instant::now)),
            ..self
        }
    }
}
