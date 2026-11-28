// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the outputs of the show play, and the gains on each device that
//! the sends of a cue make
//!
//! A stereo output takes the left of a sound on its first channel and the
//! right on its second, so that a mono file, which kira plays alike on
//! both, plays on both.  A mono output takes half of each, so that a stereo
//! file plays there as the average of its sides and a mono file at its own
//! level.  Outputs that share a channel add up on it.

use itertools::Itertools;
use kira::Decibels;

use crate::back::{Channels, Device, Output, OutputId, Sends};

/// Gain from the left and the right of a sound into each channel of a
/// device, in order
pub(super) type Gains = Box<[[f32; 2]]>;

/// Device of a mixer, by its ID, or none for the default device of the
/// system
pub(super) type Key = Option<String>;

/// Output as the sound output plays it
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Patched {
    pub(super) id: OutputId,
    pub(super) key: Key,
    /// Name of the device, which finds it on a machine that gives it
    /// another ID
    name: Option<String>,
    pub(super) channels: Channels,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Patch(Vec<Patched>);

/// Device that some outputs play on, with the channels it needs for them
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Need {
    pub(super) key: Key,
    pub(super) name: Option<String>,
    pub(super) channels: u16,
}

impl Patch {
    /// Returns where `outputs` play, those on the default device on the
    /// device of `default`
    pub(super) fn of(outputs: &[Output], default: &Key) -> Self {
        Self(
            outputs
                .iter()
                .map(|output| {
                    let (key, name) = match &output.device {
                        Device::Default => (default.clone(), None),
                        Device::Named { id, name } => (Some(id.clone()), Some(name.clone())),
                    };
                    Patched {
                        id: output.id,
                        key,
                        name,
                        channels: output.channels,
                    }
                })
                .collect(),
        )
    }

    pub(super) fn outputs(&self) -> &[Patched] {
        &self.0
    }

    /// Returns each device that the outputs play on, once, in the order of
    /// the outputs, with at least two channels, as kira renders in stereo
    pub(super) fn needs(&self) -> Vec<Need> {
        self.0
            .iter()
            .unique_by(|patched| &patched.key)
            .map(|first| Need {
                key: first.key.clone(),
                name: first.name.clone(),
                channels: self
                    .0
                    .iter()
                    .filter(|patched| patched.key == first.key)
                    .map(|patched| patched.channels.needs())
                    .fold(2, u16::max),
            })
            .collect()
    }

    /// Returns the gains on each device that `sends` play on, in the order
    /// of the outputs, given the channels of each device that is open.  A
    /// channel past those of its device plays nowhere.
    pub(super) fn gains(
        &self,
        sends: &Sends,
        channels: impl Fn(&Key) -> Option<u16>,
    ) -> Vec<(Key, Gains)> {
        let sending: Vec<(&Patched, f32, u16)> = self
            .0
            .iter()
            .filter_map(|patched| {
                let level = sends.get(&patched.id)?;
                let count = channels(&patched.key)?;
                Some((patched, Decibels(*level).as_amplitude(), count))
            })
            .collect();
        sending
            .iter()
            .unique_by(|(patched, ..)| &patched.key)
            .map(|(first, _, count)| {
                let silent = vec![[0.0; 2]; usize::from(*count)];
                let gains = sending
                    .iter()
                    .filter(|(patched, ..)| patched.key == first.key)
                    .fold(silent, |gains, (patched, amplitude, _)| {
                        tapped(gains, patched.channels, *amplitude)
                    });
                (first.key.clone(), gains.into_boxed_slice())
            })
            .collect()
    }
}

/// Adds to `gains` what an output on `channels` sends at `amplitude`
fn tapped(gains: Vec<[f32; 2]>, channels: Channels, amplitude: f32) -> Vec<[f32; 2]> {
    let taps = match channels {
        Channels::Mono(channel) => vec![(channel, [amplitude / 2.0; 2])],
        Channels::Stereo(left, right) => {
            vec![(left, [amplitude, 0.0]), (right, [0.0, amplitude])]
        }
    };
    gains
        .into_iter()
        .zip(0..)
        .map(|(gain, channel)| {
            taps.iter()
                .filter(|(tap, _)| *tap == channel)
                .fold(gain, |sum, (_, tap)| [sum[0] + tap[0], sum[1] + tap[1]])
        })
        .collect()
}
