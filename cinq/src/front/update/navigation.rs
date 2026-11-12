// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Moving the selection, along with standby as GO moves it, and switching
//! the cue list on screen

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

    pub(super) fn stepped(self, offset: isize) -> Self {
        let cues = &self.current().cues;
        let selected = step(self.selected_row(), offset, cues.len()).map(|row| cues[row].id);
        Self { selected, ..self }
    }

    /// Selects the cue on a row counted from one, or the last cue for a row
    /// past the end
    pub(super) fn at_row(self, row: usize) -> Self {
        let cues = &self.current().cues;
        let selected = cues
            .get(row.saturating_sub(1))
            .or(cues.last())
            .map(|cue| cue.id);
        Self { selected, ..self }
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

    /// Steps through the cue lists and, past the last of them, to the
    /// outputs, as the sidebar lists them
    pub(super) fn stepped_list(self, offset: isize) -> Self {
        let lists = self.workspace.lists().len();
        let from = if self.outputs_shown { lists } else { self.list };
        let to = step(Some(from), offset, lists + 1).unwrap_or(from);
        match to {
            _ if to == from => self,
            _ if to == lists => self.showing_outputs(),
            list => Self {
                outputs_shown: false,
                ..self
            }
            .switched_to(list),
        }
    }
}
