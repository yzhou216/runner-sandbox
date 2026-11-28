// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the highlight that the sidebar lights while a menu is open

use super::super::{after, show};
use crate::front::{App, Message, app::Keyboard};

/// Returns the interface with a menu open and its highlight on the default
fn menu_open() -> App {
    after(
        show(None),
        [Message::MenuOpen(true), Message::DefaultHighlighted(true)],
    )
}

#[test]
fn the_sidebar_lights_only_while_a_menu_is_open_with_its_highlight_on_the_default() {
    let lit = |app: &App| app.keyboard.menu && app.keyboard.default_highlighted;
    let open = menu_open();
    assert!(lit(&open));
    let moved = after(open, [Message::DefaultHighlighted(false)]);
    assert!(!lit(&moved));
    let closed = after(menu_open(), [Message::MenuOpen(false)]);
    assert!(!lit(&closed));
    assert!(
        !closed.keyboard.default_highlighted,
        "the app never thinks that the highlight rests there once the menu closes"
    );
}

#[test]
fn any_menu_opening_or_closing_clears_the_highlight_that_the_sidebar_lights_for() {
    assert!(
        !after(menu_open(), [Message::MenuOpen(true)])
            .keyboard
            .default_highlighted,
        "the menu of another list opens while the flag is still set"
    );
    let stale = App {
        keyboard: Keyboard {
            default_highlighted: true,
            ..Keyboard::default()
        },
        ..show(None)
    };
    assert!(
        !after(stale, [Message::MenuOpen(false)])
            .keyboard
            .default_highlighted
    );
}
