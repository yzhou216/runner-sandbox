// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints of the keys that act in most states

use super::{
    keyboard::{acts_here, renaming},
    keys::{Hint, Keys},
    menu_open, notices,
};
use crate::front::keymap::Action;

/// Hints two keys that a newcomer to the outputs needs, and none that
/// destroys
pub(super) fn teaching(keys: Keys) -> Vec<Hint> {
    let app = keys.0;
    // A keymap that binds no key to edit may bind one to change the name,
    // which enters the same field, and the hint gives way before add output
    // as that of renaming a cue does, so that the key to add still shows
    (app.outputs_shown && app.editable())
        .then(|| {
            [
                keys.here(Action::AddBelow, "add output"),
                keys.here(Action::Edit, "rename")
                    .or_else(|| keys.here(Action::ChangeName, "rename")),
            ]
        })
        .into_iter()
        .flatten()
        .flatten()
        .collect()
}

/// Hints the key of renaming a cue just added, which bears a default name that
/// the key selects whole, so that typing names it
pub(super) fn rename(keys: Keys) -> Option<Hint> {
    let app = keys.0;
    keys.here(Action::ChangeName, "rename")
        .filter(|_| renaming(app) && notices::asked(app).is_none())
}

pub(super) fn go(keys: Keys) -> Option<Hint> {
    // Space types a space while a text field has the keyboard, so that GO
    // is hinted by a key that is no character, or by none
    if keys.0.keyboard.held.text {
        keys.0
            .config
            .keymap
            .hint_over_text(Action::Go, "GO")
            .map(|hint| (Some(Action::Go), hint))
    } else {
        keys.key(Action::Go, "GO")
    }
}

pub(super) fn help(keys: Keys) -> Option<Hint> {
    let app = keys.0;
    // A field or a list that has the keyboard keeps the key of help from
    // acting, and `?` types there, while an open menu and the page itself take it
    keys.key(Action::Help, Action::Help.name()).filter(|_| {
        ((app.help || menu_open(app)) && !app.stage.has_keyboard()) || acts_here(app, Action::Help)
    })
}

/// Hints the key of fullscreen, taught where the operator needs it: to open
/// the window of the audience, closed while a cue shows on it, and to leave a
/// fullscreen stage window that covers the main window on a single display,
/// from which almost every other key is left to the main window
pub(super) fn stage(keys: Keys) -> Option<Hint> {
    let app = keys.0;
    keys.here(
        Action::Fullscreen,
        if app.stage.has_keyboard() {
            "fullscreen"
        } else {
            "open stage"
        },
    )
    .filter(|_| {
        (app.stage.has_keyboard() || app.stage.missing(&app.workspace))
            && !menu_open(app)
            && !app.help
    })
}

/// Hints the key of hint mode, taught where it acts
pub(super) fn hint_mode(keys: Keys) -> Option<Hint> {
    keys.here(Action::Hint, "hint")
        .filter(|_| !menu_open(keys.0))
}
