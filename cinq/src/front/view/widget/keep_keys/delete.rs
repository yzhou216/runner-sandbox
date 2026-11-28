// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys of Vim's Insert mode that delete before the caret of a text field
//!
//! iced's text field lets Control+w and Control+u pass, and deletes at the
//! caret only through the events that its own keys make: a word with the
//! jump modifier and Backspace, and everything before the caret by selecting
//! to the start with Shift and Home, then Backspace.  The field takes its
//! modifiers from the events that change them, not from a key press, so that
//! each key is rewritten into those events, ending with the modifiers that
//! are really held.  The field then edits at its caret and tells the app
//! once for each edit, as it does for typing.

use iced::{
    Event,
    keyboard::{
        self, Key, Location, Modifiers,
        key::{Named, Physical},
    },
};

use super::super::key_pressed;
use crate::front::keymap::{KeyPress, Press};

/// What a key that deletes before the caret removes
#[derive(Clone, Copy)]
enum Delete {
    /// Word before the caret
    Word,
    /// Everything before the caret
    All,
}

impl Delete {
    /// Returns what the key pressed deletes, if it is Control+w or Control+u
    fn of(press: Press) -> Option<Self> {
        let is = |text: &str| {
            Press::from_key(&Key::Character(text.into()), Modifiers::CTRL) == Some(press)
        };
        [(Self::Word, "w"), (Self::All, "u")]
            .into_iter()
            .find_map(|(delete, text)| is(text).then_some(delete))
    }
}

/// Returns the events that make a text field delete what Control+w or
/// Control+u does before its caret, or nothing for any other event
pub(super) fn rewrite(event: &Event) -> Option<Vec<Event>> {
    let Event::Keyboard(keyboard::Event::KeyPressed { modifiers, .. }) = event else {
        return None;
    };
    let delete = key_pressed(event)
        .and_then(|KeyPress { press, .. }| press)
        .and_then(Delete::of)?;
    let steps: &[(Modifiers, Named)] = match delete {
        Delete::Word => &[(jump(), Named::Backspace)],
        Delete::All => &[
            (Modifiers::SHIFT, Named::Home),
            (Modifiers::empty(), Named::Backspace),
        ],
    };
    Some(
        steps
            .iter()
            .flat_map(|&(modifiers, key)| {
                [
                    Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)),
                    pressed(Key::Named(key), modifiers),
                ]
            })
            .chain([Event::Keyboard(keyboard::Event::ModifiersChanged(
                *modifiers,
            ))])
            .collect(),
    )
}

/// Returns the modifier that widens a motion to a word, as iced reads it
fn jump() -> Modifiers {
    if cfg!(target_os = "macos") {
        Modifiers::ALT
    } else {
        Modifiers::CTRL
    }
}

fn pressed(key: Key, modifiers: Modifiers) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Unidentified(keyboard::key::NativeCode::Unidentified),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    })
}

#[cfg(test)]
mod tests;
