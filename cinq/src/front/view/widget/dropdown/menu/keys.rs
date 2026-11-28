// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Replies of the open menu to the keys pressed

use iced::keyboard::{Key, key::Named};

use crate::front::keymap::{Action, Keymap, Press};

/// What a key does while the menu is open
#[derive(Debug, PartialEq)]
pub(super) enum Reply {
    /// Moves the highlight by a number of options, downwards when positive
    Step(isize),
    /// Moves the highlight by a page of options, downwards when positive
    Page(isize),
    /// Moves the highlight to the first option or to the last
    First,
    Last,
    Pick,
    /// Closes the menu without a pick, and keeps the key from acting on
    /// anything behind it, which is how it differs from [`Reply::Close`]
    Dismiss,
    /// Lets the key act and closes the menu, as the keys of GO and Panic do
    Close,
    /// Lets the key act and leaves the menu open, as the key of Help does:
    /// the page hides the menu, which is open still when the page closes
    Pass,
    /// Keeps the key from acting on anything behind the menu
    Keep,
}

impl Reply {
    pub(super) fn moving(action: Action) -> Option<Self> {
        match action {
            Action::Next => Some(Self::Step(1)),
            Action::Previous => Some(Self::Step(-1)),
            Action::First => Some(Self::First),
            Action::Last => Some(Self::Last),
            _ => None,
        }
    }
}

/// Decides what a key does, given the press it makes when the key bindings
/// can name it
pub(super) fn reply(keymap: &Keymap, key: &Key, press: Option<Press>) -> Reply {
    match (press.and_then(|press| keymap.in_menu(press)), key) {
        (Some(Action::Next), _) => Reply::Step(1),
        (Some(Action::Previous), _) => Reply::Step(-1),
        (Some(Action::PageDown), _) => Reply::Page(1),
        (Some(Action::PageUp), _) => Reply::Page(-1),
        (Some(Action::First), _) => Reply::First,
        (Some(Action::Last), _) => Reply::Last,
        (Some(action), _) if action.acts_at_once() => Reply::Close,
        (Some(Action::Help), _) => Reply::Pass,
        (Some(Action::CloseMenu | Action::Cancel), _) => Reply::Dismiss,
        (Some(Action::Pick), _) | (_, Key::Named(Named::Enter)) => Reply::Pick,
        // The keys of a page, as Vim's popup menu has them, whichever keys
        // the keymap binds, unless held with a modifier
        (_, Key::Named(named)) if press.is_some_and(Press::is_bare) => match named {
            Named::PageDown => Reply::Page(1),
            Named::PageUp => Reply::Page(-1),
            Named::Home => Reply::First,
            Named::End => Reply::Last,
            _ => Reply::Keep,
        },
        _ => Reply::Keep,
    }
}
