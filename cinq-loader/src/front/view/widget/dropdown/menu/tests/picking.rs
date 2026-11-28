// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that pick the highlighted option or close the menu
//! without picking one

use super::*;
use crate::front::view::widget::dropdown::state::State;

#[test]
fn control_e_closes_the_menu_without_a_pick() {
    assert_eq!(
        outcome(typing_with(letter("e"), Modifiers::CTRL)),
        closed(true, None)
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
    assert_eq!(
        returns_keyboard(typing(named(Named::Enter))),
        (true, false),
        "a pick"
    );
    assert_eq!(
        returns_keyboard(typing_with(letter("e"), Modifiers::CTRL)),
        (true, false),
        "a close"
    );
    assert_eq!(
        returns_keyboard(typing(named(Named::ArrowDown))),
        (false, false),
        "a move"
    );
    assert_eq!(
        returns_keyboard(typing(named(Named::Escape))),
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
    let choice = Choice::new(&list, &state.shared);
    choice.open(&mut Shell::new(&mut Vec::new()));
    assert!(!state.is_focused(), "an open menu holds it");
    typing(named(Named::Enter))(
        Choice::new(&list, &state.shared),
        &mut Shell::new(&mut Vec::new()),
    );
    assert!(state.is_focused(), "before the list takes it on an event");
    state.unfocus();
    assert!(
        !state.is_focused(),
        "leaving the list takes it away for good"
    );
}
