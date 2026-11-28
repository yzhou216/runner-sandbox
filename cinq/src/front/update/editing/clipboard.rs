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
//! those are.  Their sends go by the names of the outputs that they had
//! there, since the identity of an output is its place in a show file.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::{
    back::{Command, Cue, CueId, Output, OutputId, Place, Sends, Unlit},
    front::{App, Notices, view::format},
};

/// Most cues that one paste or one add adds, even of a clipboard that holds
/// more, which keeps a count typed by mistake from filling the memory
pub(super) const PASTE_LIMIT: usize = 1000;

/// What the status bar tells when a paste finds nothing to paste, as Vim
/// does with its E353
pub(in crate::front::update) const EMPTY: &str = "The clipboard is empty";

/// What the latest copy or cut told, which the status bar shows until the
/// operator does anything else
#[derive(Debug, Clone, PartialEq)]
pub struct Clipped {
    pub told: String,
    /// Whether the notice tells a change that undo takes back, a cut or a
    /// paste, so that the status bar hints the key of undo
    pub cut: bool,
}

#[derive(Default)]
pub struct Clipboard {
    /// Serial number of the opening that brought the show the cues came
    /// from
    pub(in crate::front::update) show: u64,
    pub(in crate::front::update) cues: Vec<Cue>,
    /// Names of the outputs of that show when the cues were copied, by
    /// identity, which the sends of the cues name
    pub(in crate::front::update) output_names: BTreeMap<OutputId, String>,
    /// Outputs copied or cut, in a register of their own, so that copying
    /// an output keeps the cues that were copied
    pub(in crate::front::update) outputs: Register,
}

/// Outputs that were copied or cut last
#[derive(Default)]
pub struct Register {
    /// Serial number of the opening that brought the show they came from
    pub(in crate::front::update) show: u64,
    /// Whether they left the show, so that pasting them back restores them
    pub(in crate::front::update) cut: bool,
    pub(in crate::front::update) outputs: Vec<Output>,
}

impl App {
    /// Copies the selected row and the rows below it, as many as `count`
    /// holds, in place of what the clipboard held, each group with the cues
    /// that it holds, as a closed fold is yanked whole.  Copying changes no
    /// show, so that show mode leaves it, as Vim yanks in a read-only buffer.
    pub(in crate::front::update) fn with_copied(self, count: usize) -> Self {
        self.clipped(count, "Copied", false)
    }

    /// Fills the clipboard as [`App::with_copied`] does, and tells what went
    /// there, by the `verb` that the status bar starts with, and whether a
    /// `cut` takes the cues out of the show
    fn clipped(self, count: usize, verb: &str, cut: bool) -> Self {
        let Some(first) = self.selected_row() else {
            return self;
        };
        let list = self.current();
        let groups = list.groups();
        // A group takes the cues that it holds along, once even where the
        // count reaches them as well
        let rows: BTreeSet<usize> = self
            .shown_cues_from(first, count)
            .into_iter()
            .flat_map(|row| groups.extent(row))
            .collect();
        let cues: Vec<Cue> = rows
            .into_iter()
            .map(|row| self.workspace.copied(&list.cues[row]))
            .collect();
        let clipped = Clipped {
            told: format::clipped(verb, &cues),
            cut,
        };
        Self {
            clipboard: Clipboard {
                show: self.show.id(),
                cues,
                output_names: self
                    .workspace
                    .outputs()
                    .iter()
                    .map(|output| (output.id, output.name.clone()))
                    .collect(),
                ..self.clipboard
            },
            notices: self.notices.with_clipped(Some(clipped)),
            ..self
        }
    }

    /// Cuts as [`App::with_copied`] copies, and deletes the cues copied.
    /// Show mode locks it as it does editing, so that it copies nothing
    /// either.
    pub(in crate::front::update) fn cut(self, count: usize) -> Self {
        if !self.editable() {
            return self;
        }
        let app = self
            .clipped(count, "Cut", true)
            .taking_away(count, Command::Cut);
        Self {
            notices: Notices {
                clipped: app.notices.clipped.map(|clipped| Clipped {
                    told: format::stopping(
                        clipped.told,
                        app.clipboard.cues.len(),
                        app.notices.stopped.len(),
                    ),
                    ..clipped
                }),
                ..app.notices
            },
            ..app
        }
    }

