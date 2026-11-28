// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cues that GO has fired, with what they do and the fades they make in
//! [`effect`], and what they play and show in [`playing`]
//!
//! Time here is counted in seconds as `f64`, since a clock that adds a
//! tick forty times a second for an hour would drift by seconds in `f32`.

mod clock;
mod effect;
mod group;
mod levels;
mod playing;

use crate::back::{Cue, CueId, Kind};
pub(super) use effect::{Effect, Fade};

/// Cue that GO has fired and that has not finished yet
///
/// A fired cue keeps its timing and its effect, such as the clip of an
/// audio or video cue, as its cue had them when GO fired it, so that an
/// edit changes the next run of a cue and not the one under way.  Only a
/// running audio or video cue follows an edit at once: its level moves by
/// the change in the level of its cue, and it stops when it gets a new
/// file.  The fades that a fade cue makes, and its runs that have yet to
/// act, take the new level of the fade cue.  A light cue reads its levels,
/// and a network cue its message, as its action begins.
#[derive(Debug, Clone, Copy)]
pub struct Active {
    pub id: CueId,
    /// Serial number of this firing, which tells a cue that GO fired again
    /// apart from one that kept running
    pub(in crate::back) run: u32,
    /// Run of the cue whose start, or whose end for an auto-follow, starts
    /// this one, or `None` for the cue that GO itself starts.  A group
    /// starts the cues that it holds directly, and the cue that follows it
    /// in the chain.  Runs of a chain are not always consecutive, since a
    /// cue that GO fires again takes a run of its own, so a cue finds the
    /// cues it starts by this.
    pub(in crate::back::workspace) after: Option<u32>,
    /// Seconds since GO, negative while the cue waits its turn in a chain
    pub(in crate::back::workspace) elapsed: f64,
    pub paused: bool,
    /// Level in dB an audio or video cue plays at.  An edit can move it
    /// below -60 dB, which sounds as silence, so that moving the level
    /// of the cue back restores what a fade had set.
    pub level: f32,
    pub fade: Option<Fade>,
    pub(super) pre_wait: f64,
    /// Seconds that the action takes after the pre-wait.  For a clip that
    /// loops this is how long the clip takes from its start to its end,
    /// without going round its loop, until a devamp cue lets it go on, and
    /// then how long the whole action takes.
    pub(super) length: f64,
    pub(super) effect: Effect,
    /// Whether the action has begun, so that a fade or stop cue acts once
    pub(super) started: bool,
}

/// Where an active cue is in its lifetime.  An audio or video cue whose
/// clip loops stays [`Phase::Looping`] until something stops it, as a
/// picture cue stays [`Phase::Showing`], unless a devamp cue lets it go on
/// from its loop, which makes it [`Phase::Running`] until its clip ends.  A
/// group is [`Phase::Endless`] where a cue that it holds never ends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Scheduled { starts_in: f64 },
    PreWait { remaining: f64 },
    Running { remaining: f64 },
    Looping,
    Showing,
    Endless,
}

impl Active {
    /// Fires a cue `delay` seconds after GO as run `run`, started by run
    /// `after` of the same chain, or by GO itself when `after` is none
    pub(super) fn new(cue: &Cue, run: u32, after: Option<u32>, delay: f64) -> Self {
        let length = f64::from(cue.action());
        let effect = match (cue.kind, cue.target) {
            (Kind::Audio, _) => cue.clip().map_or(Effect::Nothing, Effect::Sound),
            (Kind::Video, _) => cue.clip().map_or(Effect::Nothing, |clip| Effect::Video {
                clip,
                sound: cue.sounds(),
            }),
            (Kind::Picture, _) => Effect::Picture,
            (Kind::Light, _) => Effect::Light,
            (Kind::Network, _) => Effect::Send,
            (Kind::Group, _) => Effect::Group { endless: false },
            (Kind::Fade, Some(target)) => Effect::Fade {
                target,
                fade: Fade {
                    by: Some(cue.id),
                    to: cue.level,
                    remaining: length,
                    length,
                    then_stop: cue.stop_target,
                    opacity: 1.0,
                },
            },
            (Kind::Stop, Some(target)) => Effect::Stop(target),
            (Kind::Start, Some(target)) => Effect::Start(target),
            (Kind::Pause, Some(target)) => Effect::Pause(target),
            (Kind::Devamp, Some(target)) => Effect::Devamp(target),
            (Kind::GoTo, Some(target)) => Effect::GoTo(target),
            _ => Effect::Nothing,
        };
        Self {
            id: cue.id,
            run,
            after,
            elapsed: -delay,
            paused: false,
            level: cue.level,
            fade: None,
            pre_wait: cue.pre_wait.value().into(),
            length,
            effect,
            started: false,
        }
    }
}
