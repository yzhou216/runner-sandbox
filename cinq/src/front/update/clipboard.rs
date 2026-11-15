// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Copying, cutting, and pasting cues
//!
//! The clipboard is cinq's own, apart from the system clipboard, and lives
//! in the interface state, so that it is neither part of the show nor of
//! what undo takes back.  It outlasts the show that it was filled from:
//! copies hold their files by absolute path, which pasting turns into the
//! path from the directory of the show that takes them, and the cues that
//! they target are kept only in the show they came from, whose identities
//! those are.

use std::collections::HashSet;

use crate::{
    back::{Command, Cue, CueId, Output, Place},
    front::{App, view::format},
};

/// Most cues that one paste or one add adds, even of a clipboard that holds
/// more, which keeps a count typed by mistake from filling the memory
pub(super) const PASTE_LIMIT: usize = 1000;

/// What the status bar tells when a paste finds nothing to paste, as Vim
/// does with its E353
pub(super) const EMPTY: &str = "The clipboard is empty";

/// What the latest copy or cut told, which the status bar shows until the
/// operator does anything else
#[derive(Debug, Clone, PartialEq)]
pub struct Clipped {
    pub told: String,
    /// Whether the cues left the show, so that undo brings them back
    pub cut: bool,
}

#[derive(Default)]
pub struct Clipboard {
    /// Serial number of the opening that brought the show the cues came
    /// from
    show: u64,
    cues: Vec<Cue>,
    /// Outputs copied or cut, in a register of their own, so that copying
    /// an output keeps the cues that were copied
    pub(super) outputs: Register,
}

impl Clipboard {
    /// Returns the clipboard with `outputs` in the register of outputs, and
    /// the cues as they were
    pub(super) fn with_outputs(self, outputs: Register) -> Self {
        Self { outputs, ..self }
    }
}

/// Outputs that were copied or cut last
#[derive(Default)]
pub struct Register {
    /// Serial number of the opening that brought the show they came from
    pub(super) show: u64,
    /// Whether they left the show, so that pasting them back restores them
    pub(super) cut: bool,
    pub(super) outputs: Vec<Output>,
}

impl App {
    /// Copies the selected cue and the cues below it, as many as `count`
    /// holds, in place of what the clipboard held.  Copying changes no show,
    /// so that show mode leaves it, as Vim yanks in a read-only buffer.
    pub(super) fn with_copied(self, count: usize) -> Self {
        self.clipped(count, "Copied", false)
    }

    /// Fills the clipboard as [`App::with_copied`] does, and tells what went
    /// there, by the `verb` that the status bar starts with, and whether a
    /// `cut` takes the cues out of the show
    fn clipped(self, count: usize, verb: &str, cut: bool) -> Self {
        let Some(first) = self.selected_row() else {
            return self;
        };
        let cues: Vec<Cue> = self.current().cues[first..]
            .iter()
            .take(count)
            .map(|cue| self.workspace.copied(cue))
            .collect();
        let clipped = Clipped {
            told: format::clipped(verb, &cues),
            cut,
        };
        Self {
            clipboard: Clipboard {
                show: self.show.id(),
                cues,
                ..self.clipboard
            },
            clipped: Some(clipped),
            ..self
        }
    }

    /// Cuts as [`App::with_copied`] copies, and deletes the cues copied.
    /// Show mode locks it as it does editing, so that it copies nothing
    /// either.
    pub(super) fn cut(self, count: usize) -> Self {
        if self.editable() {
            self.clipped(count, "Cut", true)
                .taking_away(count, Command::Cut)
        } else {
            self
        }
    }

    /// Pastes the clipboard `count` times in a row next to the selection,
    /// on the side that `place` picks, or at the end of the list without a
    /// selection, and selects the first cue pasted, as far as
    /// [`PASTE_LIMIT`] allows.  An empty clipboard pastes nothing and tells so.  Another
    /// show keeps no target of the cues but those that are copied with it,
    /// which the paste points at their new copies.
    pub(super) fn with_pasted(self, place: fn(CueId) -> Place, count: usize) -> Self {
        let copies = &self.clipboard.cues;
        let elsewhere = self.clipboard.show != self.show.id();
        let copied: HashSet<CueId> = copies.iter().map(|cue| cue.id).collect();
        let cues: Vec<Cue> = (0..count.min(PASTE_LIMIT))
            .flat_map(|_| copies.iter().cloned())
            .take(PASTE_LIMIT)
            .map(|cue| Cue {
                target: cue
                    .target
                    .filter(|target| !elsewhere || copied.contains(target)),
                ..cue
            })
            .collect();
        if cues.is_empty() {
            return Self {
                remark: Some(EMPTY),
                ..self
            };
        }
        let first = self.workspace.next_cue_id();
        let paste = Command::Paste {
            list: self.list,
            cues,
            at: self.selected_here().map_or(Place::End, place),
        };
        self.send(paste).showing(first, None)
    }
}
