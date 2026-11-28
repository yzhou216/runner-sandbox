// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keys of the keymap as hints, shared by the states that the hints of
//! the status bar tell apart

use super::keyboard::acts_here;
use crate::front::{App, keymap::Action};

/// Hint with the action whose keys it names when it can give way
pub(super) type Hint = (Option<Action>, String);

#[derive(Clone, Copy)]
pub(super) struct Keys<'a>(pub(super) &'a App);

impl Keys<'_> {
    pub(super) fn key(self, action: Action, label: &str) -> Option<Hint> {
        self.0
            .config
            .keymap
            .hint(action, label)
            .map(|hint| (Some(action), hint))
    }

    /// Key that is hinted only where it acts, which a field or a list that
    /// has the keyboard may keep from it
    pub(super) fn here(self, action: Action, label: &str) -> Option<Hint> {
        self.key(action, label)
            .filter(|_| acts_here(self.0, action))
    }
}
