// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that open hint mode or the outputs

use iced::Task;

use super::{App, Message};

impl App {
    pub(super) fn shown_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            Message::Hint => self.hinting(),
            Message::Hinted(places) => (self.hinted(places), Task::none()),
            Message::ShowOutputs => self.jumping(Self::showing_outputs).revealed(),
            Message::InspectOutput(output) => self
                .jumping(|app| {
                    Self {
                        output: Some(output),
                        ..app
                    }
                    .showing_outputs()
                })
                .revealed(),
            message => (self.settled_by(message), Task::none()),
        }
    }
}
