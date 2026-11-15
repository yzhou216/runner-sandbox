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

use crate::{
    back::{Command, Cue, CueId, Place},
    front::App,
};

/// Most cues that one paste adds, which keeps a count typed by mistake from
/// filling the memory
const PASTE_LIMIT: usize = 1000;

#[derive(Default)]
pub struct Clipboard {
    /// Serial number of the opening that brought the show the cues came
    /// from
    show: u64,
    cues: Vec<Cue>,
}

impl App {
    /// Copies the selected cue and the cues below it, as many as `count`
    /// holds, in place of what the clipboard held.  Show mode locks it as it
    /// does editing.
    pub(super) fn with_copied(self, count: usize) -> Self {
        let Some(first) = self.selected_row().filter(|_| self.editable()) else {
            return self;
        };
        let cues = self.current().cues[first..]
            .iter()
            .take(count)
            .map(|cue| self.workspace.copied(cue))
            .collect();
        Self {
            clipboard: Clipboard {
                show: self.show.id(),
                cues,
            },
            ..self
        }
    }

    pub(super) fn cut(self, count: usize) -> Self {
        self.with_copied(count).without_selected(count)
    }

    /// Pastes the clipboard `count` times in a row next to the selection,
    /// on the side that `place` picks, or at the end of the list without a
    /// selection, and selects the first cue pasted.  An empty clipboard
    /// pastes nothing.
    pub(super) fn with_pasted(self, place: fn(CueId) -> Place, count: usize) -> Self {
        let copies = &self.clipboard.cues;
        let times = count.min((PASTE_LIMIT / copies.len().max(1)).max(1));
        let elsewhere = self.clipboard.show != self.show.id();
        let cues: Vec<Cue> = (0..times)
            .flat_map(|_| copies.iter().cloned())
            .map(|cue| Cue {
                target: cue.target.filter(|_| !elsewhere),
                ..cue
            })
            .collect();
        if cues.is_empty() {
            return self;
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
