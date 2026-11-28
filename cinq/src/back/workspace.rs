// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Workspace, which holds the cue lists and the cues playing from them
//!
//! Playback here is bookkeeping without side effects: fired cues count
//! time, fades move levels, stop cues end other cues, start and pause cues
//! fire and hold them, and GoTo cues move standby.  The audio output in
//! [`super::Audio`] turns the result into sound, the video output in
//! [`super::Video`] into pictures, the light output in [`super::Lights`]
//! into DMX, and the network output in [`super::Network`] into messages.
//! A [`Command`] asks for each change, and an [`Active`] records each cue
//! that GO has fired.

mod checks;
mod command;
mod editing;
mod files;
mod history;
mod lookup;
mod outputs;
mod running;
mod show;
#[cfg(test)]
pub(super) mod tests;

use std::{collections::BTreeMap, iter, path::PathBuf};

pub use command::{Command, Place};
pub use history::{Change, Turn};
pub use running::active::{Active, Phase};

use crate::back::{
    CueId, CueList, Instrument, InstrumentId, Outgoing, Output, OutputId,
    items::{replaced, replaced_where},
};
pub use checks::Unlit;
use history::{History, Kept};
use running::lighting::Dimmer;
pub use running::lighting::Universe;

/// Show as a whole
///
/// A show always holds at least one cue list, which the front end shows,
/// and at least one output.
#[derive(Debug, Clone)]
pub struct Workspace {
    lists: Vec<CueList>,
    outputs: Vec<Output>,
    next_output: u32,
    patch: Vec<Instrument>,
    /// Level of each instrument that a light cue has set
    dimmers: BTreeMap<InstrumentId, Dimmer>,
    active: Vec<Active>,
    /// Messages of network cues started since the network output last took
    /// them
    outbox: Vec<Outgoing>,
    /// Lock on editing during a performance, which leaves playback alone
    show_mode: bool,
    /// Identity the next added cue receives, counted past the largest
    /// identity, so that a show whose identities run out adds no cue
    /// rather than one that shares an identity
    next_id: u64,
    next_run: u32,
    directory: PathBuf,
    history: History,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

/// Returns the identity past every identity of a cue and every target of a
/// cue, since a target whose cue is gone is kept and a show file saves it
/// as it is, so that a new cue must not take it
fn next_id(lists: &[CueList]) -> u64 {
    lists
        .iter()
        .flat_map(|list| &list.cues)
        .flat_map(|cue| iter::once(cue.id).chain(cue.target))
        .map(|id| u64::from(id.0) + 1)
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
            outputs: vec![Output::main(OutputId(1))],
            next_output: 2,
            patch: Vec::new(),
            dimmers: BTreeMap::new(),
            active: Vec::new(),
            outbox: Vec::new(),
            show_mode: false,
            next_run: 0,
            directory: PathBuf::new(),
            history: History::default(),
        }
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
            Command::Start(id) => self.started_cue(id, 0.0).tick(0.0),
            Command::Hold(id) => self.held(id),
            Command::Resume(id) => self.resumed(id),
            Command::GoTo(id) => self.moved_to(id),
            Command::Tick(seconds) => self.tick(seconds),
            Command::Standby { list, cue } => self.moved_standby(list, cue),
            Command::ShowMode(show_mode) => Self { show_mode, ..self },
            Command::Heard { cue, run, played } => self.heard(cue, run, played),
            Command::Loaded {
                cue,
                file,
                media,
                seen,
            } => self.loaded(cue, &file, media, seen),
            Command::EndRun => self.run_ended(),
            Command::Undo => self.undone(),
            Command::Redo => self.redone(),
            Command::Add {
                list,
                kind,
                at,
                count,
            } => self.added(list, kind, at, count),
            Command::Paste { list, cues, at } => self.pasted(list, cues, at),
            Command::Delete(ids) => self.deleted(&ids, false),
            Command::Cut(ids) => self.deleted(&ids, true),
            Command::Move { cue, by } => self.moved(cue, by),
            Command::Indent(ids) => self.indented(&ids),
            Command::Outdent(ids) => self.outdented(&ids),
            Command::Collapse { cues, collapsed } => self.collapsed(&cues, collapsed),
            Command::Edit(id, edit) => self.edited(id, edit),
            Command::Patch(edit) => self.patched(edit),
            Command::Restore(lists, outputs, patch) => self.restored(Kept {
                lists,
                outputs,
                patch,
            }),
            Command::AddOutput { at, count, anchor } => self.added_output(at, count, anchor),
            Command::DeleteOutputs(ids) => self.deleted_outputs(&ids, false),
            Command::CutOutputs(ids) => self.deleted_outputs(&ids, true),
            Command::PasteOutputs {
                at,
                outputs,
                restore,
                anchor,
            } => self.pasted_outputs(at, outputs, restore, anchor),
            Command::MoveOutput { output, by } => self.moved_output(output, by),
            Command::EditOutput(id, edit) => self.edited_output(id, edit),
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
