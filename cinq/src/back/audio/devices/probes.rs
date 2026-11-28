// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! When to look again at a device that has fewer channels than an output
//! names
//!
//! A Dante Virtual Soundcard whose channels are raised offers more, but a
//! typo in a channel number never will, and looking at a device can open it
//! on ALSA, so the looks slow down, from [`RETRY`] to [`SLOWEST`], until
//! what the device offers changes.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use super::fault::RETRY;
use crate::back::audio::patch::Key;

/// Longest wait between two looks at one device
const SLOWEST: Duration = Duration::from_secs(30);

#[derive(Default)]
pub(in crate::back::audio) struct Probes(HashMap<Key, Probe>);

struct Probe {
    /// When the device was last looked at, or opened
    at: Instant,
    /// How long to wait after `at` for the next look
    wait: Duration,
    /// What the device offered then
    offered: Option<u16>,
}

impl Probe {
    fn new(offered: Option<u16>) -> Self {
        Self {
            at: Instant::now(),
            wait: RETRY,
            offered,
        }
    }
}

impl Probes {
    /// Starts the waits of the device of `key` anew, as it opened with
    /// `offered` channels
    pub(in crate::back::audio) fn opened(&mut self, key: Key, offered: u16) {
        self.0.insert(key, Probe::new(Some(offered)));
    }

    /// Returns whether it is time to look at the device of `key` again
    pub(in crate::back::audio) fn due(&self, key: &Key) -> bool {
        self.0
            .get(key)
            .is_none_or(|probe| probe.at.elapsed() >= probe.wait)
    }

    /// Notes a look at the device of `key`, after which the wait doubles
    pub(in crate::back::audio) fn asked(&mut self, key: &Key) {
        self.0
            .entry(key.clone())
            .and_modify(|probe| {
                probe.at = Instant::now();
                probe.wait = (probe.wait * 2).min(SLOWEST);
            })
            .or_insert_with(|| Probe::new(None));
    }

    /// Notes that the device of `key` offers `offered` channels, and starts
    /// the waits anew if that is not what it offered before
    pub(in crate::back::audio) fn offered(&mut self, key: &Key, offered: Option<u16>) {
        if let Some(probe) = self.0.get_mut(key).filter(|probe| probe.offered != offered) {
            *probe = Probe::new(offered);
        }
    }

    #[cfg(test)]
    fn wait(&self, key: &Key) -> Option<Duration> {
        self.0.get(key).map(|probe| probe.wait)
    }

    /// Makes the latest look at the device of `key` `by` older
    #[cfg(test)]
    pub(in crate::back::audio) fn aged(&mut self, key: &Key, by: Duration) {
        if let Some(probe) = self.0.get_mut(key) {
            probe.at = probe.at.checked_sub(by).expect("the clock is past it");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> Key {
        Some("a".to_owned())
    }

    #[test]
    fn looks_slow_down_from_two_seconds_to_thirty_and_start_over_on_a_change() {
        let mut probes = Probes::default();
        probes.opened(key(), 2);
        let waits: Vec<_> = (0..6)
            .map(|_| {
                probes.asked(&key());
                probes.wait(&key()).expect("the device is known")
            })
            .collect();
        let seconds: Vec<_> = waits.iter().map(Duration::as_secs).collect();
        assert_eq!(seconds, [4, 8, 16, 30, 30, 30]);
        probes.offered(&key(), Some(2));
        assert_eq!(probes.wait(&key()), Some(SLOWEST), "the same offer");
        probes.offered(&key(), Some(8));
        assert_eq!(probes.wait(&key()), Some(RETRY), "a new offer");
    }

    #[test]
    fn device_is_due_once_its_wait_has_passed() {
        let mut probes = Probes::default();
        assert!(probes.due(&key()), "a device not seen is looked at");
        probes.opened(key(), 2);
        assert!(!probes.due(&key()));
        probes.aged(&key(), RETRY);
        assert!(probes.due(&key()));
    }
}
