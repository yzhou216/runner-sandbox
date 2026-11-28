// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys that make an operator with a motion, as `d` and `j` make a deletion
//! of rows

use iced::Task;

use crate::front::{App, Message, keymap::Action};

impl App {
    /// Applies the `operator` that a key completed over `motion`
    pub(super) fn operated(
        self,
        operator: Action,
        motion: Action,
        count: Option<usize>,
    ) -> (Self, Task<Message>) {
        // Help page has no cues to operate on, and the stage window leaves
        // the cues to the main window
        let remark = self.refusal(operator);
        let withheld = !self.help
            && !self.stage.has_keyboard()
            && remark.is_none()
            && self.reached(operator, motion, count).is_none();
        let app = Self {
            notices: if withheld {
                self.notices
            } else {
                self.notices.after_key(Some(operator))
            },
            ..self
        };
        if app.help || app.stage.has_keyboard() {
            (
                Self {
                    notices: app.notices.with_remark(remark),
                    ..app
                },
                Task::none(),
            )
        } else {
            let (app, task) = app.handled(Message::Across(operator, motion, count));
            (
                Self {
                    notices: app.notices.preferring_remark(remark),
                    ..app
                },
                task,
            )
        }
    }
}
