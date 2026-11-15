// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the back and forward buttons of the mouse

use iced::{mouse, window};

use super::{
    history::{after, cue, place},
    show,
};
use crate::front::{App, Message};

#[test]
fn the_side_buttons_of_the_mouse_go_back_and_forward_from_the_main_window_only() {
    let main = window::Id::unique();
    let jumped = App {
        main: Some(main),
        ..after(show(cue(5)), [Message::SwitchList(1)])
    };
    let side = |app: App, window, button| after(app, [Message::SidePressed(window, button)]);
    let elsewhere = side(jumped, window::Id::unique(), mouse::Button::Back);
    assert_eq!(place(&elsewhere), (1, cue(20)));
    let back = side(elsewhere, main, mouse::Button::Back);
    assert_eq!(place(&back), (0, cue(5)));
    let other = side(back, main, mouse::Button::Middle);
    assert_eq!(place(&other), (0, cue(5)));
    let forward = side(other, main, mouse::Button::Forward);
    assert_eq!(place(&forward), (1, cue(20)));
}

#[test]
fn the_side_buttons_do_nothing_over_the_help_page() {
    let main = window::Id::unique();
    let jumped = App {
        main: Some(main),
        help: true,
        ..after(show(cue(5)), [Message::SwitchList(1)])
    };
    let back = after(jumped, [Message::SidePressed(main, mouse::Button::Back)]);
    assert_eq!(place(&back), (1, cue(20)));
    let closed = App {
        help: false,
        ..back
    };
    let back = after(closed, [Message::SidePressed(main, mouse::Button::Back)]);
    assert_eq!(place(&back), (0, cue(5)));
}
