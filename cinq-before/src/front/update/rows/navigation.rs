// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Standby as GO moves it, the selection of a cue, and switching the cue
//! list on screen

use iced::Task;

use crate::{
    back::{Command, CueId},
    front::{App, Message, stepping::step},
};

impl App {
    /// A selection on the standby cue moves on with standby, so that the cue
    /// list keeps the next cue in view as the show runs, while a selection
    /// elsewhere stays where the operator put it.  The stage comes up for a
    /// GO that fires a picture or video cue, itself or through a start cue.
    pub(in crate::front::update) fn gone(self) -> Self {
        let following = self.selected.is_some() && self.selected == self.current().standby;
        let fired = self.workspace.next_go(self.current()).collect();
        let staged = self
            .workspace
            .reached(fired)
            .iter()
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

    /// Selects a cue as a click on its row does: the search line closes, the
    /// cue lists show in place of the outputs, a jump to another list or out
    /// of the outputs is remembered, and the row scrolls into view
    pub(in crate::front::update) fn picked(self, id: CueId) -> (Self, Task<Message>) {
        self.without_search()
            .crossing(|app| app.showing_cues().with_selection(id))
            .revealed()
    }

    pub(in crate::front::update) fn with_selection(self, id: CueId) -> Self {
        Self {
            list: self.workspace.list_of(id).unwrap_or(self.list),
            selected: Some(id),
            ..self
        }
    }

    /// Shows a cue list with the row of its standby cue selected, or its
    /// first cue when standby is past the end.  A collapsed group that hides
    /// the standby cue is selected in its place, which leaves the group
    /// collapsed, and its row tells where GO goes.
    pub(in crate::front::update) fn switched_to(self, list: usize) -> Self {
        Self {
            selected: self.workspace.lists().get(list).and_then(|cues| {
                let standing = cues
                    .standby
                    .and_then(|id| cues.position(id))
                    .map(|index| cues.groups().shown_as()[index])
                    .map(|row| cues.cues[row].id);
                standing
                    .or(cues.standby)
                    .or_else(|| cues.cues.first().map(|cue| cue.id))
            }),
            list,
            ..self
        }
    }

    /// Shows a cue list in place of the outputs or another list.  The list
    /// that the outputs took the place of comes back with the cue that was
    /// selected, and any other list with its standby cue.
    pub(in crate::front::update) fn showing_list(self, list: usize) -> Self {
        // A click from before an undo shrank the lists may name one past them
        if list >= self.workspace.lists().len() {
            return self;
        }
        let back = self.outputs_shown && list == self.list && self.selected_here().is_some();
        let app = self.showing_cues();
        if back { app } else { app.switched_to(list) }
    }

    /// Steps through the cue lists and, past the last of them, to the
    /// outputs, as the sidebar lists them
    pub(in crate::front::update) fn stepped_list(self, offset: isize) -> Self {
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
