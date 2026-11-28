// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a cue works out of its settings and its loaded file: its clip, its
//! loop, its sound, and what the stage window shows of it

use super::{
    super::{Cue, Kind, Media},
    Clip, Loop, Shown,
};

impl Cue {
    /// A video cue that loops while its sound plays ends with its sound,
    /// where that comes first, so that the sound loops with the picture.
    /// The loop of a cue lies within its clip, so a loop that starts before
    /// the clip starts with the clip, and one that ends after the clip ends
    /// with it.
    pub(in crate::back) fn clip(&self) -> Option<Clip> {
        let (start, end) = self.span()?;
        let looping = self.looping.then(|| self.section(start, end));
        looping
            .is_none_or(|section| section.plays())
            .then_some(Clip {
                start,
                end,
                looping,
            })
    }

    /// Returns whether the cue loops nothing, as one whose loop starts at or
    /// after the end of its loop does
    pub(in crate::back) fn loops_nothing(&self) -> bool {
        self.loop_section()
            .is_some_and(|section| section.length() <= 0.0)
    }

    /// Returns whether the loop of the cue lasts too short a time to play,
    /// though it lasts some
    pub(in crate::back) fn loops_briefly(&self) -> bool {
        self.loop_section()
            .is_some_and(|section| section.length() > 0.0 && !section.plays())
    }

    /// Returns the loop of a cue that loops, as far as its clip allows
    fn loop_section(&self) -> Option<Loop> {
        let (start, end) = self.span().filter(|_| self.loops())?;
        Some(self.section(start, end))
    }

    /// Returns the loop that the cue plays, as seconds from its start to its
    /// end in the file, where that differs from the times typed, because a
    /// typed time lies outside the clip, so that the operator can be told
    /// what plays instead
    pub fn loop_played(&self) -> Option<(f64, f64)> {
        let section = self.loop_section()?;
        let cut =
            |typed: Option<f32>, played: f64| typed.is_some_and(|typed| f64::from(typed) != played);
        (section.plays()
            && (cut(self.loop_start.time(), section.start)
                || cut(self.loop_end.time(), section.end)))
        .then_some((section.start, section.end))
    }

    /// Returns where the clip of the cue starts and ends in its file, once
    /// the file has loaded, when the clip starts before it ends
    fn span(&self) -> Option<(f64, f64)> {
        let Media::Loaded { duration, .. } = self.media else {
            return None;
        };
        let length = match self.sound() {
            Some(sound) if self.looping => duration.min(sound),
            _ => duration,
        };
        let start = self.start.value();
        let end = self.end.time().map_or(length, |end| end.min(length));
        (self.kind.plays_file() && start < end).then_some((start.into(), end.into()))
    }

    fn section(&self, start: f64, end: f64) -> Loop {
        Loop {
            start: self
                .loop_start
                .time()
                .map_or(start, |time| f64::from(time).max(start)),
            end: self
                .loop_end
                .time()
                .map_or(end, |time| f64::from(time).min(end)),
            released: None,
        }
    }

    pub(in crate::back) fn shown(&self) -> Option<Shown> {
        match self.kind {
            Kind::Video => self.clip().map(Shown::Clip),
            Kind::Picture => matches!(self.media, Media::Loaded { .. }).then_some(Shown::Still),
            _ => None,
        }
    }

    /// Returns how long the sound of the file of the cue plays, once the
    /// file has loaded, when the clip of the cue starts before the sound
    /// ends
    fn sound(&self) -> Option<f32> {
        match self.media {
            Media::Loaded { sound, .. } => sound.filter(|&sound| self.start.value() < sound),
            _ => None,
        }
    }

    /// Returns where in its file the sound of the cue should end for `clip`,
    /// its clip: at the end of the clip, or at the end of the sound where
    /// that comes first, as that of a video file can before its picture
    pub(in crate::back) fn sound_ends(&self, clip: Clip) -> Option<f64> {
        self.sound().map(|sound| clip.end.min(f64::from(sound)))
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
