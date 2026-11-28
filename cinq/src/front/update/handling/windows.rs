// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages about the windows

use iced::Task;

use super::{App, Message};

impl App {
    pub(super) fn windowed_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            Message::Resized(window, size) => self.resized(window, size),
            Message::ToggleFullscreen => self.toggled_fullscreen(),
            Message::Closed(window) => self.closed(window),
            message => (self.settled_by(message), Task::none()),
        }
    }
}
