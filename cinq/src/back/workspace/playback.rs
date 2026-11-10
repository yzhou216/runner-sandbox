// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of playback: GO, the clock, fades, pauses, and panics

use std::path::PathBuf;

use super::{
    Workspace,
    active::{Effect, Fade},
};
use crate::back::{Active, Cue, CueId, CueList, SILENCE};

/// Length in seconds of the fade-out that a panic starts
pub(super) const PANIC_FADE: f64 = 1.0;

impl Workspace {
    /// Returns the cues that the next GO of `list` fires, each with its
    /// delay after the GO: the chain from standby, without the cues that
    /// cannot play
    pub fn next_go<'a>(&'a self, list: &'a CueList) -> impl Iterator<Item = (&'a Cue, f32)> {
        list.standby
            .into_iter()
            .flat_map(move |standby| list.chain(standby))
            .filter(move |(cue, _)| self.problem(cue).is_none())
    }

    /// Returns the cues that the next GO of each list may start that name a
    /// file, each with its file, which the video output makes ready ahead
    pub(in crate::back) fn upcoming(&self) -> impl Iterator<Item = (&Cue, PathBuf)> {
        self.lists
            .iter()
            .flat_map(|list| self.next_go(list))
            .filter_map(|(cue, _)| Some((cue, self.path(cue)?)))
    }

    /// Fires the standby cue of `list` together with its chain, then moves
    /// standby to the first cue after the chain.  Cues in the chain that
    /// cannot play are skipped, and cues that are already running restart,
    /// which cancels the cues that their old runs had yet to start.
    pub(super) fn go(self, list: usize) -> Self {
        let Some((fired, standby)) = self.lists.get(list).and_then(|cues| {
            let end = cues.chain(cues.standby?).last()?.0.id;
            let fired: Vec<Active> = self
                .next_go(cues)
                .zip(self.next_run..)
                .map(|((cue, delay), run)| {
                    let after = (run > self.next_run).then(|| run - 1);
                    Active::new(cue, run, after, delay.into())
                })
                .collect();
            Some((fired, cues.after(end)))
        }) else {
            return self;
        };
        let next_run = fired.last().map_or(self.next_run, |active| active.run + 1);
        let workspace = fired.iter().fold(
            self.with_standby(list, standby).played_from(list),
            |workspace, cue| workspace.restarted(cue.id),
        );
        Self {
            active: workspace.active.into_iter().chain(fired).collect(),
            next_run,
            ..workspace
        }
        .tick(0.0)
    }

    pub(super) fn tick(self, seconds: f64) -> Self {
        self.with_each_active(|active| active.advanced(seconds))
            .started()
            .without_finished()
    }

    pub(super) fn heard(self, cue: CueId, run: u32, played: f64) -> Self {
        self.with_active(cue, |active| {
            if active.run == run {
                active.heard(played)
            } else {
                active
            }
        })
    }

    pub(super) fn panic(self) -> Self {
        let fading = self
            .active
            .iter()
            .any(|active| active.fade.is_some_and(|fade| fade.by.is_none()));
        let fade = Fade {
            by: None,
            to: SILENCE,
            remaining: PANIC_FADE,
            length: PANIC_FADE,
            then_stop: true,
            opacity: 1.0,
        };
        self.without_active_where(|active| fading || active.paused || !active.playing())
            .with_each_active(|active| active.under(fade))
    }

    pub(super) fn paused(self, paused: bool) -> Self {
        self.with_each_active(|active| Active { paused, ..active })
    }

    /// Begins the action of every cue whose pre-wait is over.  Audio, video,
    /// and picture cues need nothing here, since making sound and pictures
    /// is the job of the audio and video output.
    fn started(self) -> Self {
        let due = self.active_where(Active::due);
        due.into_iter()
            .fold(self.with_each_active(Active::begun), Self::with_effect_of)
    }

    /// Carries out the effect of a fade or stop cue on its target, unless
    /// an effect earlier in the same tick has stopped the cue.  A fade acts
    /// only on a target that is playing, takes over from a fade already on
    /// it only where [`Fade::yields_to`] allows, and starts as far along as
    /// the tick has passed the end of the pre-wait.
    fn with_effect_of(self, id: CueId) -> Self {
        let Some(fired) = self.active_of(id).copied() else {
            return self;
        };
        match fired.effect {
            Effect::Fade { target, fade } => self.with_active(target, |active| {
                if active.playing() && active.fade.is_none_or(|current| current.yields_to(&fade)) {
                    active.under(fade).faded(fired.played())
                } else {
                    active
                }
            }),
            Effect::Stop(target) => self.stopped(target),
            Effect::Sound(_) | Effect::Video { .. } | Effect::Picture | Effect::Nothing => self,
        }
    }

    /// Drops the cues that have finished.  A cue that a fade has faded out
    /// stops as a stopped cue does, which also cancels the rest of its
    /// chain that it has yet to start.
    fn without_finished(self) -> Self {
        let faded_out = self.active_where(Active::faded_out);
        faded_out
            .into_iter()
            .fold(self, Self::stopped)
            .without_active_where(Active::finished)
    }
}
