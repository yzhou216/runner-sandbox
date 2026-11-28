// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that act in a checkbox or a button that Tab reached

use iced::{
    keyboard::{self, Location, Modifiers, key::Physical},
    mouse,
};

use super::*;
use crate::{
    back::{CueId, Edit},
    front::keymap::{Action, Press, desk},
};

fn enter() -> Key {
    Key::Named(Named::Enter)
}

fn key_event(key: Key, repeat: bool) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Unidentified(keyboard::key::NativeCode::Unidentified),
        location: Location::Standard,
        modifiers: Modifiers::empty(),
        text: None,
        repeat,
    })
}

fn act() -> Message {
    Message::Edit(CueId(1), Edit::Loop(true))
}

/// What a widget does with an event: whether it took it, what it sent, and
/// whether it kept the event from the widgets after it
#[derive(Debug, PartialEq)]
struct Handled {
    taken: bool,
    sent: usize,
    captured: bool,
}

/// Returns the messages that a widget under `keymap` with `act` sends for
/// `event`, and what it does with it, where `focused` says whether it has the
/// keyboard
fn handling(
    keymap: &Keymap,
    focused: bool,
    event: &Event,
    act: Option<&Message>,
    leaves: bool,
) -> (Handled, Vec<Message>) {
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    let taken = keyed(
        &mut State { focused },
        event,
        keymap,
        act,
        leaves,
        &mut shell,
    );
    let captured = shell.is_event_captured();
    let handled = Handled {
        taken,
        sent: messages.len(),
        captured,
    };
    (handled, messages)
}

fn typing(focused: bool, key: Key, repeat: bool) -> Handled {
    handling(
        &Keymap::default(),
        focused,
        &key_event(key, repeat),
        Some(&act()),
        false,
    )
    .0
}

fn handled(taken: bool, sent: usize) -> Handled {
    Handled {
        taken,
        sent,
        captured: taken,
    }
}

#[test]
fn enter_acts_once_in_the_widget_that_has_the_keyboard_and_keeps_the_key_from_the_bindings() {
    assert_eq!(typing(true, enter(), false), handled(true, 1));
    // A held key repeats, but the widget acts once
    assert_eq!(typing(true, enter(), true), handled(true, 0));
    // A widget without the keyboard takes no key
    assert_eq!(typing(false, enter(), false), handled(false, 0));
}

#[test]
fn enter_is_the_key_of_standby_and_no_key_that_acts_at_once() {
    let keymap = Keymap::default();
    let press = Press::from_key(&enter(), Modifiers::empty()).expect("a key");
    assert_eq!(keymap.alone(press), Some(Action::Standby));
    assert!(!keymap.acts_at_once(press));
}

#[test]
fn space_and_escape_never_act_and_are_left_to_go_and_panic() {
    [Named::Space, Named::Escape, Named::Tab, Named::ArrowDown]
        .map(Key::Named)
        .into_iter()
        .chain([Key::Character("x".into())])
        .for_each(|key| assert_eq!(typing(true, key, false), handled(false, 0)));
}

#[test]
fn enter_sends_nothing_from_a_widget_that_has_nothing_to_send() {
    let event = key_event(enter(), false);
    let (outcome, _) = handling(&Keymap::default(), true, &event, None, true);
    assert_eq!(outcome, handled(true, 0));
}

#[test]
fn a_key_that_acts_at_once_is_not_taken_even_when_it_is_enter() {
    let keymap = desk("go = \"Enter\"\nstandby = []\nrow_to_top = []");
    let event = key_event(enter(), false);
    let (outcome, _) = handling(&keymap, true, &event, Some(&act()), false);
    assert_eq!(outcome, handled(false, 0));
}

#[test]
fn a_widget_that_leaves_gives_the_keyboard_to_the_list_once_after_it_acts() {
    let sent = |leaves, repeat| {
        let event = key_event(enter(), repeat);
        handling(&Keymap::default(), true, &event, Some(&act()), leaves).1
    };
    assert!(matches!(
        sent(true, false).as_slice(),
        [Message::Edit(..), Message::Focus(Focus::Cues)]
    ));
    assert!(matches!(sent(false, false).as_slice(), [Message::Edit(..)]));
    assert!(sent(true, true).is_empty());
}

#[test]
fn a_press_of_the_left_button_takes_the_keyboard_away() {
    let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let mut state = State { focused: true };
    keyed(
        &mut state,
        &press,
        &Keymap::default(),
        None,
        false,
        &mut Shell::new(&mut Vec::new()),
    );
    assert!(!state.is_focused());
}

#[test]
fn the_state_takes_and_gives_up_the_keyboard() {
    let mut state = State { focused: false };
    assert!(!state.is_focused());
    state.focus();
    assert!(state.is_focused());
    state.unfocus();
    assert!(!state.is_focused());
}
