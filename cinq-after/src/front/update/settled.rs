// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that change the state and start no task, which [`App::handled`]
//! passes on once its families have declined them

mod held;
mod paneled;
mod reported;
mod sent;

use super::{App, Measured, Message};

impl App {
    /// Applies a message that only changes state
    ///
    /// The families of such messages try it in turn, this one first, which
    /// takes the reports of a measure, a landing, a tick, or a read, and the
    /// last leaves the state as it is for any other message.  A message that
    /// [`Self::handled`] takes up before it reaches here, such as a press of a
    /// mouse button that it moves by, falls through to that last family.
    pub(super) fn settled_by(self, message: Message) -> Self {
        match message {
            Message::Landed { from, row } if self.spot() == from => {
                self.at_row(row.saturating_add(1))
            }
            Message::Measured(Measured::Rows {
                serial,
                rows,
                count,
                shape,
            }) => self.with_measure(serial, rows, count, shape),
            Message::Measured(Measured::Help(view)) => self.with_help_measure(view),
            Message::Reached { from, row } if self.spot() == from => {
                self.jumping(|app| app.at_row(row.saturating_add(1)))
            }
            Message::Tick(now) => self.ticked(now),
            Message::SearchSettled => self.search_settled(),
            Message::Loaded(show, found) => self.loaded(show, found),
            Message::Rechecked(show, found) => self.rechecked(show, found),
            Message::Saved(saved) => self.saved(saved),
            Message::FileChosen(show, ..) if show != self.show.id() => self,
            Message::FileChosen(_, id, path) => {
                let file = self.workspace.file_name_of(&path);
                self.with_file(id, file)
            }
            message => self.reported_by(message),
        }
    }
}
