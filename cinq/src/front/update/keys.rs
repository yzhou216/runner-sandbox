// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys and the keyboard focus
//!
//! Keys typed outside any text field go to the key bindings in
//! [`crate::front::keymap`], and the keys of a command typed halfway wait
//! in the interface state until the command is complete.  The inspector and
//! the toolbar keep the keys typed into their fields, apart from the keys
//! that act in fields.  iced moves the keyboard focus only on clicks, so the
//! keys that edit fields move it themselves.

use iced::{
    Task,
    advanced::widget::{operate, operation::focusable},
    keyboard,
    widget::operation::{self, AbsoluteOffset, RelativeOffset},
};

use crate::front::{
    App, Focus, Message, Tab,
    keymap::{Action, KeyPress, Press},
    view,
};

impl App {
    pub(super) fn typed(self, press: Press, repeat: bool) -> (Self, Task<Message>) {
        let outcome = self.config.keymap.parse(&self.keys, press);
        let app = Self {
            keys: outcome.pending,
            ..self
        };
        match outcome.action {
            Some((action, _)) if repeat && action.acts_once_a_press() => (app, Task::none()),
            // The page opens in the main window, which the fullscreen stage
            // window may cover, so that the page would open out of sight
            Some((Action::Help, _)) if app.stage.has_keyboard() => (app, Task::none()),
            Some((action @ (Action::Next | Action::Previous), count)) if app.help => {
                let down = if action == Action::Next { 1.0 } else { -1.0 };
                let lines = count.map_or(1.0, |count| count as f32);
                let offset = AbsoluteOffset {
                    x: 0.0,
                    y: down * lines * view::line_height(),
                };
                (app, operation::scroll_by(view::HELP, offset))
            }
            Some((Action::First, _)) if app.help => {
                (app, operation::snap_to(view::HELP, RelativeOffset::START))
            }
            Some((Action::Last, _)) if app.help => {
                (app, operation::snap_to(view::HELP, RelativeOffset::END))
            }
            Some((action, count)) if !app.help || action.acts_over_help() => {
                app.handled(action.message(count))
            }
            _ => (app, Task::none()),
        }
    }

    /// Editing the name shows the Basics tab, which holds the Name field.  No
    /// field takes the keyboard while the stage window has it, since the
    /// keys typed there would never reach the field.
    pub(super) fn focused(self, focus: Focus) -> (Self, Task<Message>) {
        match focus {
            Focus::Name | Focus::Next | Focus::Previous if self.stage.has_keyboard() => {
                (self, Task::none())
            }
            Focus::Name if self.outputs_shown && self.editable() && self.output.is_some() => {
                (self, operation::focus(view::OUTPUT_NAME))
            }
            Focus::Name if self.outputs_shown => (self, Task::none()),
            Focus::Name if self.editable() && self.selected.is_some() => (
                Self {
                    tab: Tab::Basics,
                    ..self
                },
                operation::focus(view::NAME),
            ),
            Focus::Name => (self, Task::none()),
            Focus::Next => (self, operation::focus_next()),
            Focus::Previous => (self, operation::focus_previous()),
            Focus::Cues => (self, operate(focusable::unfocus())),
        }
    }
}

/// Reads a key that no widget captured, with Shift applied, so that Shift
/// and `g` make `G`
pub(super) fn key_press(event: keyboard::Event) -> Option<Message> {
    let KeyPress { press, repeat, .. } = KeyPress::of(&event)?;
    press.map(|press| Message::Typed { press, repeat })
}
