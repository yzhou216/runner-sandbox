// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that move the highlight a page or to either end of the
//! menu

use iced::keyboard::{Key, Modifiers, key::Named};

use super::*;

/// Types `keys` into the menu of a list of thirty options, shown `fit` at a
/// time, with the highlight on the first, and returns the option that Enter
/// then picks
fn picked_after(fit: usize, keys: &[Key]) -> Option<usize> {
    let keymap = Keymap::default();
    let shared = Shared::default();
    shared.fit.set(fit);
    let options: Vec<isize> = (0..30).collect();
    let list = dropdown(options, Some(0), Message::Step, &keymap);
    let choice = Choice::new(&list, &shared);
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    choice.open(&mut shell);
    keys.iter()
        .chain(&[Key::Named(Named::Enter)])
        .for_each(|key| choice.press(key, Press::from_key(key, Modifiers::empty()), &mut shell));
    messages.iter().find_map(|message| match message {
        Message::Step(option) => usize::try_from(*option).ok(),
        _ => None,
    })
}

#[test]
fn page_keys_move_the_highlight_by_the_options_in_view_or_by_ten() {
    let (down, up) = (Key::Named(Named::PageDown), Key::Named(Named::PageUp));
    assert_eq!(picked_after(4, std::slice::from_ref(&down)), Some(4));
    assert_eq!(picked_after(4, &[down.clone(), down.clone()]), Some(8));
    assert_eq!(
        picked_after(4, &[down.clone(), down.clone(), up.clone()]),
        Some(4)
    );
    // Before the menu is laid out, and past either end
    assert_eq!(picked_after(0, std::slice::from_ref(&down)), Some(10));
    assert_eq!(picked_after(25, &[down.clone(), down]), Some(29));
    assert_eq!(picked_after(4, &[up]), Some(0));
}

#[test]
fn home_and_end_move_the_highlight_to_the_first_and_last_option() {
    let (home, end) = (Key::Named(Named::Home), Key::Named(Named::End));
    assert_eq!(picked_after(4, std::slice::from_ref(&end)), Some(29));
    assert_eq!(picked_after(4, &[end, home]), Some(0));
}
