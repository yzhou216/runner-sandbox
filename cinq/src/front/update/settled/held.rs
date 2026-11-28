// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that report what holds the keyboard or the pointer

use super::super::{App, Message};
use crate::front::app::Keyboard;

impl App {
    /// Applies a message that reports a menu, a list, a field, or a window
    /// that holds the keyboard, or a click, and passes any other on to
    /// [`Self::sent_by`]
    pub(super) fn held_by(self, message: Message) -> Self {
        match message {
            // Menu reports where its highlight rests right after it opens, so
            // that a flag left by another menu, or by a list that left the
            // view while open, never lights the sidebar
            Message::MenuOpen(open) => self.with_keyboard(|keyboard| keyboard.with_menu(open)),
            Message::Held(held) => self.with_keyboard(|keyboard| keyboard.with_held(held)),
            Message::DefaultHighlighted(highlighted) => {
                self.with_keyboard(|keyboard| keyboard.with_default_highlighted(highlighted))
            }
            // Click on blank space would leave the search line open with no
            // way to close it but Panic, and any click leaves hint mode
            Message::Clicked => self.without_search().without_hints(),
            // Stage window holds no cue list to keep in view, nor places that
            // hint mode could label out of its sight
            Message::WindowFocused(window) => {
                let app = if self.stage.is_window(window) {
                    self.without_hints()
                } else {
                    self
                };
                Self {
                    stage: app.stage.with_keyboard_in(window),
                    ..app
                }
            }
            message => self.sent_by(message),
        }
    }

    /// Returns the state with the keyboard as `update` leaves it
    fn with_keyboard(self, update: impl FnOnce(Keyboard) -> Keyboard) -> Self {
        Self {
            keyboard: update(self.keyboard),
            ..self
        }
    }
}
