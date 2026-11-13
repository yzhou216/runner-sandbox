// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Workspace, which holds the cue lists and the cues playing from them
//!
//! Playback here is bookkeeping without side effects: fired cues count
//! time, fades move levels, and stop cues end other cues.  The audio output
//! in [`super::Audio`] turns the result into sound, the video output in
//! [`super::Video`] into pictures, and the light output in
//! [`super::Lights`] into DMX.  A [`Command`] asks for each change, and an
//! [`Active`] records each cue that GO has fired.  The rules of playback
//! live in [`playback`] and [`stopping`], those of lights in [`lighting`],
//! those that change the cue lists in [`editing`], undo and redo in
//! [`history`], show files in [`show`], the places of the files of cues in
//! [`files`], and why cues cannot play in [`problems`].

mod active;
mod command;
mod editing;
mod files;
mod history;
mod lighting;
mod playback;
mod problems;
mod show;
mod stopping;
#[cfg(test)]
pub(super) mod tests;

use std::{collections::BTreeMap, path::PathBuf};

pub use active::{Active, Phase};
pub use command::{Command, Contents, Place};
pub use history::{Change, Turn};

use crate::back::{
    Cue, CueId, CueList, Instrument, InstrumentId,
    items::{replaced, replaced_where},
};
use history::History;
use lighting::Dimmer;
pub use lighting::Universe;

/// Show as a whole, with its cue lists, light patch, running cues, and
/// editing lock
///
/// A show always holds at least one cue list, which the front end shows.
#[derive(Debug, Clone)]
pub struct Workspace {
    lists: Vec<CueList>,
    patch: Vec<Instrument>,
    /// Level of each instrument that a light cue has set
    dimmers: BTreeMap<InstrumentId, Dimmer>,
    active: Vec<Active>,
    /// Lock on editing during a performance, which leaves playback alone
    show_mode: bool,
    /// Identity the next added cue receives, counted past the largest
    /// identity, so that a show whose identities run out adds no cue
    /// rather than one that shares an identity
    next_id: u64,
    next_run: u32,
    /// Directory that relative file paths start from
    directory: PathBuf,
    history: History,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

fn next_id(lists: &[CueList]) -> u64 {
    lists
        .iter()
        .flat_map(|list| &list.cues)
        .map(|cue| u64::from(cue.id.0) + 1)
        .max()
        .unwrap_or(1)
}

fn listed(lists: Vec<CueList>) -> Vec<CueList> {
    if lists.is_empty() {
        vec![CueList::new("Main", Vec::new())]
    } else {
        lists
    }
}

impl Workspace {
    pub fn new(lists: Vec<CueList>) -> Self {
        let lists = listed(lists);
        Self {
            next_id: next_id(&lists),
            lists,
            patch: Vec::new(),
            dimmers: BTreeMap::new(),
            active: Vec::new(),
            show_mode: false,
            next_run: 0,
            directory: PathBuf::new(),
            history: History::default(),
        }
    }

    pub fn lists(&self) -> &[CueList] {
        &self.lists
    }

    /// Returns the cues that GO has fired and that have not finished yet,
    /// in the order that GO fired them, the latest last
    pub fn active(&self) -> &[Active] {
        &self.active
    }

    pub fn show_mode(&self) -> bool {
        self.show_mode
    }

    /// Returns the identity the next added cue will receive, or none once
    /// the identities have run out
    pub fn next_cue_id(&self) -> Option<CueId> {
        u32::try_from(self.next_id).ok().map(CueId)
    }

    pub fn cues(&self) -> impl Iterator<Item = &Cue> {
        self.lists.iter().flat_map(|list| &list.cues)
    }

    pub fn cue(&self, id: CueId) -> Option<&Cue> {
        self.cues().find(|cue| cue.id == id)
    }

    pub fn target(&self, cue: &Cue) -> Option<&Cue> {
        cue.target.and_then(|id| self.cue(id))
    }

    pub fn list_of(&self, id: CueId) -> Option<usize> {
        self.lists
            .iter()
            .position(|list| list.position(id).is_some())
    }

    pub fn active_of(&self, id: CueId) -> Option<&Active> {
        self.active.iter().find(|active| active.id == id)
    }

    fn active_where(&self, test: impl Fn(&Active) -> bool) -> Vec<CueId> {
        self.active
            .iter()
            .filter(|active| test(active))
            .map(|active| active.id)
            .collect()
    }

    fn without_active_where(self, test: impl Fn(&Active) -> bool) -> Self {
        Self {
            active: self
                .active
                .into_iter()
                .filter(|active| !test(active))
                .collect(),
            ..self
        }
    }

    fn with_each_active(self, change: impl FnMut(Active) -> Active) -> Self {
        Self {
            active: self.active.into_iter().map(change).collect(),
            ..self
        }
    }

    /// Returns the workspace with a command applied.  Commands that change
    /// cues do nothing in show mode, which only allows playback and moves
    /// of standby, and undo and redo there take back and make again only
    /// moves of standby.
    pub fn apply(self, command: Command) -> Self {
        match command {
            command if self.show_mode && command.edits() => self,
            Command::Go { list } => self.go(list),
            Command::Panic => self.panic(),
            Command::Pause(paused) => self.paused(paused),
            Command::Stop(id) => self.stopped(id),
            Command::Tick(seconds) => self.tick(seconds),
            Command::Standby { list, cue } => self.moved_standby(list, cue),
            Command::ShowMode(show_mode) => Self { show_mode, ..self },
            Command::Heard { cue, run, played } => self.heard(cue, run, played),
            Command::Loaded { cue, file, media } => self.loaded(cue, &file, media),
            Command::Undo => self.undone(),
            Command::Redo => self.redone(),
            Command::Add { list, kind, at } => self.added(list, kind, at),
            Command::Delete(ids) => self.deleted(&ids),
            Command::Move { cue, by } => self.moved(cue, by),
            Command::Edit(id, edit) => self.edited(id, edit),
            Command::Patch(edit) => self.patched(edit),
            Command::Restore(contents) => self.restored(contents),
        }
    }

    fn with_list(self, list: usize, change: impl FnOnce(CueList) -> CueList) -> Self {
        Self {
            lists: replaced(self.lists, list, change),
            ..self
        }
    }

    fn with_list_of(self, id: CueId, change: impl FnOnce(CueList) -> CueList) -> Self {
        match self.list_of(id) {
            Some(list) => self.with_list(list, change),
            None => self,
        }
    }

    fn with_active(self, id: CueId, change: impl FnOnce(Active) -> Active) -> Self {
        Self {
            active: replaced_where(self.active, |active| active.id == id, change),
            ..self
        }
    }
}
