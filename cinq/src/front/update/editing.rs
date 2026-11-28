// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Adding, moving, and deleting cues around the selection, choosing the
//! files of audio, video, and picture cues, and undoing and redoing changes
//! to the show

mod clipboard;
mod operators;
mod stopping;

use std::ops::ControlFlow;

use iced::Task;
use itertools::Itertools;

use clipboard::PASTE_LIMIT;
pub(in crate::front) use clipboard::{Clipboard, Clipped};
pub(super) use clipboard::{EMPTY, Register};
pub(super) use stopping::Stopping;

use crate::{
    back::{
        Change, Command, CueId, Kind, PICTURE_FILES, Place, SOUND_FILES, VIDEO_FILES, Workspace,
    },
    front::{App, Message, Tab},
};

impl App {
    /// Adds `count` cues in a row next to the selection, at most as many as
    /// one paste adds, and selects the last of them, as Vim leaves its
    /// cursor on the last line that `3o` inserts.  The inspector shows its
    /// Basics tab, which holds the File field that a new cue needs first.
    pub(super) fn with_new_cue(self, place: fn(CueId) -> Place, count: usize) -> Self {
        let first = self.workspace.next_cue_id();
        let add = Command::Add {
            list: self.list,
            kind: self.new_kind,
            at: self.selected_here().map_or(Place::End, place),
            count: count.min(PASTE_LIMIT),
        };
        let app = self.send(add);
        let last = app
            .workspace
            .next_cue_id()
            .and_then(|next| next.0.checked_sub(1))
            .map(CueId)
            .filter(|last| first.is_some_and(|first| last.0 >= first.0));
        let tab = last.map_or(app.tab, |_| Tab::Basics);
        Self { tab, ..app }.showing(last, None)
    }

    /// Makes cue `id` play or show the `file` that was chosen, which names
    /// the cue after the file as [`crate::back::Cue::playing`] says
    pub(super) fn with_file(self, id: CueId, file: String) -> Self {
        let playing = self.workspace.cue(id).map(|cue| cue.playing(file));
        self.send_all(playing.map(|edit| Command::Edit(id, edit)))
    }

    pub(super) fn choose_cue_file(&self, id: CueId) -> Task<Message> {
        let Some(cue) = self.workspace.cue(id) else {
            return Task::none();
        };
        let (title, filter, files) = match cue.kind {
            Kind::Video => ("Choose video file", "Video files", VIDEO_FILES.as_slice()),
            Kind::Picture => (
                "Choose picture file",
                "Picture files",
                PICTURE_FILES.as_slice(),
            ),
            _ => ("Choose sound file", "Sound files", SOUND_FILES.as_slice()),
        };
        // File choosers on GNU/Linux tell endings apart by case
        let endings = files
            .iter()
            .flat_map(|ending| [ending.to_string(), ending.to_uppercase()])
            .collect_vec();
        let chooser = rfd::AsyncFileDialog::new()
            .set_title(title)
            .add_filter(filter, &endings)
            .add_filter("All files", &["*"])
            .set_directory(self.workspace.file_directory(cue));
        let show = self.show.id();
        Task::future(chooser.pick_file()).and_then(move |chosen| {
            Task::done(Message::FileChosen(show, id, chosen.path().to_owned()))
        })
    }

    pub(super) fn with_selected_moved(self, by: isize) -> Self {
        let moved = self.selected_here().map(|cue| Command::Move { cue, by });
        self.send_all(moved)
    }

    pub(super) fn without_selected(self, count: usize) -> Self {
        self.taking_away(count, Command::Delete)
    }

    /// Removes the selected row and the rows below it, as many as `count`
    /// holds, with the command that `remove` makes of their identities
    fn taking_away(self, count: usize, remove: fn(Vec<CueId>) -> Command) -> Self {
        let Some(first) = self.selected_row() else {
            return self;
        };
        let ids = self.ids_from(first, count);
        self.send(remove(ids)).showing(None, Some(first))
    }

    /// Moves the selected row and the rows below it, as many as `count`
    /// holds, with the command that `regroup` makes of their identities,
    /// into the group above or out of their groups.  The selection stays on
    /// the first of them, as it does for a move.
    pub(super) fn regrouped(self, count: usize, regroup: fn(Vec<CueId>) -> Command) -> Self {
        let regrouping = self
            .selected_row()
            .map(|first| regroup(self.ids_from(first, count)));
        self.send_all(regrouping)
    }

