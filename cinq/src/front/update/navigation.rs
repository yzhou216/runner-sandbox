// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Standby as GO moves it, the selection of a cue, and switching the cue
//! list on screen

use crate::{
    back::{Command, CueId},
    front::{App, step},
};

impl App {
    /// A selection on the standby cue moves on with standby, so that the cue
    /// list keeps the next cue in view as the show runs, while a selection
    /// elsewhere stays where the operator put it.
    pub(super) fn gone(self) -> Self {
        let following = self.selected.is_some() && self.selected == self.current().standby;
        let staged = self
            .workspace
            .next_go(self.current())
            .any(|(cue, _)| cue.kind.on_stage());
        let go = Command::Go { list: self.list };
        let app = self.send(go);
        Self {
            help: false,
            selected: app.current().standby.filter(|_| following).or(app.selected),
            stage: if staged { app.stage.cued() } else { app.stage },
            ..app
        }
    }

    pub(super) fn with_selection(self, id: CueId) -> Self {
        Self {
            list: self.workspace.list_of(id).unwrap_or(self.list),
            selected: Some(id),
            ..self
        }
    }

    /// Shows a cue list with its standby cue selected, or its first cue
    /// when standby is past the end
    pub(super) fn switched_to(self, list: usize) -> Self {
        Self {
            selected: self
                .workspace
                .lists()
                .get(list)
                .and_then(|cues| cues.standby.or_else(|| cues.cues.first().map(|cue| cue.id))),
            list,
            ..self
        }
    }

    /// Shows a cue list in place of the outputs or another list.  The list
    /// that the outputs took the place of comes back with the cue that was
    /// selected, and any other list with its standby cue.
    pub(super) fn showing_list(self, list: usize) -> Self {
        match self.outputs_shown && list == self.list && self.selected_here().is_some() {
            true => self.showing_cues(),
            false => self.showing_cues().switched_to(list),
        }
    }

    /// Steps through the cue lists and, past the last of them, to the
    /// outputs, as the sidebar lists them
    pub(super) fn stepped_list(self, offset: isize) -> Self {
        let lists = self.workspace.lists().len();
        let from = if self.outputs_shown { lists } else { self.list };
        let to = step(Some(from), offset, lists + 1).unwrap_or(from);
        match to {
            _ if to == from => self,
            _ if to == lists => self.showing_outputs(),
            list => self.showing_list(list),
        }
    }
}
