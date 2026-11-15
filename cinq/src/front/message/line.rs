// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keys of Vim's command line, which edit the search line

use iced::keyboard::{Key, Modifiers, key::Named};

use crate::front::keymap::Press;

/// What a key of the command line does to the text of the search line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKey {
    /// Deletes the word before the cursor
    DeleteWord,
    /// Deletes everything before the cursor
    DeleteAll,
    /// Recalls an older search that starts with the text typed
    Older,
    /// Recalls a newer search, or gives back the text typed
    Newer,
}

impl LineKey {
    /// Returns what the key pressed does to the search line, if it is one
    /// of Control+w, Control+u, ArrowUp, and ArrowDown, as Vim has them
    pub fn of(press: Press) -> Option<Self> {
        let is = |key: Key, modifiers| Press::from_key(&key, modifiers) == Some(press);
        let (control, none) = (Modifiers::CTRL, Modifiers::empty());
        let letter = |text: &str| Key::Character(text.into());
        [
            (Self::DeleteWord, letter("w"), control),
            (Self::DeleteAll, letter("u"), control),
            (Self::Older, Key::Named(Named::ArrowUp), none),
            (Self::Newer, Key::Named(Named::ArrowDown), none),
        ]
        .into_iter()
        .find_map(|(line, key, modifiers)| is(key, modifiers).then_some(line))
    }
}
