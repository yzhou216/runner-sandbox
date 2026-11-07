// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Part of a file that an audio or video cue plays, and what the stage
//! window shows of the file of a video or picture cue

use super::{Cue, Kind, Media};

/// Part of its file that an audio or video cue plays, from `start` to `end`
/// in seconds into the file, where `start` comes before `end`.  A clip
/// that loops plays from its start again each time it reaches its end,
/// until something stops its cue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clip {
    pub(in crate::back) start: f64,
    pub(in crate::back) end: f64,
    pub(in crate::back) looping: bool,
}

/// What the stage window shows of the file of a video or picture cue
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shown {
    Clip(Clip),
    Still,
}

impl Clip {
    pub(in crate::back) fn length(&self) -> f64 {
        self.end - self.start
    }

    /// Returns where in the file the clip is once `played` seconds of it
    /// have played, counting every pass of a clip that loops
    pub(in crate::back) fn position(&self, played: f64) -> f64 {
        let into = if self.looping {
            played % self.length()
        } else {
            played
        };
        self.start + into
    }
}

impl Cue {
    /// Returns the part of its file that an audio or video cue plays: from
    /// its start time to its end time, or to the end of the file where that
    /// comes first.  A video cue that loops while its sound plays ends with
    /// its sound, where that comes first, so that the sound loops with the
    /// picture.  There is none for other kinds of cue, before the file has
    /// loaded, or when the start is not before the end.
    pub(in crate::back) fn clip(&self) -> Option<Clip> {
        let Media::Loaded { duration, .. } = self.media else {
            return None;
        };
        let length = match self.sound() {
            Some(sound) if self.looping => duration.min(sound),
            _ => duration,
        };
        let start = self.start.value();
        let end = self.end.time().map_or(length, |end| end.min(length));
        (self.kind.plays_file() && start < end).then_some(Clip {
            start: start.into(),
            end: end.into(),
            looping: self.looping,
        })
    }

    /// Returns what the stage window shows of the file of the cue, once the
    /// file has loaded
    pub(in crate::back) fn shown(&self) -> Option<Shown> {
        match self.kind {
            Kind::Video => self.clip().map(Shown::Clip),
            Kind::Picture => matches!(self.media, Media::Loaded { .. }).then_some(Shown::Still),
            _ => None,
        }
    }

    /// Returns how long the sound of the file of the cue plays, once the
    /// file has loaded, when the clip of the cue starts before the sound
    /// ends.  There is none for a video file with a picture alone, nor for
    /// a picture file.
    fn sound(&self) -> Option<f32> {
        match self.media {
            Media::Loaded {
                sound: Some(sound), ..
            } => Some(sound).filter(|&sound| self.start.value() < sound),
            _ => None,
        }
    }

    /// Returns whether the clip of the cue has sound to play: that of an
    /// audio cue always has, once its file has loaded, and that of a video
    /// cue has unless its file holds a picture alone or its sound ends
    /// before the clip starts
    pub fn sounds(&self) -> bool {
        self.sound().is_some()
    }

    pub fn loops(&self) -> bool {
        self.kind.plays_file() && self.looping
    }
}
