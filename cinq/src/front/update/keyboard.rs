// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding out whether a text field or a drop-down list has the keyboard
//!
//! The widgets keep that to themselves, and the status bar hints how to leave
//! a field, so that one operation asks them after anything the operator does.
//! A second asks whether the File field of a cue is the one, since its hint
//! differs.

use iced::{
    Rectangle, Task,
    advanced::widget::{
        Id, Operation, operate,
        operation::{Focusable, Outcome},
    },
};

use crate::front::{Message, view};

/// Asks the widgets whether one of them has the keyboard, and whether it is
/// the File field, and tells [`Message::Held`] and [`Message::FileHeld`] the
/// answers
pub(super) fn held() -> Task<Message> {
    Task::batch([
        operate(Holding::anywhere()).map(Message::Held),
        operate(Holding::in_field(view::FILE)).map(Message::FileHeld),
    ])
}

/// Finds out whether any widget in any window has the keyboard focus, or
/// the one that goes by an identity
struct Holding {
    field: Option<Id>,
    found: bool,
}

impl Holding {
    fn anywhere() -> Self {
        Self {
            field: None,
            found: false,
        }
    }

    fn in_field(field: &'static str) -> Self {
        Self {
            field: Some(Id::new(field)),
            found: false,
        }
    }
}

impl Operation<bool> for Holding {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<bool>)) {
        operate(self);
    }

    fn focusable(&mut self, id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        self.found |=
            state.is_focused() && self.field.as_ref().is_none_or(|field| id == Some(field));
    }

    fn finish(&self) -> Outcome<bool> {
        Outcome::Some(self.found)
    }
}

#[cfg(test)]
mod tests;
