// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that start or stop the show, here or from the network

use iced::Task;

use super::{App, Message};
use crate::back::Command;

impl App {
    pub(super) fn playing_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            // GO and Panic close the help page, so that the show is in view
            // as it runs or stops, and a selection that GO moves scrolls
            // into view, while the list stays put otherwise
            Message::Go => self.revealing_a_move(Self::gone),
            // Escape leaves a text field, so it gives up a search as well,
            // and the selection goes back to where the search began
            Message::Panic => Self {
                help: false,
                ..self.send(Command::Panic)
            }
            .search_abandoned(),
            Message::Remote(from, remote) => self.remote(from, remote),
            message => (self.settled_by(message), Task::none()),
        }
    }
}
