// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which keys the bar hints where a field has the keyboard

use super::*;

#[test]
fn the_help_key_is_hinted_only_where_it_acts() {
    let told = |app: App| has_hint(&app, "?: help");
    assert!(told(opened(false, false)));
    assert!(!told(held(Keymap::default(), true)));
    assert!(!told(held(Keymap::default(), false)));
    assert!(told(App {
        keyboard: holding(true).with_menu(true),
        ..opened(true, false)
    }));
    assert!(told(App {
        keyboard: Keyboard::default().with_held(Held {
            any: true,
            text: true,
            ..Held::default()
        }),
        ..opened(false, true)
    }));
}

#[test]
fn the_bar_hints_enter_while_the_empty_file_field_has_the_keyboard() {
    let told = |file: &str, file_held: bool| {
        let keyboard = Keyboard::default().with_held(Held {
            any: true,
            file: file_held,
            ..Held::default()
        });
        let app = holding_file(Keymap::default(), file, keyboard);
        has_hint(&app, "Enter: choose a file")
    };
    assert!(told("", true));
    assert!(!told("rain.flac", true), "a named file needs no chooser");
    assert!(!told("", false), "another field has the keyboard");
}

#[test]
fn a_keymap_with_no_key_to_cancel_hints_enter_to_leave_a_text_field_only() {
    let told = |text_held| has_hint(&held(desk(""), text_held), "Enter: leave");
    assert!(told(true));
    assert!(!told(false), "Enter opens the menu of a drop-down list");
}

#[test]
fn enter_is_not_hinted_to_leave_the_empty_file_field_where_it_chooses_a_file() {
    let told = |keymap, file: &str| {
        let keyboard = Keyboard::default().with_held(Held {
            any: true,
            file: true,
            text: true,
            ..Held::default()
        });
        texts(&holding_file(keymap, file, keyboard))
    };
    let empty = told(desk(""), "");
    assert!(empty.contains(&"Enter: choose a file".to_owned()));
    assert!(!empty.contains(&"Enter: leave".to_owned()));
    assert!(told(desk(""), "rain.flac").contains(&"Enter: leave".to_owned()));
}

#[test]
fn the_key_of_the_next_tab_is_hinted_late_while_no_field_has_the_keyboard() {
    let hints = |held| {
        hinted(&App {
            keyboard: holding(held),
            ..opened(false, false)
        })
    };
    assert_eq!(
        hints(false)
            .iter()
            .rev()
            .nth(2)
            .map(|(_, hint)| hint.as_str()),
        Some("gt: next tab")
    );
    assert!(
        hints(true)
            .iter()
            .all(|(_, hint)| !hint.contains("next tab"))
    );
}

#[test]
fn enter_is_not_hinted_to_leave_where_a_key_binding_takes_it_over() {
    let told = |action| {
        let mut app = held(desk(""), true);
        let enter = vec![sequence("Enter").expect("a key")];
        app.config.keymap = app.config.keymap.clone().with(action, enter);
        has_hint(&app, "Enter: leave")
    };
    assert!(!told(Action::Go));
    assert!(!told(Action::Panic));
    assert!(told(Action::Pick), "pick acts in a menu and not in a field");
}
