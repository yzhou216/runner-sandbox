// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes to the cue lists: standby, loading, and the edits that show
//! mode locks, which [`super::history`] records for undo

use itertools::Itertools;

use super::{
    Workspace,
    history::{Change, Kept, Placed, Standby},
    numbering::Numbers,
};
use crate::back::{Active, CueId, CueList, Edit, Kind, Media, Output, Place};

impl Workspace {
    /// Puts `standby` on standby in a list without recording the move, as
    /// GO moves it
    pub(super) fn with_standby(self, list: usize, standby: Option<CueId>) -> Self {
        self.with_list(list, |cues| cues.with_standby(standby))
    }

    /// Moves standby as the operator asks, recording the move unless
    /// standby is there already
    pub(super) fn moved_standby(self, list: usize, cue: Option<CueId>) -> Self {
        let before = self.lists.get(list).map(|cues| cues.standby);
        let workspace = self.with_standby(list, cue);
        let after = workspace.lists.get(list).map(|cues| cues.standby);
        match before.zip(after) {
            Some((before, after)) if before != after => {
                workspace.recorded(Change::Standby(Standby {
                    list,
                    before,
                    after,
                }))
            }
            _ => workspace,
        }
    }

    /// Adds cues with the next identities, each after the one before and
    /// numbered after the highest cue of any list, and records them as one
    /// change at the place of the first, with the standby of their list before
    /// and after, which the first of them can take.  Once the identities have
    /// run out, no more cues are added.
    pub(super) fn added(self, list: usize, kind: Kind, at: Place, count: usize) -> Self {
        let ids: Vec<CueId> = (self.next_id..)
            .take(count)
            .map_while(|id| u32::try_from(id).ok().map(CueId))
            .collect();
        let before = self.lists.get(list).and_then(|cues| cues.standby);
        let sends = self.new_sends().filter(|_| kind.plays_file());
        let numbers = Numbers::of(self.cues().map(|cue| cue.number.as_str()));
        let (workspace, ..) =
            ids.iter()
                .fold((self, at, numbers), |(workspace, at, mut numbers), &id| {
                    let number = numbers.next();
                    numbers.take(&number);
                    let added = Self {
                        next_id: workspace.next_id + 1,
                        ..workspace.with_list(list, |cues| cues.with_new_cue(id, number, kind, at))
                    }
                    .changed(id, Edit::Sends(sends.clone()));
                    (added, Place::After(id), numbers)
                });
        let cues: Vec<Placed> = ids
            .iter()
            .filter_map(|&id| workspace.place_of(id))
            .collect();
        if cues.is_empty() {
            return workspace;
        }
        let standby = Standby {
            list,
            before,
            after: workspace.lists[list].standby,
        };
        workspace.recorded(Change::Added { cues, at, standby })
    }

    /// Deletes cues as one change, with the places they had and where
    /// standby was before and after, and whether they were cut
    pub(super) fn deleted(self, ids: &[CueId], cut: bool) -> Self {
        // Cues in the order of their places, which is the order undo puts
        // them back in
        let cues: Vec<Placed> = ids
            .iter()
            .unique()
            .filter_map(|&id| self.place_of(id))
            .sorted_by_key(|placed| (placed.list, placed.index))
            .collect();
        if cues.is_empty() {
            return self;
        }
        // Standby of each list that loses a cue, before it does
        let before: Vec<(usize, Option<CueId>)> = cues
            .iter()
            .map(|placed| placed.list)
            .dedup()
            .map(|list| (list, self.lists[list].standby))
            .collect();
        let workspace = self.removed(cues.iter().map(|placed| placed.cue.id));
        let standby = before
            .into_iter()
            .map(|(list, before)| Standby {
                list,
                before,
                after: workspace.lists[list].standby,
            })
            .collect();
        workspace.recorded(Change::Deleted { cues, standby, cut })
    }

    pub(super) fn moved(self, id: CueId, by: isize) -> Self {
        let place = self
            .lists
            .iter()
            .find_map(|cues| cues.position(id).map(|from| (from, cues.cues.len() - 1)));
        let Some((from, last)) = place else {
            return self;
        };
        let to = from.saturating_add_signed(by).min(last);
        if to == from {
            return self;
        }
        self.with_cue_moved(id, to)
            .recorded(Change::Moved { cue: id, from, to })
    }

    /// Moves a cue to the place `to` in its list without recording the
    /// move, as undo and redo move it
    pub(super) fn with_cue_moved(self, id: CueId, to: usize) -> Self {
        self.with_list_of(id, |cues| cues.with_moved(id, to))
    }

