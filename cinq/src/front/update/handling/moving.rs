// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that move through the rows, the lists, and the history of places

use iced::{Task, mouse};

use super::{App, Message};

impl App {
    pub(super) fn moved_by(self, message: Message) -> (Self, Task<Message>) {
        match message {
            // Selection scrolls into view as it or the rows around it move,
            // and as the main window changes size
            Message::Select(id) => self.picked(id),
            Message::Step(offset) => self.stepped(offset).revealed(),
            Message::Row(row) => self.jumping(|app| app.at_row(row)).revealed(),
            Message::Page(page, count) => self.paged(page, count),
            Message::Drawn(window) if self.main == Some(window) && self.unmeasured() => {
                let measure = self.measure_rows();
                (self, measure)
            }
            Message::Remeasure => {
                let app = self.without_hints().remeasured();
                let measure = app.measure_rows();
                (app, measure)
            }
            Message::Scroll(rows) => self.scrolled(rows),
            Message::InView(height, count) => self.in_view(height, count),
            Message::Place(height, count) => self.placed(height, count),
            Message::Fold(fold) => self.folded(fold).revealed(),
            // A click on the mark of a group leaves the rows where they are,
            // unless it took the selection into the group
            Message::FlipGroup(id) => self.revealing_a_move(|app| app.flipped_group(id)),
            Message::SwitchList(list) => self
                .without_search()
                .jumping(|app| app.showing_list(list))
                .revealed(),
            Message::StepList(offset) => self.jumping(|app| app.stepped_list(offset)).revealed(),
            Message::Back(count) => self.walking(|app| app.went_back(count)),
            Message::Forward(count) => self.walking(|app| app.went_forward(count)),
            // Buttons of the mouse go back and forward from the main window, as
            // the keys do, apart from the help page, which the keys leave be
            Message::SidePressed(
                window,
                button @ (mouse::Button::Back | mouse::Button::Forward),
            ) if self.main == Some(window) && !self.help => self.walking(|app| match button {
                mouse::Button::Back => app.went_back(1),
                _ => app.went_forward(1),
            }),
            message => (self.settled_by(message), Task::none()),
        }
    }
}
