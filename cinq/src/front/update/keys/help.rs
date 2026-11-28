// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys typed while the help page shows, which scroll and page it

use iced::{
    Task,
    widget::operation::{self, AbsoluteOffset, RelativeOffset},
};

use crate::front::{App, Message, keymap::Action, view};

impl App {
    /// Applies the key of `action` while the help page shows.  The keys that
    /// scroll the page, and the key that opens the tutorial, act on it, and
    /// the others page it, unless they act over it, which leave it to
    /// [`Self::keyed`].
    pub(super) fn help_keyed(
        self,
        action: Action,
        count: Option<usize>,
        repeat: bool,
    ) -> (Self, Task<Message>) {
        match action {
            Action::Next | Action::Previous | Action::ScrollDown | Action::ScrollUp => {
                let down = if matches!(action, Action::Next | Action::ScrollDown) {
                    1.0
                } else {
                    -1.0
                };
                let lines = count.map_or(1.0, |count| count as f32);
                let offset = AbsoluteOffset {
                    x: 0.0,
                    y: down * lines * view::line_height(),
                };
                (self, operation::scroll_by(view::HELP, offset))
            }
            Action::First => (self, operation::snap_to(view::HELP, RelativeOffset::START)),
            Action::Last => (self, operation::snap_to(view::HELP, RelativeOffset::END)),
            // The page has no focus for links, so that its keys reach the
            // tutorial, which a held key would open again on every repeat
            Action::Standby if repeat => (self, Task::none()),
            Action::Standby => self.handled(Message::OpenTutorial),
            action if !action.acts_over_help() => match action.page() {
                Some(page) => self.help_paged(page, count),
                None => (self, Task::none()),
            },
            action => self.keyed(action, count),
        }
    }
}
