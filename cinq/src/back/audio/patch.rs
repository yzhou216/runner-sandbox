// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the outputs of the show play, and the gains on each device that
//! the sends of a cue make
//!
//! A stereo output takes the left of a sound on its first channel and the
//! right on its second, so that a mono file, which kira plays alike on
//! both, plays on both.  A mono output takes half of each, so that a stereo
//! file plays there as its sum and a mono file at its own level.  Outputs
//! that share a channel add up on it.

use kira::Decibels;

use super::mixer::Gains;
use crate::back::{Channels, Device, Output, OutputId, Sends};

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
    pub(super) name: Option<String>,
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
    /// the outputs, with at least the two channels that kira's own output
    /// opened
    pub(super) fn needs(&self) -> Vec<Need> {
        self.0.iter().fold(Vec::new(), |needs, patched| {
            let channels = patched.channels.needs();
            let found = needs.iter().any(|need: &Need| need.key == patched.key);
            if found {
                needs
                    .into_iter()
                    .map(|need| {
                        if need.key == patched.key {
                            Need {
                                channels: need.channels.max(channels),
                                ..need
                            }
                        } else {
                            need
                        }
                    })
                    .collect()
            } else {
                let need = Need {
                    key: patched.key.clone(),
                    name: patched.name.clone(),
                    channels: channels.max(2),
                };
                needs.into_iter().chain([need]).collect()
            }
        })
    }

    /// Returns the gains on each device that `sends` play on, those of the
    /// device of the first output that the cue sends to first, given the
    /// channels of each device that is open.  A channel past those of its
    /// device plays nowhere.
    pub(super) fn gains(
        &self,
        sends: &Sends,
        channels: impl Fn(&Key) -> Option<u16>,
    ) -> Vec<(Key, Gains)> {
        self.0
            .iter()
            .filter_map(|patched| {
                let level = sends.get(&patched.id)?;
                let count = channels(&patched.key)?;
                Some((patched, Decibels(*level).as_amplitude(), count))
            })
            .fold(
                Vec::new(),
                |devices: Vec<(Key, Gains)>, (patched, amplitude, count)| {
                    let taps: Vec<(u16, [f32; 2])> = match patched.channels {
                        Channels::Mono(channel) => vec![(channel, [amplitude / 2.0; 2])],
                        Channels::Stereo(left, right) => {
                            vec![(left, [amplitude, 0.0]), (right, [0.0, amplitude])]
                        }
                    };
                    let tapped = |gains: Gains| -> Gains {
                        gains
                            .iter()
                            .zip(0..)
                            .map(|(gain, channel)| {
                                taps.iter()
                                    .filter(|(tap, _)| *tap == channel)
                                    .fold(*gain, |sum, (_, tap)| [sum[0] + tap[0], sum[1] + tap[1]])
                            })
                            .collect()
                    };
                    if devices.iter().any(|(key, _)| *key == patched.key) {
                        devices
                            .into_iter()
                            .map(|(key, gains)| {
                                if key == patched.key {
                                    (key, tapped(gains))
                                } else {
                                    (key, gains)
                                }
                            })
                            .collect()
                    } else {
                        let silent = vec![[0.0; 2]; usize::from(count)].into_boxed_slice();
                        let entry = (patched.key.clone(), tapped(silent));
                        devices.into_iter().chain([entry]).collect()
                    }
                },
            )
    }
}
