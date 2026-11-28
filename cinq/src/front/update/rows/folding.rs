// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Collapsing the groups of the cue list and expanding them, as Vim closes
//! and opens folds
//!
//! A collapsed group hides the cues that it holds, but never the selection:
//! collapsing the group that holds the selected cue selects the group, as
//! closing a fold puts the cursor on its line.

use crate::{
    back::{Command, CueId, Kind},
    front::{App, message::Fold},
};

impl App {
    pub(in crate::front::update) fn folded(self, fold: Fold) -> Self {
        let list = self.current();
        let selected = self.selected_index().map(|index| &list.cues[index]);
        let closed = selected.filter(|cue| cue.kind == Kind::Group && cue.collapsed);
        let all = || {
            list.cues
                .iter()
                .filter(|cue| cue.kind == Kind::Group)
                .map(|cue| cue.id)
                .collect()
        };
        let (cues, collapsed): (Vec<CueId>, bool) = match fold {
            Fold::Open | Fold::Toggle if closed.is_some() => {
                (closed.map(|cue| cue.id).into_iter().collect(), false)
            }
            Fold::Open => (Vec::new(), false),
            Fold::Close | Fold::Toggle => {
                let group = selected.and_then(|cue| match cue.kind {
                    Kind::Group if !cue.collapsed => Some(cue.id),
                    _ => list.group_of(cue.id).map(|group| group.id),
                });
                (group.into_iter().collect(), true)
            }
            Fold::CloseAll => (all(), true),
            Fold::OpenAll => (all(), false),
        };
        self.send(Command::Collapse { cues, collapsed })
            .with_selection_shown()
    }

    /// Collapses a group, or expands it where it is collapsed
    pub(in crate::front::update) fn flipped_group(self, id: CueId) -> Self {
        let collapsed = self
            .workspace
            .cue(id)
            .is_some_and(|cue| cue.kind == Kind::Group && !cue.collapsed);
        self.send(Command::Collapse {
            cues: vec![id],
            collapsed,
        })
        .with_selection_shown()
    }

    /// Selects the row that stands for the selected cue in the cue list, which
    /// is the outermost collapsed group that holds it, if one does
    fn with_selection_shown(self) -> Self {
        let standing = self.selected_index().and_then(|index| {
            let list = self.current();
            let row = list.groups().shown_as()[index];
            list.cues.get(row).map(|cue| cue.id)
        });
        Self {
            selected: standing.or(self.selected),
            ..self
        }
    }

    /// Expands the collapsed groups that hold the selected cue, since the
    /// selection is never hidden, as Vim opens the folds that a search or a
    /// jump lands in.  Every message that brings the selection into view
    /// comes through here, whichever cue it selects.
    pub(in crate::front::update) fn with_selection_unhidden(self) -> Self {
        let hiding = self.hiding_selection();
        self.collapsing(hiding, false)
    }

    /// Returns the collapsed groups that hide the selected cue, which are
    /// none while the outputs show
    pub(in crate::front::update) fn hiding_selection(&self) -> Vec<CueId> {
        self.selected
            .filter(|_| !self.outputs_shown)
            .map(|id| self.current().groups().hiding(id))
            .unwrap_or_default()
    }

    /// Collapses `cues` when `collapsed` and expands them otherwise, unless
    /// there are none
    pub(in crate::front::update) fn collapsing(self, cues: Vec<CueId>, collapsed: bool) -> Self {
        if cues.is_empty() {
            self
        } else {
            self.send(Command::Collapse { cues, collapsed })
        }
    }
}
