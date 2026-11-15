// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Devices that the outputs need, which open as the outputs change, and
//! again while they fail
//!
//! A device that is not there, or that stops, as one unplugged or a Dante
//! Virtual Soundcard that restarts does, opens again every [`RETRY`] on its
//! own, so that the operator need not act and the devices that play go on
//! unbroken.  The thread of [`super::opener`] opens them, and a device that
//! is being opened keeps what it was until the thread answers, so that no
//! sync waits on a device.

use std::time::{Duration, Instant};

use itertools::Itertools;

use super::{
    Audio, Fault,
    opener::Answer,
    patch::{Key, Need, Patch},
    preload::Preloader,
};

/// Time between tries to open a device that is not there, or that has
/// stopped
pub(super) const RETRY: Duration = Duration::from_secs(2);

impl Audio {
    /// Opens the devices that the outputs need and that are not open, or
    /// that were opened for fewer channels than the outputs need now, and
    /// closes those that they no longer need, which ends the sounds on
    /// them as [`Audio::route`] does for all.  The other devices play on
    /// unbroken, so that a change to one output cannot cut the sound of
    /// the rest.  A device that fails to open, or that stops, opens again
    /// as [`Audio::revived`] tells.  Outputs that move within the channels
    /// that are open only ramp the sounds there, as the next sync does.
    pub(super) fn patched(&mut self, patch: Patch) {
        let needs = patch.needs();
        let fits = |need: &Need| match self.mixers.get(&need.key) {
            Some(Ok(mixer)) => mixer.asked() >= need.channels,
            Some(Err(_)) => true,
            None => self.opener.opening(&need.key),
        };
        let gone: Vec<Key> = self
            .mixers
            .keys()
            .filter(|key| needs.iter().all(|need| need.key != **key))
            .cloned()
            .collect();
        let changed = needs.into_iter().filter(|need| !fits(need)).collect();
        self.patch = patch;
        self.reopened(changed, gone);
    }

    /// Takes in what the thread of the devices has answered since the last
    /// sync: the devices that it opened, which play from here on, and how
    /// many channels the devices that it looked at offer, where a device
    /// that opened with fewer than its outputs need now offers more
    pub(super) fn arrived(&mut self) {
        let needs = self.patch.needs();
        self.opener
            .answers()
            .into_iter()
            .for_each(|answer| match answer {
                Answer::Opened { key, result } => {
                    // A device that the outputs no longer need goes at once
                    if needs.iter().any(|need| need.key == key) {
                        self.mixers.insert(key, result);
                    }
                }
                Answer::Offered { key, offered } => {
                    let have = match self.mixers.get(&key) {
                        Some(Ok(mixer)) => Some(mixer.channels()),
                        _ => None,
                    };
                    let grown = have
                        .zip(offered)
                        .is_some_and(|(have, offered)| offered > have);
                    let need = needs.iter().find(|need| need.key == key);
                    if let Some(need) = need.filter(|_| grown) {
                        self.reopened(vec![need.clone()], Vec::new());
                    }
                }
            });
    }

    /// Opens again, once every [`RETRY`], each device that is not there or
    /// that has stopped, and looks at each that opened with fewer channels
    /// than the outputs need, to open it again once it offers more, as a
    /// Dante Virtual Soundcard does once its channels are raised, and no
    /// other, so that the sounds on the rest play on unbroken.  The runs
    /// that had a sound on a device that stopped start again on all of
    /// theirs, from where they are, as the next part of the sync starts
    /// them, and so do those that come to play on a device that opens at
    /// last.
    pub(super) fn revived(&mut self) {
        if self.tried.elapsed() < RETRY {
            return;
        }
        let channels = |need: &Need| match self.mixers.get(&need.key) {
            Some(Ok(mixer)) => Some(mixer.channels()),
            _ => None,
        };
        let (failed, rest): (Vec<Need>, Vec<Need>) = self
            .patch
            .needs()
            .into_iter()
            .filter(|need| !self.opener.opening(&need.key))
            .partition(|need| match self.mixers.get(&need.key) {
                Some(Ok(mixer)) => mixer.failure().is_some(),
                Some(Err(_)) => true,
                None => false,
            });
        let short: Vec<Need> = rest
            .into_iter()
            .filter(|need| channels(need).is_some_and(|have| have < need.channels))
            .collect();
        if failed.is_empty() && short.is_empty() {
            return;
        }
        self.tried = Instant::now();
        short.into_iter().for_each(|need| self.opener.probe(need));
        self.reopened(failed, Vec::new());
    }

    /// Opens the devices of `needs` anew and closes those of the keys in
    /// `gone`, after ending the sounds that play on any of them, which the
    /// next part of the sync starts again where they should play.  A device
    /// that failed keeps its fault until the thread of the devices answers,
    /// and one that was open and stopped keeps that, so that the warnings
    /// do not flicker.
    fn reopened(&mut self, needs: Vec<Need>, gone: Vec<Key>) {
        let keys: Vec<Key> = needs
            .iter()
            .map(|need| need.key.clone())
            .chain(gone.iter().cloned())
            .unique()
            .collect();
        if keys.is_empty() {
            return;
        }
        self.tried = Instant::now();
        let (stopped, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.voices)
            .into_iter()
            .partition(|(_, voice)| voice.keys().iter().any(|key| keys.contains(key)));
        self.voices = kept.into_iter().collect();
        stopped
            .into_iter()
            .for_each(|(_, voice)| voice.stop(&mut self.mixers));
        self.preloader = Preloader::new();
        // Old streams close first, which frees a device that a new one may
        // open again, as the thread of the devices takes its jobs in order
        keys.iter().for_each(|key| {
            let fault = match self.mixers.remove(key) {
                Some(Err(fault)) => Some(fault),
                Some(Ok(mixer)) => mixer.failure().map(Fault::Stopped),
                None => None,
            };
            if let Some(fault) = fault.filter(|_| !gone.contains(key)) {
                self.mixers.insert(key.clone(), Err(fault));
            }
        });
        needs.into_iter().for_each(|need| self.opener.open(need));
    }
}
