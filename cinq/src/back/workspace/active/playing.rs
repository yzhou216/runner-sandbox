// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a fired audio, video, or picture cue plays or shows once its
//! pre-wait is over, and how opaque its picture is

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

    pub(in crate::back) fn sound(&self) -> Option<Clip> {
        self.effect.sound().filter(|_| self.under_way())
    }

    /// Returns the clip of an audio or video cue while it has yet to sound,
    /// waiting for its turn in a chain or for its pre-wait to end
    pub(in crate::back) fn pending(&self) -> Option<Clip> {
        self.effect.sound().filter(|_| !self.under_way())
    }

    /// Returns what the stage window shows of the file of a video or
    /// picture cue from the moment GO fires it, even before its picture
    /// shows, so that the video output can make the picture ready
    pub(in crate::back) fn shown(&self) -> Option<Shown> {
        self.effect.shown()
    }

    pub fn shows_picture(&self) -> bool {
        self.shown().is_some() && self.under_way()
    }

    /// Returns whether the cue is a video cue whose picture shows, and so
    /// moves while the cue is not paused
    pub fn shows_video(&self) -> bool {
        matches!(self.shown(), Some(Shown::Clip(_))) && self.under_way()
    }

    pub(in crate::back::workspace) fn playing(&self) -> bool {
        self.effect.plays() && self.under_way()
    }

    /// Returns how opaque the picture of a video or picture cue is, from
    /// none at zero to whole at one.  It falls while a fade that stops the
    /// cue runs, as that of a panic does, so that the picture fades out
    /// along with the sound.
    pub fn opacity(&self) -> f32 {
        self.fade
            .filter(|fade| fade.then_stop)
            .map_or(1.0, |fade| fade.opacity * fade.left() as f32)
    }

    pub(super) fn loops(&self) -> bool {
        self.effect.clip().is_some_and(|clip| clip.looping)
    }

    pub(super) fn endless(&self) -> bool {
        self.effect == Effect::Picture || self.loops()
    }
}
