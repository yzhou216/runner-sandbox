// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Outputs of a show, the named places that its cues send their sound to
//!
//! An output plays on one channel of an audio device, as a subwoofer
//! does, or on two, left first, as a pair of speakers does.  Cues name
//! outputs and never devices or channels, so that moving an output to other
//! channels, or to another device at another venue, moves every cue that
//! sends to it at once.  A show always has an output, and a show file that
//! names none has Main, on the first two channels of the default device.

#[cfg(test)]
mod tests;

use std::{collections::BTreeMap, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId(pub u32);

/// Device that an output plays on
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Device {
    /// Default device of the machine, which config.toml can choose
    #[default]
    Default,
    /// Device by its ID, and by the name it had when it was chosen, which
    /// finds it on a machine that gives it another ID
    Named { id: String, name: String },
}

/// Channels of a device that an output plays on, counted from zero
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Mono(u16),
    /// Left and right
    Stereo(u16, u16),
}

/// Channels that a device can have, past the 128 of the Dante Virtual
/// Soundcard with a Pro license and of the largest Dante cards, so that a
/// show for them opens while a typo such as 1000 does not
pub const DEVICE_CHANNELS: u16 = 256;

impl Channels {
    /// First two channels, on which Main plays
    pub const MAIN: Self = Self::Stereo(0, 1);

    /// Reads channels as people count them, from one: `3` for a mono
    /// output, and `1-2` or `1,3` for a stereo one
    pub fn parse(text: &str) -> Option<Self> {
        let channel = |text: &str| {
            text.trim()
                .parse::<u16>()
                .ok()
                .filter(|channel| (1..=DEVICE_CHANNELS).contains(channel))
                .map(|channel| channel - 1)
        };
        match text.split_once(['-', ',']) {
            None => channel(text).map(Self::Mono),
            Some((left, right)) => {
                let (left, right) = (channel(left)?, channel(right)?);
                let pair = text.contains(',') || right == left + 1;
                (pair && left != right).then_some(Self::Stereo(left, right))
            }
        }
    }

    /// Returns each channel, the left first
    pub fn each(self) -> impl Iterator<Item = u16> {
        let (first, second) = match self {
            Self::Mono(channel) => (channel, None),
            Self::Stereo(left, right) => (left, Some(right)),
        };
        std::iter::once(first).chain(second)
    }

    /// Returns how many channels a device needs to hold these
    pub fn needs(self) -> u16 {
        self.each().max().map_or(0, |highest| highest + 1)
    }
}

impl fmt::Display for Channels {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Mono(channel) => write!(formatter, "{}", channel + 1),
            Self::Stereo(left, right) if right == left + 1 => {
                write!(formatter, "{}-{}", left + 1, right + 1)
            }
            Self::Stereo(left, right) => write!(formatter, "{},{}", left + 1, right + 1),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub id: OutputId,
    pub name: String,
    pub device: Device,
    pub channels: Channels,
    /// Level in dB at which new cues send here, or none for new cues to
    /// send elsewhere
    pub new_cues: Option<f32>,
}

impl Output {
    /// Returns the output that a show has when its file names none
    pub fn main(id: OutputId) -> Self {
        Self {
            id,
            name: "Main".to_owned(),
            device: Device::Default,
            channels: Channels::MAIN,
            new_cues: Some(0.0),
        }
    }

    pub(super) fn edited(self, edit: OutputEdit) -> Self {
        match edit {
            OutputEdit::Name(name) => Self { name, ..self },
            OutputEdit::Device(device) => Self { device, ..self },
            OutputEdit::Channels(channels) => Self { channels, ..self },
            OutputEdit::NewCues(new_cues) => Self { new_cues, ..self },
        }
    }
}

/// Change that the operator makes to one setting of an output
#[derive(Debug, Clone, PartialEq)]
pub enum OutputEdit {
    Name(String),
    Device(Device),
    Channels(Channels),
    NewCues(Option<f32>),
}

/// Level in dB at which a cue sends to each output that it plays on
pub type Sends = BTreeMap<OutputId, f32>;
