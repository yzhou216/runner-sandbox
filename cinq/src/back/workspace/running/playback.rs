// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of playback: GO, the clock, fades, pauses, and panics

use std::path::PathBuf;

use super::{
    active::{Effect, Fade},
    group_runs::started_by,
};
use crate::back::{Active, Cue, CueId, CueList, SILENCE, Workspace};

/// Length in seconds of the fade-out that a panic starts
pub(in crate::back::workspace) const PANIC_FADE: f64 = 1.0;

impl Workspace {
    /// Returns the cues that the next GO of `list` fires, each with its
    /// delay after the GO: the chain from standby, without the cues that
    /// cannot play
    pub fn next_go<'a>(&'a self, list: &'a CueList) -> impl Iterator<Item = (&'a Cue, f32)> {
        list.standby
            .into_iter()
            .flat_map(move |standby| self.chain_from(list, standby))
    }

    /// Returns the chain that fires from `start`, without the cues that
    /// cannot play
    pub(super) fn chain_from<'a>(
        &'a self,
        list: &'a CueList,
        start: CueId,
    ) -> impl Iterator<Item = (&'a Cue, f32)> {
        list.chain(start)
            .filter(move |(cue, _)| self.problem(cue).is_none())
    }

    /// Returns the cues of [`Workspace::ahead`] that name a file, each with
    /// its file, which the video output makes ready ahead
    pub(in crate::back) fn upcoming(&self) -> impl Iterator<Item = (&Cue, PathBuf)> {
        self.ahead()
            .into_iter()
            .filter_map(|(cue, _)| Some((cue, self.path(cue)?)))
    }

    /// Fires the standby cue of `list` together with its chain, then moves
    /// standby to the first cue after the chain.  Cues in the chain that
    /// cannot play are skipped, and cues that are already running restart,
    /// which cancels the cues that their old runs had yet to start.
    pub(in crate::back::workspace) fn go(self, list: usize) -> Self {
        let Some((start, standby)) = self.lists.get(list).and_then(|cues| {
            let start = cues.standby?;
            let end = cues.chain(start).last()?.0.id;
            Some((start, cues.after(end)))
        }) else {
            return self;
        };
        self.with_standby(list, standby)
            .played_from(list)
            .fired(list, start, 0.0)
            .tick(0.0)
    }

    /// Fires the chain that starts at `start` in `list`, each cue started
    /// by the run that [`started_by`] tells, as runs `offset` seconds along.
    /// Cues that are already running restart.
    pub(super) fn fired(self, list: usize, start: CueId, offset: f64) -> Self {
        let fired: Vec<Active> = self
            .lists
            .get(list)
            .into_iter()
            .flat_map(|cues| {
                let groups = cues.groups();
                let chain: Vec<(&Cue, f32, u32)> = self
                    .chain_from(cues, start)
                    .zip(self.next_run..)
                    .map(|((cue, delay), run)| (cue, delay, run))
                    .collect();
                let runs: Vec<(CueId, u32)> =
                    chain.iter().map(|&(cue, _, run)| (cue.id, run)).collect();
                chain
                    .into_iter()
                    .zip(started_by(&groups, &runs))
                    .map(|((cue, delay, run), after)| {
                        Active::new(cue, run, after, delay.into())
                            .spanning(groups.action_of(cue).into(), groups.endless(cue))
                            .advanced(offset)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let next_run = fired.last().map_or(self.next_run, |active| active.run + 1);
        let workspace = fired
            .iter()
            .fold(self, |workspace, cue| workspace.restarted(cue.id));
        Self {
            active: workspace.active.into_iter().chain(fired).collect(),
            next_run,
            ..workspace
        }
    }

    pub(in crate::back::workspace) fn tick(self, seconds: f64) -> Self {
        let held = self.active_where(|active| active.paused);
        self.with_each_active(|active| active.advanced_unless_held(seconds, &held))
            .started()
            .dimmed()
            .without_finished()
    }

    /// Moves the clock of the run `run` of a cue toward the time its sound
    /// has reached, and those of the runs that it would start by as much,
    /// so that its chain stays with what is heard.
    pub(in crate::back::workspace) fn heard(self, cue: CueId, run: u32, played: f64) -> Self {
        let Some(by) = self
            .active_of(cue)
            .filter(|active| active.run == run)
            .map(|active| active.drift(played))
        else {
            return self;
        };
        let following = self.followers(run);
        self.with_each_active(|active| {
            if active.run == run || following.contains(&active.run) {
                active.shifted(by)
            } else {
                active
            }
        })
    }

    pub(in crate::back::workspace) fn panic(self) -> Self {
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
        // A group that has begun lasts while the cues it holds fade out, and
        // one that waits out its pre-wait goes as any waiting cue does
        self.without_active_where(|active| {
            fading || active.paused || !(active.playing() || (active.holds() && active.started))
        })
        .with_each_active(|active| {
            if active.holds() {
                active
            } else {
                active.under(fade)
            }
        })
        .without_ended_groups()
    }

    pub(in crate::back::workspace) fn paused(self, paused: bool) -> Self {
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

    /// Carries out the effect of cue `id`, unless an effect earlier in the
    /// same tick has stopped the cue or a start cue has fired it again,
    /// which leaves its action to the next tick.  A fade acts only on a
    /// target that is playing, takes over from a fade already on it only
    /// where [`Fade::yields_to`] allows, and starts as far along as the
    /// tick has passed the end of the pre-wait.
    fn with_effect_of(self, id: CueId) -> Self {
        let Some(fired) = self.active_of(id).filter(|active| active.started).copied() else {
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
            Effect::Start(target) => self.started_cue(target, fired.played()),
            Effect::Pause(target) => self.held(target),
            Effect::Devamp(target) => self.devamped(target),
            Effect::GoTo(target) => self.moved_to(target),
            Effect::Light => self.lit(fired.id, fired.run),
            Effect::Send => self.sent_by(fired.id),
            Effect::Sound(_)
            | Effect::Video { .. }
            | Effect::Picture
            | Effect::Group { .. }
            | Effect::Nothing => self,
        }
    }

    /// Drops the cues that have finished, and the groups that hold no cue
    /// with a run.  A cue that a fade has faded out stops as a stopped cue
    /// does, which also cancels the rest of its chain that it has yet to
    /// start.
    fn without_finished(self) -> Self {
        let faded_out = self.active_where(Active::faded_out);
        faded_out
            .into_iter()
            .fold(self, Self::stopped)
            .without_active_where(Active::finished)
            .without_ended_groups()
    }
}
