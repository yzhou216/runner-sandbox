// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that pick the highlighted option or close the menu
//! without picking one

use iced::keyboard::{Key, Modifiers, key::Named};

use super::*;
use crate::front::view::widget::dropdown::state::State;

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

/// Opens a menu, lets `act` hand it events, and returns whether it then gives
/// the keyboard back to the list, which it does once
fn returns_keyboard(act: impl Fn(Choice<'_>, &mut Shell<'_, Message>)) -> (bool, bool) {
    let keymap = Keymap::default();
    let shared = Shared::default();
    let list = dropdown(OPTIONS, Some(OPTIONS[0]), Message::Step, &keymap);
    let choice = Choice::new(&list, &shared);
    choice.open(&mut Shell::new(&mut Vec::new()));
    act(choice, &mut Shell::new(&mut Vec::new()));
    (shared.take_returning(), shared.take_returning())
}

#[test]
fn the_keyboard_stays_on_the_list_after_a_pick_or_a_close_by_key_and_only_then() {
    let press = |key: Key| {
        move |choice: Choice<'_>, shell: &mut Shell<'_, Message>| {
            choice.press(&key, Press::from_key(&key, Modifiers::empty()), shell);
        }
    };
    let enter = Key::Named(Named::Enter);
    assert_eq!(returns_keyboard(press(enter)), (true, false), "a pick");
    let close = |choice: Choice<'_>, shell: &mut Shell<'_, Message>| {
        let e = Key::Character("e".into());
        choice.press(&e, Press::from_key(&e, Modifiers::CTRL), shell);
    };
    assert_eq!(returns_keyboard(close), (true, false), "a close");
    let down = Key::Named(Named::ArrowDown);
    assert_eq!(returns_keyboard(press(down)), (false, false), "a move");
    let escape = Key::Named(Named::Escape);
    assert_eq!(
        returns_keyboard(press(escape)),
        (false, false),
        "Escape acts as Panic and leaves the list"
    );
    assert_eq!(
        returns_keyboard(|choice, shell| choice.click_beside(false, shell)),
        (false, false),
        "a click is no key"
    );
}

#[test]
fn a_list_that_a_key_gave_the_keyboard_back_to_reads_as_holding_it_at_once() {
    use iced::advanced::widget::operation::Focusable;

    let keymap = Keymap::default();
    let mut state = State::new();
    let list = dropdown(OPTIONS, Some(OPTIONS[0]), Message::Step, &keymap);
    let enter = Key::Named(Named::Enter);
    let choice = Choice::new(&list, &state.shared);
    choice.open(&mut Shell::new(&mut Vec::new()));
    assert!(!state.is_focused(), "an open menu holds it");
    let choice = Choice::new(&list, &state.shared);
    choice.press(
        &enter,
        Press::from_key(&enter, Modifiers::empty()),
        &mut Shell::new(&mut Vec::new()),
    );
    assert!(state.is_focused(), "before the list takes it on an event");
    state.unfocus();
    assert!(
        !state.is_focused(),
        "leaving the list takes it away for good"
    );
}
