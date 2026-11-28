// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of control cues: a start cue fires or resumes its target, a pause
//! cue holds it, a devamp cue lets it leave its loop, and a GoTo cue moves
//! standby to it

use std::iter;

use itertools::Itertools;

use super::active::Effect;
use crate::back::{Active, Cue, CueId, Kind, Media, Workspace};

/// Fewest seconds before the end of a pass through its loop that a devamp
/// cue has to act for its target to leave the loop there, rather than at
/// the end of the next pass.  A sound that has decoded the start of its next
/// pass plays that pass, so the notice also has to outlast what the audio
/// output decodes ahead of the sound, by [`MARGIN`] seconds.
const RELEASE_NOTICE: f64 = 0.5;

/// Seconds that a devamp cue acts before the decoder of its target can have
/// reached the end of the pass
const MARGIN: f64 = 0.15;

impl Workspace {
    /// Carries out a start cue.  A target that is paused resumes together
    /// with the cues that it would start, and nothing else fires.  Any other
    /// target fires with its chain as GO fires the chain, in the list of the
    /// target and `offset` seconds along, leaving standby and the history
    /// of moves alone.  The cues that fire begin their actions on the next
    /// tick, so that start cues that start each other go round once a tick.
    pub(in crate::back::workspace) fn started_cue(self, target: CueId, offset: f64) -> Self {
        let paused = self.active_of(target).is_some_and(|run| run.paused);
        match self.list_of(target) {
            _ if paused => self.resumed(target),
            Some(list) => self.fired(list, target, offset),
            None => self,
        }
    }

    /// Resumes the run of `target`, with the runs that it would start
    pub(in crate::back::workspace) fn resumed(self, target: CueId) -> Self {
        self.with_pause_of(target, false)
    }

    /// Returns the cues that a start cue on `target` fires, which are the
    /// chain of the target and those that its start cues fire in turn, each
    /// with the seconds until it fires
    pub fn firing(&self, target: CueId) -> Vec<(&Cue, f32)> {
        self.reached(self.chain_of(target).collect())
    }

    /// Carries out a pause cue, which holds the run of its target, whatever
    /// it is doing, and the cues that the run would start, so that the chain
    /// stays where it is.  A target with no run is left alone.
    pub(in crate::back::workspace) fn held(self, target: CueId) -> Self {
        self.with_pause_of(target, true)
    }

    /// Carries out a devamp cue, which lets the run of its target leave its
    /// loop at the end of the first pass that ends more than
    /// [`RELEASE_NOTICE`] seconds after where the run is, or more than the
    /// seconds that its sound decodes ahead, with a margin, when that is
    /// longer, paused or not, and play on to the end of its clip.  A run
    /// that a devamp cue has let go on already keeps the pass that it leaves
    /// at.
    pub(super) fn devamped(self, target: CueId) -> Self {
        let ahead = match self.cue(target).map(|cue| &cue.media) {
            Some(&Media::Loaded { ahead, .. }) => f64::from(ahead),
            _ => 0.0,
        };
        let notice = RELEASE_NOTICE.max(ahead + MARGIN);
        self.with_active(target, |active| active.released(notice))
    }

    /// Carries out a GoTo cue, which moves the standby of the list that
    /// holds its target to the target, as GO moves standby: undo does not
    /// take the move back.
    pub(in crate::back::workspace) fn moved_to(self, target: CueId) -> Self {
        match self.list_of(target) {
            Some(list) => self.with_standby(list, Some(target)).played_from(list),
            None => self,
        }
    }

    /// Returns the cue that the first GoTo cue to act among those that have
    /// fired is to put on standby of `list`, with the seconds until it does,
    /// so that the standby shown there is known to be about to move.  A GoTo
    /// cue that is paused, has acted, or is stopped has none.
    pub fn moving(&self, list: usize) -> Option<(&Cue, f64)> {
        self.active
            .iter()
            .filter_map(|active| match active.effect {
                Effect::GoTo(target) => Some((self.cue(target)?, active.acts_in()?)),
                _ => None,
            })
            .filter(|(target, _)| self.list_of(target.id) == Some(list))
            .min_by(|(_, first), (_, second)| first.total_cmp(second))
    }

