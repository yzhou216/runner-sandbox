// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that move the highlight a page or to either end of the
//! menu

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
        .chain(&[named(Named::Enter)])
        .for_each(|key| type_key(choice, key, Modifiers::empty(), &mut shell));
    messages.iter().find_map(|message| match message {
        Message::Step(option) => usize::try_from(*option).ok(),
        _ => None,
    })
}

#[test]
fn page_keys_move_the_highlight_by_the_options_in_view_or_by_ten() {
    let (down, up) = (named(Named::PageDown), named(Named::PageUp));
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
    let (home, end) = (named(Named::Home), named(Named::End));
    assert_eq!(picked_after(4, std::slice::from_ref(&end)), Some(29));
    assert_eq!(picked_after(4, &[end, home]), Some(0));
}

#[test]
fn g_g_moves_the_highlight_to_the_first_option_as_capital_g_does_to_the_last() {
    // Each character of `typed` is a key of its own
    let after = |typed: &str| {
        let keys: Vec<Key> = typed.chars().map(|c| letter(&c.to_string())).collect();
        picked_after(4, &keys)
    };
    assert_eq!(after("Ggg"), Some(0));
    assert_eq!(after("G"), Some(29));
    // g j and g k move as j and k do
    assert_eq!(after("gj"), Some(1));
    assert_eq!(after("Ggk"), Some(28));
    // Key that completes no motion drops the g, which then waits afresh
    assert_eq!(after("Ggxg"), Some(29));
    assert_eq!(after("Ggxgg"), Some(0));
}

#[test]
fn closing_the_menu_drops_a_g_that_waits() {
    let g = letter("g");
    let keymap = Keymap::default();
    let shared = Shared::default();
    let list = dropdown(OPTIONS, Some(OPTIONS[0]), Message::Step, &keymap);
    let choice = Choice::new(&list, &shared);
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    choice.open(&mut shell);
    type_key(choice, &g, Modifiers::empty(), &mut shell);
    choice.close(&mut shell);
    choice.open(&mut shell);
    type_key(choice, &g, Modifiers::empty(), &mut shell);
    assert!(shared.is_open());
    assert_eq!(shared.highlight.take(), None);
}
