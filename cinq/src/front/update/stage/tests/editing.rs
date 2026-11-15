// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that edit the show, which the stage window leaves to
//! the main window

use iced::{
    keyboard::{Key, Modifiers, key::Named},
    window,
};

use super::show;
use crate::{
    back::{CueId, Kind},
    front::{
        App, Message,
        update::tests::{chord, press},
    },
};

/// Returns the state of the interface after the window that `main` picks
/// takes the keyboard, with the only cue selected
fn focused(main: bool) -> App {
    let (open, _) = show(Kind::Video).staged();
    let window = if main { open.main } else { open.stage.window() };
    let open = App {
        selected: Some(CueId(1)),
        ..open
    };
    open.handled(Message::WindowFocused(window.expect("a window opens")))
        .0
}

/// Types the keys one after another
fn keyed(app: App, keys: &[&str]) -> App {
    keys.iter().fold(app, |app, key| {
        let press = match *key {
            "Delete" => press(Key::Named(Named::Delete)),
            "Control+r" => chord(Key::Character("r".into()), Modifiers::CTRL),
            text => press(Key::Character(text.into())),
        };
        app.typed(press, false).0
    })
}

/// Returns the number of cues of the first list and whether anything went
/// to the clipboard
fn found(app: &App) -> (usize, bool) {
    (app.workspace.lists()[0].cues.len(), app.clipped.is_some())
}

#[test]
fn keys_that_edit_the_show_do_nothing_while_the_stage_window_has_the_keyboard() {
    [
        &["o"][..],
        &["O"],
        &["x"],
        &["Delete"],
        &["Y"],
        &["p"],
        &["P"],
        &["d", "d"],
        &["y", "y"],
        &["d", "j"],
        &["y", "G"],
    ]
    .into_iter()
    .for_each(|keys| {
        assert_eq!(found(&keyed(focused(false), keys)), (1, false), "{keys:?}");
    });
}

#[test]
fn the_same_keys_act_from_the_main_window() {
    let from_main = |keys: &[&str]| found(&keyed(focused(true), keys));
    assert_eq!(from_main(&["o"]), (2, false));
    assert_eq!(from_main(&["Y"]), (1, true));
    assert_eq!(from_main(&["x"]), (0, true));
    assert_eq!(from_main(&["d", "d"]), (0, true));
}

#[test]
fn undo_and_redo_wait_for_the_main_window_too() {
    let added = keyed(focused(true), &["o"]);
    let (main, stage) = (added.main, added.stage.window());
    let focus = |app: App, window: Option<window::Id>| {
        app.handled(Message::WindowFocused(window.expect("a window opens")))
            .0
    };
    let undone = keyed(focus(added, stage), &["u"]);
    assert_eq!(found(&undone).0, 2);
    let undone = keyed(focus(undone, main), &["u"]);
    assert_eq!(found(&undone).0, 1);
    let redone = keyed(focus(undone, stage), &["Control+r"]);
    assert_eq!(found(&redone).0, 1);
}
