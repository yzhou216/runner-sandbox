// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding out which widgets have the keyboard
//!
//! The widgets keep that to themselves, and the status bar hints how to leave
//! a field, so that one operation asks them after anything the operator does.
//! Besides whether any widget has it, the answer tells whether it is the File
//! field of a cue, a checkbox, a button, or an address of the light patch,
//! since each changes what the status bar or the patch shows, and whether it
//! is a text field, which keeps every key but those that act in fields, while
//! a closed drop-down list lets the keys of undo and redo act as well.

use iced::{
    Rectangle, Task,
    advanced::widget::{
        Id, Operation, operate,
        operation::{Focusable, Outcome, TextInput},
    },
};

use crate::front::{Message, app::Held, view};

/// Asks the widgets which of them has the keyboard, and tells
/// [`Message::Held`] the answer.  Chain it after a task that may move the
/// keyboard: a move by Tab takes two operations, so that a question asked
/// beside it would be answered between them, for the field that had the
/// keyboard before.
pub(in crate::front::update) fn held() -> Task<Message> {
    operate(Holding::default()).map(Message::Held)
}

#[derive(Default)]
struct Holding {
    held: Held,
    /// Whether the widget visited last is a text field, which tells itself
    /// just before it offers its focus
    text: bool,
}

impl Operation<Held> for Holding {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Held>)) {
        operate(self);
    }

    fn text_input(&mut self, _id: Option<&Id>, _bounds: Rectangle, _state: &mut dyn TextInput) {
        self.text = true;
    }

    fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        let focused = state.is_focused();
        let named = |field: &'static str| focused && id == Some(&Id::new(field));
        self.held = Held {
            any: self.held.any || focused,
            file: self.held.file || named(view::FILE),
            check: self.held.check || named(view::CHECK),
            button: self.held.button || named(view::BUTTON),
            address: self.held.address || named(view::ADDRESS),
            text: self.held.text || (focused && self.text),
        };
        self.text = false;
    }

    fn finish(&self) -> Outcome<Held> {
        Outcome::Some(self.held)
    }
}

#[cfg(test)]
mod tests;
