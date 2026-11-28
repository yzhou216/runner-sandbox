// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints while hint mode has the keyboard

use super::keys::{Hint, Keys};
use crate::front::{App, keymap::Action, update::Hints};

/// Returns only the keys that act in hint mode, since the others type a letter
/// of a label or leave it
pub(super) fn hints(app: &App, keys: Keys) -> Vec<Hint> {
    let letters = app
        .hints
        .as_ref()
        .map(Hints::typed)
        .filter(|typed| !typed.is_empty());
    // Backspace takes a letter back while any is typed, and leaves
    // only where none is
    let backspace = if letters.is_some() { "back" } else { "leave" };
    let leave = keys
        .key(Action::Cancel, "cancel")
        .map_or_else(|| format!("Backspace: {backspace}"), |(_, hint)| hint);
    [
        Some((None, letters.unwrap_or("Type a label").to_owned())),
        Some((None, leave)),
        keys.key(Action::Go, "GO"),
        keys.key(Action::Panic, "Panic"),
    ]
    .into_iter()
    .flatten()
    .collect()
}
