// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a fired cue does once its pre-wait is over, and the fades that fade
//! cues and panics make on the level of another cue

use crate::back::{Clip, CueId, Shown};

/// Change to the level of an audio or video cue that a fade cue or a panic
/// makes, which also fades out the picture of a video or picture cue that
/// it stops
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fade {
    /// Fade cue making the change, or `None` for a panic
    pub(in crate::back::workspace) by: Option<CueId>,
    /// Level in dB the fade ends at
    pub(in crate::back::workspace) to: f32,
    /// Seconds until the fade ends
    pub(in crate::back::workspace) remaining: f64,
    /// Seconds that the whole fade takes
    pub(in crate::back::workspace) length: f64,
    pub(in crate::back::workspace) then_stop: bool,
    /// Opacity of the picture of the faded cue as the fade starts, which is
    /// less than whole where the fade takes over from one that stops the
    /// cue, so that the picture fades on from where it was
    pub(in crate::back::workspace) opacity: f32,
}

impl Fade {
    /// Returns whether a newer fade may take over from this one.  Nothing
    /// takes over from a panic, and only a fade that stops the cue takes
    /// over from another that stops it, so that no fade brings back a cue
    /// on its way out.
    pub(in crate::back::workspace) fn yields_to(&self, newer: &Fade) -> bool {
        self.by.is_some() && (newer.then_stop || !self.then_stop)
    }

    /// Returns the share of the fade that has yet to run, from one as it
    /// starts to none once it ends.  A fade that takes no time ends as it
    /// starts.
    pub(super) fn left(&self) -> f64 {
        if self.length > 0.0 {
            (self.remaining / self.length).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::back::workspace) enum Effect {
    /// Plays a clip of the sound file of an audio cue
    Sound(Clip),
    /// Plays a clip of the file of a video cue: its picture, along with its
    /// sound when the file holds any
    Video { clip: Clip, sound: bool },
    /// Shows the picture file of a picture cue, until something stops it
    Picture,
    /// Makes a fade on the level of a cue
    Fade { target: CueId, fade: Fade },
    /// Stops a cue
    Stop(CueId),
    /// Takes time and does nothing else, as wait and memo cues do
    Nothing,
}

impl Effect {
    pub(super) fn clip(self) -> Option<Clip> {
        match self {
            Effect::Sound(clip) | Effect::Video { clip, .. } => Some(clip),
            _ => None,
        }
    }

    /// Returns the clip whose sound an audio or video cue plays, which a
    /// video file with a picture alone has none of
    pub(super) fn sound(self) -> Option<Clip> {
        match self {
            Effect::Sound(clip) | Effect::Video { clip, sound: true } => Some(clip),
            _ => None,
        }
    }

    pub(super) fn shown(self) -> Option<Shown> {
        match self {
            Effect::Video { clip, .. } => Some(Shown::Clip(clip)),
            Effect::Picture => Some(Shown::Still),
            _ => None,
        }
    }

    pub(super) fn plays(self) -> bool {
        self.clip().is_some() || self == Effect::Picture
    }
}