    /// Edits a cue and records the edit, unless it leaves the setting as it
    /// was, when it changes nothing at all.  Text fields send their text
    /// again on keys that change nothing, such as Backspace at the start of
    /// a field, and such an edit must not stop a running cue.
    pub(super) fn edited(self, id: CueId, edit: Edit) -> Self {
        let undo = self
            .cue(id)
            .map(|cue| cue.reverting(&edit))
            .filter(|undo| *undo != edit);
        match undo {
            Some(undo) => self.changed(id, edit.clone()).recorded(Change::Edited {
                cue: id,
                undo,
                redo: edit,
            }),
            None => self,
        }
    }

    pub(super) fn restored(self, lists: Vec<CueList>, outputs: Vec<Output>) -> Self {
        let before = Kept {
            lists: self.lists.clone(),
            outputs: self.outputs.clone(),
        };
        let after = Kept { lists, outputs };
        self.with_kept(after.clone())
            .recorded(Change::Restored { before, after })
    }

    /// Puts kept lists and outputs in place of those of the show without
    /// recording a change, as undo and redo do.  Kept lists without
    /// outputs leave the outputs as they are, since a show keeps one.
    pub(super) fn with_kept(self, kept: Kept) -> Self {
        let workspace = self.with_lists(kept.lists);
        if kept.outputs.is_empty() {
            workspace
        } else {
            workspace.with_outputs(kept.outputs)
        }
    }

    /// Puts `lists` in place of the cue lists without recording a change,
    /// as undo and redo do.  The cues that have gone stop, and new cues get
    /// identities that none of the lists holds.
    pub(super) fn with_lists(self, lists: Vec<CueList>) -> Self {
        let lists = super::listed(lists);
        let workspace = Self {
            next_id: super::next_id(&lists).max(self.next_id),
            lists,
            ..self
        };
        let gone = workspace.active_where(|active| workspace.cue(active.id).is_none());
        gone.into_iter().fold(workspace, Self::stopped)
    }

    /// Removes cues from their lists and stops those that are running,
    /// without recording a change, as undo and redo remove them
    pub(super) fn removed(self, ids: impl IntoIterator<Item = CueId>) -> Self {
        ids.into_iter().fold(self, |workspace, id| {
            workspace
                .with_list_of(id, |cues| cues.without(id))
                .stopped(id)
        })
    }

    /// Edits a cue without recording the edit, as undo and redo edit it.
    /// A running cue stops when it gets another file, which has yet to
    /// load, while a file name that leads to the same file, such as the
    /// one chosen again, leaves it playing.  A running audio or video cue
    /// takes a new level at once, which ends a fade on it.  A cue that a
    /// panic or a fade is stopping keeps fading out, so that no edit brings
    /// it back.
    pub(super) fn changed(self, id: CueId, edit: Edit) -> Self {
        let Some(cue) = self.cue(id) else {
            return self;
        };
        let (path, plays) = (self.path(cue), cue.kind.plays_file());
        let workspace = match edit {
            Edit::Level(level) if plays => self.with_active(id, |active| match active.fade {
                Some(fade) if fade.then_stop => active,
                _ => Active {
                    level,
                    fade: None,
                    ..active
                },
            }),
            _ => self,
        }
        .with_list_of(id, |cues| cues.edited(id, edit));
        let refiled = workspace
            .cue(id)
            .is_some_and(|cue| workspace.path(cue) != path);
        if refiled {
            workspace
                .stopped(id)
                .with_list_of(id, |cues| cues.loaded(id, Media::Unloaded))
        } else {
            workspace
        }
    }

    /// Records what loading the file of a cue found, unless the cue has
    /// been given another file since loading began.  A run whose sound
    /// could not start then ends, and the rest of its chain plays on, as GO
    /// skips a cue that cannot play.
    pub(super) fn loaded(self, id: CueId, file: &str, media: Media) -> Self {
        if self.cue(id).is_none_or(|cue| cue.file != file) {
            return self;
        }
        let workspace = match media {
            Media::Failed(_) => self.skipped(id),
            _ => self,
        };
        workspace.with_list_of(id, |cues| cues.loaded(id, media))
    }

    fn place_of(&self, id: CueId) -> Option<Placed> {
        self.lists.iter().enumerate().find_map(|(list, cues)| {
            cues.position(id).map(|index| Placed {
                list,
                index,
                cue: cues.cues[index].clone(),
            })
        })
    }
}
