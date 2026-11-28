// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints for leaving what has the keyboard: typed keys, a search, a field, or
//! a list

use super::{
    keyboard::{choosing, enter_leaves, in_field},
    keys::{Hint, Keys},
};
use crate::front::{App, keymap::Action};

pub(super) fn hints(keys: Keys) -> Vec<Hint> {
    let app = keys.0;
    // A pick or a close by key, and a Tab, leave the keyboard on a field or a
    // list, from which Escape would panic and no click is at hand
    let in_field = in_field(app);
    let next_field = keys
        .key(Action::NextField, "next field")
        .filter(|_| in_field);
    // Tab skips the Choose button, so that Enter says how to reach the chooser
    // while the empty File field has the keyboard
    let choose = choosing(app).then(|| (None, "Enter: choose a file".to_owned()));
    // A keymap of the file's own may bind no key to cancel, while Enter
    // leaves a text field, though it opens the menu of a drop-down list and
    // chooses a file in the empty File field
    let leave = keys
        .key(Action::Cancel, "leave")
        .or_else(|| {
            (app.keyboard.held.text && enter_leaves(app) && choose.is_none())
                .then(|| (Some(Action::Cancel), "Enter: leave".to_owned()))
        })
        .filter(|_| in_field && app.keys.is_empty());
    [
        cancel(keys),
        choose,
        enter(app, in_field),
        next_field,
        leave,
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn cancel(keys: Keys) -> Option<Hint> {
    let app = keys.0;
    // Escape would panic, so that the key that only cancels is told while it
    // can.  Cancelling keys typed or a search never gives way, while the
    // hint of leaving a field does, as it only teaches.  Where the keymap binds
    // no key to cancel, the search line is left by Backspace on an empty line
    // and by Enter once text is typed, which accepts it.
    keys.key(Action::Cancel, "cancel")
        .map(|(_, hint)| hint)
        .or_else(|| {
            app.search
                .as_ref()
                .map(|search| match search.text.is_empty() {
                    true => "Backspace: leave".to_owned(),
                    false => "Enter: find".to_owned(),
                })
        })
        .filter(|_| !app.keys.is_empty() || app.search.is_some())
        .map(|hint| (None, hint))
}

/// Returns the hint of what Enter does in the field or list that has the
/// keyboard, if `in_field`
fn enter(app: &App, in_field: bool) -> Option<Hint> {
    // Enter flips a checkbox, presses a button, and opens a closed list, where
    // a newcomer may take Space, which is GO, for the key that does.  A text
    // field keeps Enter for leaving, and a key bound to act at once keeps it
    // from any of them.
    (in_field && !app.keyboard.held.text && enter_leaves(app)).then(|| {
        let does = match (app.keyboard.held.check, app.keyboard.held.button) {
            (true, _) => "flip",
            (false, true) => "press",
            (false, false) => "open",
        };
        (Some(Action::Pick), format!("Enter: {does}"))
    })
}
