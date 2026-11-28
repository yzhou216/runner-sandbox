// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! When a stream that asks for no sound counts as stopped

use std::time::Duration;

/// Time without a call for sound after which a stream that has called
/// counts as stopped, longer than the 1.9 seconds that a new stream on the
/// ALSA plugin of `PulseAudio` waits after its first calls
pub(super) const STALL: Duration = Duration::from_secs(3);

/// Time that a new stream has for its first call for sound, longer than a
/// device that is slow to wake, as a Bluetooth sink that sleeps, takes
const STARTING: Duration = Duration::from_secs(10);

/// Returns whether a stream that started `elapsed` ago has stalled, as it
/// asked for no sound for [`STALL`] since its latest call at `called`, or
/// for [`STARTING`] since it started if it has not called yet
pub(super) fn stalled(elapsed: Duration, called: Option<Duration>) -> bool {
    match called {
        Some(called) => elapsed.saturating_sub(called) >= STALL,
        None => elapsed >= STARTING,
    }
}