    /// Returns the cues of `fired`, each with the seconds until it fires,
    /// together with those that the start cues among them fire, and those
    /// that the start cues among these fire in turn.  A start cue whose
    /// target is paused fires nothing, since it resumes the target.  A cue
    /// that a later round reaches sooner than an earlier one did takes the
    /// sooner delay and keeps its place, so that start cues that start each
    /// other end the search, as delays only fall.
    pub fn reached<'a>(&'a self, fired: Vec<(&'a Cue, f32)>) -> Vec<(&'a Cue, f32)> {
        let found: Vec<(&Cue, f32)> = fired
            .iter()
            .filter(|(cue, _)| cue.kind == Kind::Start)
            .filter_map(|&(cue, delay)| Some((self.target(cue)?, delay + cue.pre_wait.value())))
            .filter(|(target, _)| !self.active_of(target.id).is_some_and(|run| run.paused))
            .flat_map(|(target, acts)| {
                self.chain_of(target.id)
                    .map(move |(cue, delay)| (cue, acts + delay))
            })
            .filter(|(cue, delay)| {
                fired
                    .iter()
                    .all(|(old, sooner)| old.id != cue.id || delay < sooner)
            })
            .collect();
        // A cue that several start cues reach fires at the soonest of them
        let started: Vec<(&Cue, f32)> = found
            .iter()
            .unique_by(|(cue, _)| cue.id)
            .map(|&(cue, _)| {
                let soonest = found
                    .iter()
                    .filter(|(other, _)| other.id == cue.id)
                    .map(|&(_, delay)| delay)
                    .fold(f32::INFINITY, f32::min);
                (cue, soonest)
            })
            .collect();
        if started.is_empty() {
            return fired;
        }
        let sooner = |cue: &Cue| started.iter().find(|(other, _)| other.id == cue.id);
        let kept = fired
            .iter()
            .map(|&(cue, delay)| (cue, sooner(cue).map_or(delay, |&(_, soonest)| soonest)));
        let added = started
            .iter()
            .filter(|(cue, _)| fired.iter().all(|(old, _)| old.id != cue.id))
            .copied();
        self.reached(kept.chain(added).collect())
    }

    /// Returns the cues that may fire without the operator, through the
    /// start cues that have fired and have yet to act, or with the next GO
    /// of a list, through its start cues as well, each with the seconds
    /// until it fires, from now or from that GO, so that the audio and video
    /// outputs can make them ready ahead
    pub(in crate::back) fn ahead(&self) -> Vec<(&Cue, f32)> {
        let starting = self
            .active
            .iter()
            .filter(|active| !active.started)
            .filter_map(|active| {
                let cue = self.cue(active.id).filter(|cue| cue.kind == Kind::Start)?;
                Some((cue, (-active.elapsed) as f32))
            })
            .collect();
        let next = self
            .lists
            .iter()
            .flat_map(|list| self.reached(self.next_go(list).collect()));
        self.reached(starting).into_iter().chain(next).collect()
    }

    /// Returns the chain that a start cue on `target` fires, without the
    /// cues that cannot play
    fn chain_of(&self, target: CueId) -> impl Iterator<Item = (&Cue, f32)> {
        self.list_holding(target)
            .into_iter()
            .flat_map(move |list| self.chain_from(list, target))
    }

    /// Holds or resumes the run of a cue with the runs that it would start,
    /// and for a group those of the cues that it holds as well
    fn with_pause_of(self, id: CueId, paused: bool) -> Self {
        let held: Vec<u32> = self
            .with_members(id)
            .into_iter()
            .filter_map(|member| self.active_of(member))
            .flat_map(|run| iter::once(run.run).chain(self.followers(run.run)))
            .collect();
        self.with_each_active(|active| {
            if held.contains(&active.run) {
                Active { paused, ..active }
            } else {
                active
            }
        })
    }
}
