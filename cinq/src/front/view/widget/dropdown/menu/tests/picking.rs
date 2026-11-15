// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that pick the highlighted option or close the menu
//! without picking one

use iced::keyboard::{Key, Modifiers, key::Named};

use super::*;

#[test]
fn control_y_picks_the_highlight_and_control_e_closes_the_menu_without_a_pick() {
    let (letter, named) = (|text: &str| Key::Character(text.into()), Key::Named);
    let press = |key: &Key, modifiers| Press::from_key(key, modifiers);
    let closed = |picked| Outcome {
        open: false,
        captured: true,
        picked,
    };
    let (y, e, down) = (letter("y"), letter("e"), named(Named::ArrowDown));
    assert_eq!(
        outcome(|choice, shell| choice.press(&y, press(&y, Modifiers::CTRL), shell)),
        closed(Some(1))
    );
    assert_eq!(
        outcome(|choice, shell| choice.press(&e, press(&e, Modifiers::CTRL), shell)),
        closed(None)
    );
    assert_eq!(
        outcome(|choice, shell| {
            choice.press(&down, press(&down, Modifiers::empty()), shell);
            choice.press(&y, press(&y, Modifiers::CTRL), shell);
        }),
        closed(Some(2))
    );
}

#[test]
fn escape_still_closes_the_menu_and_acts_as_panic() {
    let escape = Key::Named(Named::Escape);
    assert_eq!(
        pressed(&[escape]),
        Outcome {
            open: false,
            captured: false,
            picked: None,
        }
    );
}
