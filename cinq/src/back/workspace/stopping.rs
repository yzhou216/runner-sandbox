// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rules of stopping: a stopped cue, the rest of its chain, and the fades
//! it was making

use std::iter;

use super::Workspace;
use crate::back::{Active, CueId};

impl Workspace {
    /// Stops a cue.  The cues of its chain that have yet to start go too
    /// when the stopped cue would have started them, directly or through
    /// other cues that have yet to start.  Cues that wait for another cue
    /// that has started stay, since that cue starts them.
    pub(super) fn stopped(self, id: CueId) -> Self {
        self.without_followers_of(id).skipped(id)
    }

    /// Ends the run of a cue that GO fires again.  As in QLab, a run that
    /// has started ends completely, so the cues that it had yet to start
    /// are cancelled and the new run starts its own by the current
    /// settings of the cue.  A run that has yet to start is not a cue
    /// playing, so the cues that wait for it wait for whatever would have
    /// started it instead.
    pub(super) fn restarted(self, id: CueId) -> Self {
        match self.active_of(id) {
            Some(old) if !old.waiting() => self.without_followers_of(id),
            _ => self,
        }
        .without_run_of(id)
    }

    /// Cancels the cues that the run of a cue would start, directly or
    /// through other cues that have yet to start
    fn without_followers_of(self, id: CueId) -> Self {
        let cancelled = self
            .active_of(id)
            .map(|run| self.followers(run.run))
            .unwrap_or_default();
        self.without_active_where(|active| cancelled.contains(&active.run))
    }

    /// Returns the runs that the run `run` would start, directly or
    /// through other runs that have yet to start
    fn followers(&self, run: u32) -> Vec<u32> {
        self.active
            .iter()
            .filter(|next| next.after == Some(run) && next.waiting())
            .flat_map(|next| iter::once(next.run).chain(self.followers(next.run)))
            .collect()
    }

    /// Ends the run of a cue and leaves the rest of its chain to play, as GO
    /// skips a cue that cannot play.  Fades the cue was making stop where
    /// they are.
    pub(super) fn skipped(self, id: CueId) -> Self {
        self.without_run_of(id).with_each_active(|active| Active {
            fade: active.fade.filter(|fade| fade.by != Some(id)),
            ..active
        })
    }

    /// Removes the run of a cue without stopping the cues that wait for it.
    /// When the run has yet to start, they wait for whatever would have
    /// started it, so that stopping that cue still cancels them.  When it
    /// has started, they keep waiting for a run that is gone, so that no
    /// stop cancels them, as that run is what starts them.
    fn without_run_of(self, id: CueId) -> Self {
        let handover = self
            .active_of(id)
            .filter(|gone| gone.waiting())
            .map(|gone| (gone.run, gone.after));
        self.with_each_active(|active| match handover {
            Some((run, after)) if active.after == Some(run) => Active { after, ..active },
            _ => active,
        })
        .without_active_where(|active| active.id == id)
    }
}
