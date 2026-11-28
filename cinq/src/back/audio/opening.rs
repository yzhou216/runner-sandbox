// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Devices that the outputs need, which open as the outputs change, and
//! again while they fail
//!
//! A device that is not there, or that stops, as one unplugged or a Dante
//! Virtual Soundcard that restarts does, opens again every [`RETRY`] on its
//! own, so that the operator need not act and the devices that play go on
//! unbroken.  The threads of [`super::sound::opener`] open them, and a
//! device that is being opened keeps what it was until its thread answers,
//! so that no sync waits on a device.

use std::time::Instant;

use itertools::Itertools;

use crate::back::audio::{
    Audio, Fault,
    devices::fault::RETRY,
    patch::{Key, Need, Patch},
    sound::{mixer::channels_of, opener::Answer},
};

impl Audio {
    /// Opens the devices that the outputs need and that are not open, or
    /// that were opened for fewer channels than the outputs need now.
    /// Closes those that the outputs no longer need, which ends the sounds
    /// on them.  The other devices play on unbroken, so that a change to one
    /// output cannot cut the sound of the rest.  Outputs that move within
    /// the channels that are open only ramp the sounds there, as the next
    /// sync does.
    pub(super) fn patched(&mut self, patch: Patch) {
        let needs = patch.needs();
        let fits = |need: &Need| match self.mixers.get(&need.key) {
            Some(Ok(mixer)) => mixer.asked() >= need.channels,
            Some(Err(_)) => true,
            None => self.opener.opening(&need.key),
        };
        let gone = self
            .mixers
            .keys()
            .filter(|key| needs.iter().all(|need| need.key != **key))
            .cloned()
            .collect();
        let changed = needs.into_iter().filter(|need| !fits(need)).collect();
        self.patch = patch;
        self.reopened(changed, gone);
    }

    /// Takes in what the threads of the devices have answered since the last
    /// sync: the devices that they opened, which play from here on, and how
    /// many channels the devices that they looked at offer, where a device
    /// that opened with fewer than its outputs need now offers more
    pub(super) fn arrived(&mut self) {
        let needs = self.patch.needs();
        self.opener
            .answers()
            .into_iter()
            .for_each(|answer| match answer {
                Answer::Opened { key, result } => {
                    // Next try of the devices that failed waits from now
                    self.tried = Instant::now();
                    // Device that the outputs no longer need goes at once
                    if needs.iter().any(|need| need.key == key) {
                        if let Ok(mixer) = &result {
                            self.probes.opened(key.clone(), mixer.channels());
                        }
                        self.mixers.insert(key, result);
                    }
                }
                Answer::Offered { key, offered } => {
                    self.probes.offered(&key, offered);
                    let grown = channels_of(&self.mixers, &key)
                        .zip(offered)
                        .is_some_and(|(have, offered)| offered > have);
                    let need = needs.iter().find(|need| need.key == key);
                    if let Some(need) = need.filter(|_| grown) {
                        self.reopened(vec![need.clone()], Vec::new());
                    }
                }
            });
    }

    /// Opens again each device that is absent or has stopped, once every
    /// [`RETRY`] after its latest answer, unless its fault is one that the
    /// show must mend.  Also looks at each device that opened with fewer
    /// channels than the outputs need, to open it once it offers more, ever
    /// more seldom as [`super::devices::probes`] tells.  The runs that had a
    /// sound on a device that stopped start again on all of theirs, as the
    /// next part of the sync starts them, and so do those that come to play
    /// on a device that opens at last.
    pub(super) fn revived(&mut self) {
        let due = self.tried.elapsed() >= RETRY;
        let (failed, rest): (Vec<Need>, Vec<Need>) = self
            .patch
            .needs()
            .into_iter()
            .filter(|need| !self.opener.opening(&need.key))
            .partition(|need| match self.mixers.get(&need.key) {
                Some(Ok(mixer)) => mixer.failure().is_some(),
                Some(Err(fault)) => fault.retried(),
                None => false,
            });
        // Dante Virtual Soundcard offers more channels once they are raised
        let short: Vec<Need> = rest
            .into_iter()
            .filter(|need| {
                channels_of(&self.mixers, &need.key).is_some_and(|have| have < need.channels)
            })
            .filter(|need| self.probes.due(&need.key))
            .collect();
        short.into_iter().for_each(|need| {
            self.probes.asked(&need.key);
            self.opener.probe(need);
        });
        if due {
            self.reopened(failed, Vec::new());
        }
    }

    /// Opens the devices of `needs` anew and closes those of the keys in
    /// `gone`, after ending the sounds that play on any of them.  A device
    /// that failed keeps its fault until its thread answers, and one that
    /// was open and stopped keeps that, so that the warnings do not flicker.
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
        self.voices = std::mem::take(&mut self.voices)
            .into_iter()
            .map(|(id, voice)| {
                (
                    id,
                    voice.retained(|key| !keys.contains(key), &mut self.mixers),
                )
            })
            .filter(|(_, voice)| !voice.is_empty())
            .collect();
        // Old streams close first, which frees a device that a new one may
        // open again, as the thread of a device takes its jobs in order
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