    /// Returns the identities of the cues that `count` rows of the current
    /// list from the row `first` down show, which stand for the cues that a
    /// collapsed group holds, as an expanded group stands for its own
    fn ids_from(&self, first: usize, count: usize) -> Vec<CueId> {
        let cues = &self.current().cues;
        self.shown_cues_from(first, count)
            .into_iter()
            .map(|index| cues[index].id)
            .collect()
    }

    /// Applies `turn`, an undo or a redo, which sends one command for each
    /// change that it takes back or makes again, and counts those changes
    /// for the notice.  A turn that finds none leaves the count of the turn
    /// before, as it leaves that turn's notice.
    pub(super) fn counting(self, turn: impl FnOnce(Self) -> Self) -> Self {
        let earlier = self.notices.turns;
        let app = turn(Self {
            notices: self.notices.with_turns(0),
            ..self
        });
        let turns = if app.notices.turns == 0 {
            earlier
        } else {
            app.notices.turns
        };
        Self {
            notices: app.notices.with_turns(turns),
            ..app
        }
    }

    /// Undoes up to `count` changes, and stops quietly at the oldest.  An
    /// addition or a paste below a cue gives the selection back to that cue,
    /// as the cursor of Vim goes back to where the change was made.
    pub(super) fn undone(self, count: usize) -> Self {
        let lands = |change: &Change| {
            change
                .anchor()
                .or_else(|| change.cue())
                .or_else(|| change.standing(false))
        };
        self.turned(
            count,
            &Command::Undo,
            Workspace::undoable,
            lands,
            Self::undone_outputs,
        )
    }

    /// Redoes up to `count` changes, and stops quietly at the newest
    pub(super) fn redone(self, count: usize) -> Self {
        let lands = |change: &Change| change.cue().or_else(|| change.standing(true));
        self.turned(
            count,
            &Command::Redo,
            Workspace::redoable,
            lands,
            Self::redone_outputs,
        )
    }

    /// Sends `command` once for each of `count` changes that `next` finds,
    /// one at a time so that the selection follows the last of them, which
    /// `lands` tells the cue to select for, and the notice tells the last and
    /// how many there were.  A change to the outputs that the cue list shows
    /// no sign of is one that `in_outputs` takes over with the changes that
    /// remain, which shows the outputs, since the change would happen out of
    /// sight.
    fn turned(
        self,
        count: usize,
        command: &Command,
        next: fn(&Workspace) -> Option<&Change>,
        lands: fn(&Change) -> Option<CueId>,
        in_outputs: fn(Self, usize) -> Self,
    ) -> Self {
        let flow = (0..count).try_fold(self, |app, done| match next(&app.workspace) {
            Some(Change::Outputs { .. }) if app.editable() && !app.outputs_shown => {
                let app = app.jumping(Self::showing_outputs);
                ControlFlow::Break(in_outputs(app, count - done))
            }
            Some(change) => {
                let cue = lands(change);
                let row = app.selected_row();
                // Light patch has no row to select, so its tab shows
                let tab = if matches!(change, Change::Patched { .. }) {
                    Tab::Patch
                } else {
                    app.tab
                };
                let app = Self { tab, ..app };
                ControlFlow::Continue(app.send(command.clone()).showing(cue, row))
            }
            None => ControlFlow::Break(app),
        });
        match flow {
            ControlFlow::Continue(app) | ControlFlow::Break(app) => app,
        }
    }

    /// Selects a cue that a change touched, if it is still there.  A
    /// selected cue that the change took away, from the row `row`, leaves
    /// the selection to the cue that has taken its place, or to the last
    /// cue when none has.
    fn showing(self, cue: Option<CueId>, row: Option<usize>) -> Self {
        let gone = |id: &CueId| self.workspace.cue(*id).is_none();
        match cue.filter(|id| !gone(id)) {
            Some(id) => self.with_selection(id),
            None if self.selected.as_ref().is_some_and(gone) => match row {
                Some(row) => self.at_row(row + 1),
                None => Self {
                    selected: None,
                    ..self
                },
            },
            None => self,
        }
    }
}
