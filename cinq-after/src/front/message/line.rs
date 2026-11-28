// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The keys of Vim's command line that recall a search, which a text field
//! does not take

use iced::keyboard::{Key, Modifiers, key::Named};

use crate::front::keymap::Press;

/// What a key of the command line does to the text of the search line
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKey {
    /// Recalls an older search that starts with the text typed
    Older,
    /// Recalls a newer search, or gives back the text typed
    Newer,
}

impl LineKey {
    /// Returns what the key pressed does to the search line, if it is one
    /// of `ArrowUp` and `ArrowDown`, as Vim has them
    pub fn of(press: Press) -> Option<Self> {
        let is = |key: Key| Press::from_key(&key, Modifiers::empty()) == Some(press);
        [
            (Self::Older, Key::Named(Named::ArrowUp)),
            (Self::Newer, Key::Named(Named::ArrowDown)),
        ]
        .into_iter()
        .find_map(|(line, key)| is(key).then_some(line))
    }
}
