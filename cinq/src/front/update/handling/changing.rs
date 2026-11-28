// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that change the cues, apart from typing into their fields

use iced::Task;

use super::{App, Message};
use crate::back::Command;

impl App {
    pub(super) fn changed_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            Message::Add(place, count) => self.with_new_cue(place, count).revealed(),
            Message::Move(by) => self.with_selected_moved(by).revealed(),
            Message::Indent(count) => self.regrouped(count, Command::Indent).revealed(),
            Message::Outdent(count) => self.regrouped(count, Command::Outdent).revealed(),
            Message::Cut(count) => self.cut(count).revealed(),
            Message::Paste(place, count) => self.with_pasted(place, count).revealed(),
            Message::Delete(count) => self.without_selected(count).revealed(),
            Message::Across(operator, motion, count) => {
                self.across(operator, motion, count).revealed()
            }
            Message::NewKind(new_kind) => self.with_new_kind(new_kind),
            Message::Undo(count) => self.counting(|app| app.undone(count)).revealed(),
            Message::Redo(count) => self.counting(|app| app.redone(count)).revealed(),
            message => (self.settled_by(message), Task::none()),
        }
    }
}
