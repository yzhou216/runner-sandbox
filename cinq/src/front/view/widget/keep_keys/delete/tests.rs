// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the events that Control+w and Control+u are rewritten into

use super::*;

fn control(letter: &str, modifiers: Modifiers) -> Event {
    pressed(Key::Character(letter.into()), modifiers)
}

/// Lists what each event does as the field reads it: the modifiers that it
/// sets, or the named key that it presses with the modifiers held
fn read(events: &[Event]) -> Vec<(Option<Named>, Modifiers)> {
    events
        .iter()
        .map(|event| match event {
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => (None, *modifiers),
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key),
                modifiers,
                ..
            }) => (Some(*key), *modifiers),
            other => panic!("unexpected event {other:?}"),
        })
        .collect()
}

#[test]
fn control_w_deletes_a_word_before_the_caret_as_backspace_with_the_jump_modifier() {
    let events = rewrite(&control("w", Modifiers::CTRL)).expect("a key that deletes");
    assert_eq!(
        read(&events),
        [
            (None, jump()),
            (Some(Named::Backspace), jump()),
            (None, Modifiers::CTRL),
        ]
    );
}

#[test]
fn control_u_selects_to_the_start_with_shift_then_deletes_the_selection() {
    let events = rewrite(&control("u", Modifiers::CTRL)).expect("a key that deletes");
    assert_eq!(
        read(&events),
        [
            (None, Modifiers::SHIFT),
            (Some(Named::Home), Modifiers::SHIFT),
            (None, Modifiers::empty()),
            (Some(Named::Backspace), Modifiers::empty()),
            (None, Modifiers::CTRL),
        ]
    );
}

#[test]
fn no_other_key_is_rewritten() {
    assert_eq!(rewrite(&control("w", Modifiers::empty())), None);
    assert_eq!(rewrite(&control("d", Modifiers::CTRL)), None);
    assert_eq!(
        rewrite(&pressed(Key::Named(Named::ArrowUp), Modifiers::empty())),
        None
    );
}
