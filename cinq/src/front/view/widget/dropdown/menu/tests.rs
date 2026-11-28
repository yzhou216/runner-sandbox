// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that move through an open menu, of how far they move
//! its highlight, and of what keys and clicks leave to the widgets under it

mod announcing;
mod mouse;
mod narrow;
mod pages;
mod picking;

use iced::{
    advanced::Shell,
    keyboard::{Key, Modifiers, key::Named},
};

use super::{Choice, Reply, Shared, reply};
use crate::front::{
    Message,
    keymap::{Keymap, Press, desk},
    stepping::step,
    view::widget::{dropdown::dropdown, wrap::Intercept},
};

/// Options of the menus in the tests, each picked as the step it makes
const OPTIONS: [isize; 3] = [1, 2, 3];

fn letter(text: &str) -> Key {
    Key::Character(text.into())
}

fn named(key: Named) -> Key {
    Key::Named(key)
}

/// Presses `key` with `modifiers` in the menu of `choice`
fn type_key(choice: Choice<'_>, key: &Key, modifiers: Modifiers, shell: &mut Shell<'_, Message>) {
    choice.press(key, Press::from_key(key, modifiers), shell);
}

/// Returns an action that presses `key` with `modifiers`, for the helpers
/// that take one
fn typing_with(key: Key, modifiers: Modifiers) -> impl Fn(Choice<'_>, &mut Shell<'_, Message>) {
    move |choice, shell| type_key(choice, &key, modifiers, shell)
}

/// Returns an action that presses `key` alone
fn typing(key: Key) -> impl Fn(Choice<'_>, &mut Shell<'_, Message>) {
    typing_with(key, Modifiers::empty())
}

/// What a menu does with the events of a test
#[derive(Debug, PartialEq)]
struct Outcome {
    open: bool,
    /// Whether the menu keeps the events from the widgets under it
    captured: bool,
    picked: Option<isize>,
}

fn closed(captured: bool, picked: Option<isize>) -> Outcome {
    Outcome {
        open: false,
        captured,
        picked,
    }
}

fn kept_open(captured: bool) -> Outcome {
    Outcome {
        open: true,
        captured,
        picked: None,
    }
}

/// Opens a menu of [`OPTIONS`] with the highlight on the first, lets `act`
/// hand it events, and returns what they amount to
fn outcome(act: impl Fn(Choice<'_>, &mut Shell<'_, Message>)) -> Outcome {
    let keymap = Keymap::default();
    let shared = Shared::default();
    let list = dropdown(OPTIONS, Some(OPTIONS[0]), Message::Step, &keymap);
    let choice = Choice::new(&list, &shared);
    choice.open(&mut Shell::new(&mut Vec::new()));
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    act(choice, &mut shell);
    let captured = shell.is_event_captured();
    let picked = messages.iter().find_map(|message| match message {
        Message::Step(option) => Some(*option),
        _ => None,
    });
    Outcome {
        open: shared.is_open(),
        captured,
        picked,
    }
}

/// Presses each of `keys` alone in turn in a menu that [`outcome`] opens,
/// and returns what they amount to
fn pressed(keys: &[Key]) -> Outcome {
    outcome(|choice, shell| {
        keys.iter()
            .for_each(|key| type_key(choice, key, Modifiers::empty(), shell));
    })
}

#[test]
fn keys_move_pick_close_or_wait_as_the_keymap_binds_them() {
    let (none, ctrl) = (Modifiers::empty(), Modifiers::CTRL);
    let cases = [
        (Keymap::default(), letter("n"), ctrl, Reply::Step(1)),
        (Keymap::default(), letter("p"), ctrl, Reply::Step(-1)),
        (
            Keymap::default(),
            named(Named::ArrowDown),
            none,
            Reply::Step(1),
        ),
        (Keymap::default(), letter("k"), none, Reply::Step(-1)),
        (Keymap::default(), named(Named::Enter), none, Reply::Pick),
        (Keymap::default(), letter("y"), ctrl, Reply::Pick),
        (Keymap::default(), letter("e"), ctrl, Reply::Dismiss),
        (Keymap::default(), letter("c"), ctrl, Reply::Dismiss),
        (desk(""), letter("c"), ctrl, Reply::Keep),
        (
            desk("close_menu = \"F4\""),
            named(Named::F4),
            none,
            Reply::Dismiss,
        ),
        (Keymap::default(), letter("y"), none, Reply::Keep),
        (Keymap::default(), named(Named::Escape), none, Reply::Close),
        (Keymap::default(), named(Named::Space), none, Reply::Close),
        (Keymap::default(), named(Named::F1), none, Reply::Pass),
        (Keymap::default(), letter("?"), none, Reply::Pass),
        (Keymap::default(), letter("q"), none, Reply::Keep),
        (Keymap::default(), letter("n"), none, Reply::Keep),
        (Keymap::default(), letter("d"), none, Reply::Keep),
        (Keymap::default(), letter("o"), ctrl, Reply::Keep),
        (Keymap::default(), letter("i"), ctrl, Reply::Keep),
        (Keymap::default(), letter("d"), ctrl, Reply::Keep),
        (
            Keymap::default(),
            named(Named::PageDown),
            none,
            Reply::Page(1),
        ),
        (Keymap::default(), letter("f"), ctrl, Reply::Page(1)),
        (Keymap::default(), letter("b"), ctrl, Reply::Page(-1)),
        (Keymap::default(), named(Named::Home), none, Reply::First),
        (Keymap::default(), named(Named::End), none, Reply::Last),
        (Keymap::default(), letter("G"), none, Reply::Last),
        (Keymap::default(), named(Named::Home), ctrl, Reply::First),
        (Keymap::default(), named(Named::F5), none, Reply::Keep),
    ];
    cases
        .into_iter()
        .for_each(|(keymap, key, modifiers, expected)| {
            let got = reply(&keymap, &key, Press::from_key(&key, modifiers));
            assert_eq!(got, expected, "{} {modifiers:?} {key:?}", keymap.name());
        });
}

#[test]
fn highlight_stays_put_past_either_end() {
    assert_eq!(step(None, -1, 3), Some(0));
    assert_eq!(step(Some(1), 1, 3), Some(2));
    assert_eq!(step(Some(2), 1, 3), Some(2));
    assert_eq!(step(Some(0), -1, 3), Some(0));
    assert_eq!(step(Some(0), 1, 0), None);
}

#[test]
fn enter_picks_the_highlight_and_go_closes_the_menu_without_keeping_its_key() {
    assert_eq!(pressed(&[named(Named::Enter)]), closed(true, Some(1)));
    assert_eq!(
        pressed(&[letter("j"), named(Named::Enter)]),
        closed(true, Some(2))
    );
    assert_eq!(pressed(&[named(Named::Space)]), closed(false, None));
    // Escape closes the same way and acts as Panic
    assert_eq!(pressed(&[named(Named::Escape)]), closed(false, None));
    assert_eq!(pressed(&[letter("d")]), kept_open(true));
}

#[test]
fn help_acts_through_the_menu_and_leaves_it_open_on_its_row() {
    let help = named(Named::F1);
    assert_eq!(pressed(std::slice::from_ref(&help)), kept_open(false));
    let highlight = outcome(|choice, shell| {
        [letter("j"), help.clone()]
            .iter()
            .for_each(|key| type_key(choice, key, Modifiers::empty(), shell));
        assert_eq!(choice.shared.highlight.get(), Some(1));
    });
    assert!(highlight.open);
}