    /// Pastes the clipboard `count` times in a row next to the selection,
    /// on the side that `place` picks, or at the end of the list without a
    /// selection, and selects the first cue pasted, as far as
    /// [`PASTE_LIMIT`] allows.  An empty clipboard pastes nothing and tells
    /// so.  Another show keeps no target of the cues but those that are
    /// copied with it, which the paste points at their new copies.  There,
    /// each cue sends to the outputs that bear the names of those it sent
    /// to, without a send whose output has no namesake, and a cue left with
    /// no send plays on the first output, as a cue without sends does.  The
    /// notice also counts the light cues there that name instruments which
    /// the patch lacks.
    pub(in crate::front::update) fn with_pasted(
        self,
        place: fn(CueId) -> Place,
        count: usize,
    ) -> Self {
        let copies = &self.clipboard.cues;
        let elsewhere = self.clipboard.show != self.show.id();
        let copied_ids: HashSet<CueId> = copies.iter().map(|cue| cue.id).collect();
        let (cues, lost): (Vec<Cue>, Vec<(usize, usize)>) = (0..count.min(PASTE_LIMIT))
            .flat_map(|_| copies.iter().cloned())
            .take(PASTE_LIMIT)
            .map(|cue| self.adapted(cue, elsewhere, &copied_ids))
            .unzip();
        if cues.is_empty() {
            return Self {
                notices: self.notices.with_remark(Some(EMPTY)),
                ..self
            };
        }
        let told = self.paste_notice(&cues, &lost, elsewhere);
        let first = self.workspace.next_cue_id();
        let paste = Command::Paste {
            list: self.list,
            cues,
            at: self.selected_here().map_or(Place::End, place),
        };
        let app = self.send(paste).showing(first, None);
        Self {
            notices: app
                .notices
                .with_clipped(told.map(|told| Clipped { told, cut: true })),
            ..app
        }
    }

    /// Returns `cue` as it pastes into this show, with the number of sends
    /// and of targets that it lost on the way, `elsewhere` telling that the
    /// clipboard comes from another show, of which `copied` holds the cues
    fn adapted(&self, cue: Cue, elsewhere: bool, copied: &HashSet<CueId>) -> (Cue, (usize, usize)) {
        let target = cue
            .target
            .filter(|target| !elsewhere || copied.contains(target));
        let had = cue.sends.as_ref().map_or(0, Sends::len);
        let sends = match cue.sends {
            Some(sends) if elsewhere && !sends.is_empty() => {
                Some(self.sends_by_name(sends)).filter(|named| !named.is_empty())
            }
            sends => sends,
        };
        let lost = (
            had.saturating_sub(sends.as_ref().map_or(0, Sends::len)),
            usize::from(cue.target.is_some() && target.is_none()),
        );
        (
            Cue {
                target,
                sends,
                ..cue
            },
            lost,
        )
    }

    /// Returns what the status bar tells of a paste of `cues`, whose
    /// `lost` holds the sends and targets that each of them lost
    fn paste_notice(
        &self,
        cues: &[Cue],
        lost: &[(usize, usize)],
        elsewhere: bool,
    ) -> Option<String> {
        format::pasted_losses(
            cues.len(),
            lost.iter().map(|lost| lost.0).sum(),
            lost.iter().map(|lost| lost.1).sum(),
            cues.iter()
                .filter(|cue| {
                    elsewhere && matches!(self.workspace.unlit(cue), Some(Unlit::Unpatched(_)))
                })
                .count(),
        )
    }

    /// Returns `sends` of a cue of the show of the clipboard as they go to
    /// the outputs of this show of the same names, which are unique there
    fn sends_by_name(&self, sends: Sends) -> Sends {
        sends
            .into_iter()
            .filter_map(|(id, level)| {
                let name = self.clipboard.output_names.get(&id)?;
                let output = self
                    .workspace
                    .outputs()
                    .iter()
                    .find(|output| output.name == *name)?;
                Some((output.id, level))
            })
            .collect()
    }
}
