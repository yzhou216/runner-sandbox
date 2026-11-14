// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that move through an open menu, of how far they move
//! its highlight, and of what keys and clicks leave to the widgets under it

use iced::{
    Event, Point, Size,
    advanced::{
        Shell,
        layout::{Layout, Node},
    },
    keyboard::{Key, Modifiers, key::Named},
    mouse,
};

use super::{Choice, Reply, Shared, reply, rows::Rows};
use crate::front::{
    Message,
    keymap::{Keymap, Preset, Press},
    step,
    view::widget::{dropdown::dropdown, wrap::Intercept},
};

/// Options of the menus in the tests, each picked as the step it makes
const OPTIONS: [isize; 3] = [1, 2, 3];

/// What a menu does with the events of a test
#[derive(Debug, PartialEq)]
struct Outcome {
    open: bool,
    /// Whether the menu keeps the events from the widgets under it
    captured: bool,
    picked: Option<isize>,
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
            .for_each(|key| choice.press(key, Press::from_key(key, Modifiers::empty()), shell));
    })
}

#[test]
fn keys_move_pick_close_or_wait_as_the_keymap_binds_them() {
    let (none, ctrl) = (Modifiers::empty(), Modifiers::CTRL);
    let (letter, named) = (|text: &str| Key::Character(text.into()), Key::Named);
    let cases = [
        (Preset::Vi, letter("n"), ctrl, Reply::Step(1)),
        (Preset::Vi, letter("p"), ctrl, Reply::Step(-1)),
        (Preset::Vi, named(Named::ArrowDown), none, Reply::Step(1)),
        (Preset::Vi, letter("k"), none, Reply::Step(-1)),
        (Preset::Cua, named(Named::ArrowUp), none, Reply::Step(-1)),
        (Preset::Cua, letter("n"), ctrl, Reply::Keep),
        (Preset::Vi, named(Named::Enter), none, Reply::Pick),
        (Preset::Vi, named(Named::Escape), none, Reply::Close),
        (Preset::Vi, named(Named::Space), none, Reply::Close),
        (Preset::Vi, letter("?"), none, Reply::Pass),
        (Preset::Cua, named(Named::F1), none, Reply::Pass),
        (Preset::Vi, letter("q"), none, Reply::Keep),
        (Preset::Vi, letter("n"), none, Reply::Keep),
        (Preset::Vi, letter("d"), none, Reply::Keep),
    ];
    let wrong: Vec<_> = cases
        .into_iter()
        .map(|(preset, key, modifiers, expected)| {
            let press = Press::from_key(&key, modifiers);
            let got = reply(&Keymap::from(preset), &key, press);
            (preset, key, modifiers, expected, got)
        })
        .filter(|(.., expected, got)| expected != got)
        .collect();
    assert!(
        wrong.is_empty(),
        "expected one reply but got another: {wrong:#?}"
    );
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
    let (letter, named) = (|text: &str| Key::Character(text.into()), Key::Named);
    let closed = |captured, picked| Outcome {
        open: false,
        captured,
        picked,
    };
    assert_eq!(pressed(&[named(Named::Enter)]), closed(true, Some(1)));
    assert_eq!(
        pressed(&[letter("j"), named(Named::Enter)]),
        closed(true, Some(2))
    );
    assert_eq!(pressed(&[named(Named::Space)]), closed(false, None));
    assert_eq!(
        pressed(&[letter("d")]),
        Outcome {
            open: true,
            captured: true,
            picked: None,
        }
    );
}

#[test]
fn help_acts_through_the_menu_and_leaves_it_open_on_its_row() {
    let letter = |text: &str| Key::Character(text.into());
    assert_eq!(
        pressed(&[letter("?")]),
        Outcome {
            open: true,
            captured: false,
            picked: None,
        }
    );
    let highlight = outcome(|choice, shell| {
        [letter("j"), letter("?")]
            .iter()
            .for_each(|key| choice.press(key, Press::from_key(key, Modifiers::empty()), shell));
        assert_eq!(choice.shared.highlight.get(), Some(1));
    });
    assert!(highlight.open);
}

#[test]
fn click_beside_the_menu_closes_it_and_acts_unless_it_lands_on_the_list() {
    let closed = |captured| Outcome {
        open: false,
        captured,
        picked: None,
    };
    assert_eq!(
        outcome(|choice, shell| choice.click_beside(false, shell)),
        closed(false)
    );
    assert_eq!(
        outcome(|choice, shell| choice.click_beside(true, shell)),
        closed(true)
    );
}

#[test]
fn mouse_highlights_the_row_under_it_and_picks_the_row_it_presses() {
    let rows = Node::with_children(
        Size::new(100.0, 90.0),
        (0..3)
            .map(|row| {
                Node::new(Size::new(100.0, 30.0)).move_to(Point::new(0.0, 30.0 * row as f32))
            })
            .collect(),
    );
    let at = |y| mouse::Cursor::Available(Point::new(10.0, y));
    let moved = Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(10.0, 40.0),
    });
    let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let picked = outcome(|choice, shell| {
        let list = Rows(choice);
        list.before(&moved, Layout::new(&rows), at(40.0), shell);
        assert_eq!(choice.shared.highlight.get(), Some(1));
        assert!(!shell.is_event_captured());
        list.before(&press, Layout::new(&rows), at(100.0), shell);
        assert_eq!(choice.shared.highlight.get(), Some(1));
        list.before(&press, Layout::new(&rows), at(70.0), shell);
    });
    assert_eq!(
        picked,
        Outcome {
            open: false,
            captured: true,
            picked: Some(3),
        }
    );
}

#[test]
fn highlight_follows_the_selected_option_until_a_key_moves_it() {
    let keymap = Keymap::default();
    let shared = Shared::default();
    let mut messages = Vec::new();
    let list =
        dropdown(OPTIONS, Some(OPTIONS[0]), Message::Step, &keymap).on_open(Message::Clicked);
    Choice::new(&list, &shared).open(&mut Shell::new(&mut messages));
    assert!(matches!(messages[..], [Message::Clicked]));
    // Options listed afresh under the open menu put the selected one last
    let relisted = dropdown([3, 2, 1], Some(OPTIONS[0]), Message::Step, &keymap);
    let choice = Choice::new(&relisted, &shared);
    assert_eq!(choice.highlighted(), Some(2));
    let up = Key::Named(Named::ArrowUp);
    choice.press(
        &up,
        Press::from_key(&up, Modifiers::empty()),
        &mut Shell::new(&mut messages),
    );
    assert_eq!(choice.highlighted(), Some(1));
}
