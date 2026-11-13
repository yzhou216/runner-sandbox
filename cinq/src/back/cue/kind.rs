// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a cue does, and what happens to the next cue once it has started

use serde::{Deserialize, Serialize};
use strum::{Display, VariantArray};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Display, VariantArray, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Plays a sound file
    #[default]
    Audio,
    /// Plays a video file: its picture in the stage window, with its sound
    Video,
    /// Shows a picture file in the stage window until something stops it
    Picture,
    /// Fades instruments of the light patch to levels of its own, which
    /// they keep once it has ended
    Light,
    /// Changes the level of a playing audio or video cue over time, and
    /// fades out the picture of a picture or video cue that it stops
    Fade,
    /// Stops another cue
    Stop,
    /// Sends an OSC message over the network, as to a lighting console to
    /// fire one of its own cues
    Network,
    /// Takes time and does nothing else, to pace a chain of cues
    Wait,
    /// Does nothing, as a note in the cue list
    Memo,
}

impl Kind {
    /// Returns whether the operator sets how long the action of this kind
    /// takes.  Audio and video cues take as long as the part of their file
    /// that they play, picture cues last until something stops them, and
    /// the rest are instant.
    pub fn has_duration(self) -> bool {
        matches!(self, Kind::Light | Kind::Fade | Kind::Wait)
    }

    pub fn plays_file(self) -> bool {
        matches!(self, Kind::Audio | Kind::Video)
    }

    pub fn has_file(self) -> bool {
        self.plays_file() || self == Kind::Picture
    }

    pub fn on_stage(self) -> bool {
        matches!(self, Kind::Video | Kind::Picture)
    }

    pub(in crate::back) fn has_level(self) -> bool {
        self.plays_file() || self == Kind::Fade
    }

    pub(in crate::back) fn has_target(self) -> bool {
        matches!(self, Kind::Fade | Kind::Stop)
    }
}

/// What happens to the next cue in the list once this cue has started,
/// which goes by its name in lower case in a show file
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Display, VariantArray, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Advance {
    /// Next cue waits for the operator to press GO
    #[default]
    #[strum(to_string = "wait for GO")]
    Manual,
    /// Next cue starts when this cue's post-wait has elapsed
    #[strum(to_string = "auto-continue")]
    Continue,
    /// Next cue starts when this cue's action ends
    #[strum(to_string = "auto-follow")]
    Follow,
}
