// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages of the search line

use iced::Task;

use super::{App, Message};

impl App {
    pub(super) fn searched_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            Message::Search { backward } => self.searching(backward),
            Message::Searching(text) => self.search_typed(text),
            Message::SearchKey(key) => self.search_edited(key),
            Message::SearchAccepted => self.search_accepted(),
            Message::SearchStep(offset) => self.search_stepped(offset),
            // Keys typed so far went already, as for any message
            Message::Cancel => self.cancelled(),
            message => (self.settled_by(message), Task::none()),
        }
    }
}
