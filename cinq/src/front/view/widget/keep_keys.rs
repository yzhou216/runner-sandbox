// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Region of fields that keeps the keys typed into them
//!
//! iced's text fields let some keys through, such as the arrows, and a
//! read-only field lets every key through, as does a drop-down list apart
//! from Enter, which opens its menu.  Those keys would reach the key
//! bindings and act on the cue list while the operator types.  While one of
//! the fields has the keyboard focus, this region keeps the keys that the
//! field let through, apart from the keys that act in fields, which are
//! those of GO, Panic, and cancel, those that move between fields, those that
//! go back and forward, and those that open and save shows.  Enter leaves a
//! text field.  A field that takes a key of GO, Panic, or cancel without
//! typing it, as iced's text fields take Escape to give up the keyboard and
//! Control+c to copy a selection, leaves the key to act all the same, but a
//! letter that a user binds to cancel still types in the field.  The region
//! decides as each key arrives, so keys act in the order the operator typed
//! them.

use iced::{
    Element, Event, Rectangle, Renderer,
    advanced::{
        Shell,
        layout::Layout,
        widget::{
            Id, Operation, Tree,
            operation::{Focusable, TextInput},
        },
    },
    keyboard::{Key, key::Named},
};

use super::{
    key_pressed,
    wrap::{Intercept, wrap},
};
use crate::front::{
    Focus, Message,
    keymap::{KeyPress, Keymap},
};

pub fn keep_keys<'a>(
    content: impl Into<Element<'a, Message>>,
    keymap: &'a Keymap,
) -> Element<'a, Message> {
    wrap(content, Keep(keymap))
}

struct Keep<'a>(&'a Keymap);

impl Intercept for Keep<'_> {
    fn after(
        &self,
        content: &mut Element<'_, Message>,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let Some(KeyPress { key, press, repeat }) = key_pressed(event) else {
            return;
        };
        if shell.is_event_captured() {
            let character = matches!(key, Key::Character(_) | Key::Named(Named::Space));
            let acts = |press| self.0.acts_when_captured(press, character);
            if let Some(press) = press.filter(|&press| acts(press)) {
                shell.publish(Message::Typed { press, repeat });
            }
            return;
        }
        let mut focus = HasFocus::default();
        content
            .as_widget_mut()
            .operate(tree, layout, renderer, &mut focus);
        if !(focus.text || focus.list) {
            return;
        }
        // A closed list has no text to undo and no key of its own but Enter,
        // so that the keys of undo and redo act there
        let acts = press.is_some_and(|press| {
            self.0.acts_in_fields(press) || (focus.list && self.0.acts_in_lists(press))
        });
        match key {
            _ if acts => {}
            Key::Named(Named::Enter) => {
                shell.publish(Message::Focus(Focus::Cues));
                shell.capture_event();
            }
            _ => shell.capture_event(),
        }
    }
}

/// Operation that finds out whether a text field or a drop-down list has
/// the keyboard focus, read-only fields included, and which of the two
#[derive(Default)]
struct HasFocus {
    text: bool,
    list: bool,
    /// Whether the widget being visited reported itself as a text input
    /// just before it reports as focusable, which only a text field does
    reading: bool,
}

impl Operation for HasFocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }

    fn text_input(&mut self, _id: Option<&Id>, _bounds: Rectangle, _state: &mut dyn TextInput) {
        self.reading = true;
    }

    fn focusable(&mut self, _id: Option<&Id>, _bounds: Rectangle, state: &mut dyn Focusable) {
        let focused = state.is_focused();
        self.text |= focused && self.reading;
        self.list |= focused && !self.reading;
        self.reading = false;
    }
}

#[cfg(test)]
mod tests;
