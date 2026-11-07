// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a fired audio, video, or picture cue plays or shows once its
//! pre-wait is over: the clip of its file, with its sound, its picture, or
//! both, or the picture of its file, and how opaque its picture is

use super::{Active, Effect, Phase};
use crate::back::{Clip, Shown};

impl Active {
    /// Returns whether the action is under way, paused or not, which it is
    /// once the pre-wait is over
    fn under_way(&self) -> bool {
        matches!(
            self.phase(),
            Phase::Running { .. } | Phase::Looping | Phase::Showing
        )
    }

    /// Returns the clip of an audio or video cue while it has a sound,
    /// paused or not, which it has once its pre-wait is over
    pub(in crate::back) fn sound(&self) -> Option<Clip> {
        self.effect.sound().filter(|_| self.under_way())
    }

    /// Returns the clip of an audio or video cue while it has yet to sound,
    /// waiting for its turn in a chain or for its pre-wait to end
    pub(in crate::back) fn pending(&self) -> Option<Clip> {
        self.effect.sound().filter(|_| !self.under_way())
    }

    /// Returns what the stage window shows of the file of a video or
    /// picture cue from the moment GO fires it, which shows once the
    /// pre-wait is over
    pub(in crate::back) fn shown(&self) -> Option<Shown> {
        self.effect.shown()
    }

    /// Returns whether the cue is a video or picture cue whose picture
    /// shows, paused or not, which it does once its pre-wait is over
    pub fn shows_picture(&self) -> bool {
        self.shown().is_some() && self.under_way()
    }

    /// Returns whether the cue is a video cue whose picture shows, paused
    /// or not, and so moves while the cue is not paused
    pub fn shows_video(&self) -> bool {
        matches!(self.shown(), Some(Shown::Clip(_))) && self.under_way()
    }

    /// Returns whether the cue plays the clip of an audio or video cue,
    /// with sound or without, or shows the picture of a picture cue, paused
    /// or not, which it does once its pre-wait is over
    pub(in crate::back::workspace) fn playing(&self) -> bool {
        self.effect.plays() && self.under_way()
    }

    /// Returns how opaque the picture of a video or picture cue is, from
    /// none at zero to whole at one: whole, apart from while a fade that
    /// stops the cue runs, as that of a panic does, when it falls with the
    /// progress of the fade, so that the picture fades out along with the
    /// sound
    pub fn opacity(&self) -> f32 {
        self.fade
            .filter(|fade| fade.then_stop)
            .map_or(1.0, |fade| fade.left() as f32)
    }

    /// Returns whether the action goes on until something stops the cue,
    /// as the clip of an audio or video cue that loops does
    pub(super) fn loops(&self) -> bool {
        self.effect.clip().is_some_and(|clip| clip.looping)
    }

    /// Returns whether the action goes on until something stops the cue,
    /// as that of a picture cue does, and the clip of a cue that loops
    pub(super) fn endless(&self) -> bool {
        self.effect == Effect::Picture || self.loops()
    }
}
