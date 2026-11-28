// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what a menu tells the interface when it opens and closes

use super::*;

/// Opens a menu of [`OPTIONS`] that tells where its highlight rests, lets
/// `act` hand it events, and returns what `heard` takes of the messages that
/// the opening and the events send
fn listened<T>(
    act: impl Fn(Choice<'_>, &mut Shell<'_, Message>),
    heard: fn(&Message) -> Option<T>,
) -> Vec<T> {
    let keymap = Keymap::default();
    let shared = Shared::default();
    let list = dropdown(OPTIONS, Some(OPTIONS[0]), Message::Step, &keymap)
        .on_highlight(|row| Message::DefaultHighlighted(row == Some(0)));
    let choice = Choice::new(&list, &shared);
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    choice.open(&mut shell);
    act(choice, &mut shell);
    messages.iter().filter_map(heard).collect()
}

/// Returns the `MenuOpen` messages that opening a menu and then `act` send
fn announced(act: impl Fn(Choice<'_>, &mut Shell<'_, Message>)) -> Vec<bool> {
    listened(act, |message| match message {
        Message::MenuOpen(open) => Some(*open),
        _ => None,
    })
}

/// Returns the `DefaultHighlighted` messages that opening a menu of a list
/// whose first option is its default, with the default chosen, and then
/// `act` send
fn highlighted(act: impl Fn(Choice<'_>, &mut Shell<'_, Message>)) -> Vec<bool> {
    listened(act, |message| match message {
        Message::DefaultHighlighted(on) => Some(*on),
        _ => None,
    })
}

#[test]
fn the_interface_hears_when_a_menu_opens_and_each_way_it_closes() {
    assert_eq!(announced(|_, _| {}), [true]);
    let closes = [
        announced(typing(named(Named::Enter))),
        announced(typing_with(letter("y"), Modifiers::CTRL)),
        announced(typing_with(letter("e"), Modifiers::CTRL)),
        announced(typing(named(Named::Escape))),
        announced(typing(named(Named::Space))),
        announced(|choice, shell| choice.click_beside(false, shell)),
        announced(|choice, shell| choice.close(shell)),
    ];
    closes
        .iter()
        .for_each(|heard| assert_eq!(heard, &[true, false]));
    // Keys that keep it open say nothing
    assert_eq!(announced(typing(letter("j"))), [true]);
}

#[test]
fn a_list_without_options_never_opens_and_says_nothing() {
    let keymap = Keymap::default();
    let list = dropdown(Vec::<isize>::new(), None, Message::Step, &keymap);
    let shared = Shared::default();
    let mut messages = Vec::new();
    Choice::new(&list, &shared).open(&mut Shell::new(&mut messages));
    assert!(!shared.is_open());
    assert!(messages.is_empty());
}

#[test]
fn the_interface_hears_each_option_that_the_highlight_moves_onto_and_when_the_menu_closes() {
    let (down, up, enter) = (
        named(Named::ArrowDown),
        named(Named::ArrowUp),
        named(Named::Enter),
    );
    assert_eq!(
        highlighted(|_, _| {}),
        [true],
        "it opens on the chosen option"
    );
    assert_eq!(
        highlighted(|choice, shell| {
            [&down, &down, &up, &up]
                .into_iter()
                .for_each(|key| type_key(choice, key, Modifiers::empty(), shell));
        }),
        [true, false, false, false, true],
        "each option that it moves onto is told, and none twice running"
    );
    assert_eq!(
        highlighted(|choice, shell| {
            type_key(choice, &down, Modifiers::empty(), shell);
            choice.close(shell);
        }),
        [true, false, false],
        "closing tells that none has it, so the interface is never left with a stale one"
    );
    assert_eq!(highlighted(typing(enter)), [true, false], "so does a pick");
    assert_eq!(
        highlighted(|choice, shell| {
            choice.close(shell);
            choice.open(shell);
        }),
        [true, false, true]
    );
}
