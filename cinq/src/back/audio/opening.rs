// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Devices that the outputs need, which open as the outputs change, and
//! again while they fail
//!
//! A device that is not there, or that stops, as one unplugged or a Dante
//! Virtual Soundcard that restarts does, opens again every [`RETRY`] on its
//! own, so that the operator need not act and the devices that play go on
//! unbroken.

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use super::{
    Audio,
    mixer::Mixer,
    patch::{Need, Patch},
    preload::Preloader,
};

/// Time between tries to open a device that is not there, or that has
/// stopped
pub(super) const RETRY: Duration = Duration::from_secs(2);

impl Audio {
    /// Opens the devices again where the outputs need others than are
    /// open, or more channels of one than it opened for, which restarts
    /// the sounds that play as [`Audio::route`] does.  A device that fails
    /// to open, or that stops, opens again as [`Audio::revived`] tells.
    /// Outputs that move within the channels that are open only ramp the
    /// sounds there, as the next sync does.
    pub(super) fn patched(&mut self, patch: Patch) {
        let needs = patch.needs();
        let fits = needs.len() == self.mixers.len()
            && needs.iter().all(|need| match self.mixers.get(&need.key) {
                Some(Ok(mixer)) => mixer.asked() >= need.channels,
                Some(Err(_)) => true,
                None => false,
            });
        if !fits {
            self.voices.clear();
            self.preloader = Preloader::new();
            // Old streams close first, which frees a device that a new one
            // may open again
            self.mixers.clear();
            self.mixers = needs
                .into_iter()
                .map(|need| {
                    let mixer =
                        Mixer::open(need.key.as_deref(), need.name.as_deref(), need.channels);
                    (need.key, mixer)
                })
                .collect();
        }
        self.patch = patch;
    }

    /// Opens again, once every [`RETRY`], each device that is not there or
    /// that has stopped, and no other, so that the sounds on the rest play
    /// on unbroken.  The runs that had a sound on a device that stopped
    /// start again on all of theirs, from where they are, as the next part
    /// of the sync starts them, and so do those that come to play on a
    /// device that opens at last.
    pub(super) fn revived(&mut self) {
        let failed: Vec<Need> = self
            .patch
            .needs()
            .into_iter()
            .filter(|need| match self.mixers.get(&need.key) {
                Some(Ok(mixer)) => mixer.failure().is_some(),
                Some(Err(_)) => true,
                None => false,
            })
            .collect();
        if failed.is_empty() || self.tried.elapsed() < RETRY {
            return;
        }
        self.tried = Instant::now();
        failed.into_iter().for_each(|need| {
            let (stopped, kept): (HashMap<_, _>, HashMap<_, _>) = std::mem::take(&mut self.voices)
                .into_iter()
                .partition(|(_, voice)| voice.keys().contains(&&need.key));
            self.voices = kept;
            stopped
                .into_values()
                .for_each(|voice| voice.stop(&mut self.mixers));
            // Old stream closes first, which frees the device for the new one
            self.mixers.remove(&need.key);
            let mixer = Mixer::open(need.key.as_deref(), need.name.as_deref(), need.channels);
            self.mixers.insert(need.key, mixer);
        });
    }
}
