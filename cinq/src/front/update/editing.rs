// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Adding, moving, and deleting cues around the selection, choosing the
//! files of audio, video, and picture cues, and undoing and redoing changes
//! to the show

use iced::Task;
use itertools::Itertools;

use crate::{
    back::{Change, Command, CueId, Kind, PICTURE_FILES, Place, SOUND_FILES, VIDEO_FILES},
    front::{App, Message},
};

impl App {
    pub(super) fn with_new_cue(self, place: fn(CueId) -> Place) -> Self {
        let id = self.workspace.next_cue_id();
        let add = Command::Add {
            list: self.list,
            kind: self.new_kind,
            at: self.selected_here().map_or(Place::End, place),
        };
        self.send(add).showing(id, None)
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
        Task::future(chooser.pick_file())
            .and_then(move |chosen| Task::done(Message::FileChosen(id, chosen.path().to_owned())))
    }

    pub(super) fn with_selected_moved(self, by: isize) -> Self {
        let moved = self.selected_here().map(|cue| Command::Move { cue, by });
        self.send_all(moved)
    }

    pub(super) fn without_selected(self, count: usize) -> Self {
        let Some(first) = self.selected_row() else {
            return self;
        };
        let ids = self.current().cues[first..]
            .iter()
            .take(count)
            .map(|cue| cue.id)
            .collect();
        self.send(Command::Delete(ids)).showing(None, Some(first))
    }

    pub(super) fn undone(self) -> Self {
        let cue = self.workspace.undoable().and_then(Change::cue);
        let row = self.selected_row();
        self.send(Command::Undo).showing(cue, row)
    }

    pub(super) fn redone(self) -> Self {
        let cue = self.workspace.redoable().and_then(Change::cue);
        let row = self.selected_row();
        self.send(Command::Redo).showing(cue, row)
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
