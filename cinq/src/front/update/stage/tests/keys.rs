// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys while the stage window has the keyboard: the fields that
//! take no keys, the key of fullscreen, and the keys of the help page

use iced::keyboard::{Key, key::Named};

use super::show;
use crate::{
    back::{CueId, Kind},
    front::{
        App, Caret, Focus, Message,
        update::tests::{field_keys, press},
    },
};

#[test]
fn no_field_takes_the_keyboard_while_the_stage_window_has_it() {
    let (open, _) = show(Kind::Video).staged();
    let stage = open.stage.window().expect("the stage window opens");
    let main = open.main.expect("the main window opens");
    let selected = App {
        selected: Some(CueId(1)),
        ..open
    };
    let (staged, _) = selected.handled(Message::WindowFocused(stage));
    let (staged, naming) = staged.focused(Focus::Name(Caret::End));
    let (staged, next) = staged.focused(Focus::Next);
    let (staged, previous) = staged.focused(Focus::Previous);
    assert_eq!((naming.units(), next.units(), previous.units()), (0, 0, 0));
    let (back, _) = staged.handled(Message::WindowFocused(main));
    let (_, naming) = back.focused(Focus::Name(Caret::End));
    assert_eq!(naming.units(), 1);
}

#[test]
fn key_of_fullscreen_held_down_toggles_the_stage_window_once() {
    let (open, _) = show(Kind::Video).staged();
    let f11 = press(Key::Named(Named::F11));
    let (open, toggling) = open.typed(f11, false);
    assert_eq!(toggling.units(), 1);
    let (_, repeating) = open.typed(f11, true);
    assert_eq!(repeating.units(), 0);
}

#[test]
fn help_key_opens_the_page_from_the_main_window_only() {
    // The page opens in the main window, which the fullscreen stage window
    // may cover
    let typed = |in_stage: bool| {
        let (open, _) = show(Kind::Video).staged();
        let stage = open.stage.window().expect("the stage window opens");
        let window = if in_stage {
            stage
        } else {
            open.main.expect("the main window opens")
        };
        let (keyboard, _) = open.handled(Message::WindowFocused(window));
        let (next, task) = keyboard.typed(press(Key::Named(Named::F1)), false);
        (next.help, task.units())
    };
    assert_eq!(typed(false), (true, 0));
    assert_eq!(typed(true), (false, 0));
}

#[test]
fn other_help_page_keys_act_from_the_stage_window_so_that_it_closes() {
    // The view hides the main window under the page, so that no field of
    // it takes the keys, and the update only decides which keys act
    let (open, _) = show(Kind::Video).staged();
    let stage = open.stage.window().expect("the stage window opens");
    let (keyboard, _) = App { help: true, ..open }.handled(Message::WindowFocused(stage));
    let keys = [
        press(Key::Named(Named::F1)),
        press(Key::Character("j".into())),
    ]
    .into_iter()
    .chain(field_keys())
    .chain([press(Key::Character("q".into()))]);
    let typed = keys
        .scan(keyboard, |app, key| {
            let (next, task) = std::mem::take(app).typed(key, false);
            let outcome = (next.help, task.units());
            *app = next;
            Some(outcome)
        })
        .collect::<Vec<_>>();
    // Help does nothing, scrolling takes a task, the keys that move between
    // fields, open and save shows, or go back and forward take none, and the
    // key of closing closes
    let expected = [
        (true, 0),
        (true, 1),
        (true, 0),
        (true, 0),
        (true, 0),
        (true, 0),
        (true, 0),
        (true, 0),
        (true, 0),
        (false, 0),
    ];
    assert_eq!(typed, expected);
}
