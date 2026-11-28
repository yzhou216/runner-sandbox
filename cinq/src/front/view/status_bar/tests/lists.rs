// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints while a drop-down list or a checkbox has the keyboard

use super::{added, clipped, clipping, deleted, held, keyed, opened_with, texts};
use crate::{
    back::Command,
    front::{
        App,
        app::{Held, Keyboard},
        keymap::{Action, Keymap, desk, sequence},
    },
};

#[test]
fn a_list_hints_undo_but_not_the_rename_of_the_cue_just_added() {
    let list = App {
        workspace: added(),
        ..held(Keymap::default(), false)
    };
    assert_eq!(texts(&list)[0], "Added cue 1 | u: undo");
    assert!(texts(&list).iter().all(|hint| !hint.contains("rename")));
}

#[test]
fn a_list_hints_the_key_to_add_an_output_but_not_to_rename_one_nor_paste() {
    let outputs = App {
        outputs_shown: true,
        ..held(Keymap::default(), false)
    };
    assert!(texts(&outputs).contains(&"o: add output".to_owned()));
    assert!(texts(&outputs).iter().all(|hint| !hint.contains("rename")));
    // Undo acts in a list, where paste does not
    let cut = |text| App {
        keyboard: Keyboard::default().with_held(Held {
            any: true,
            text,
            ..Held::default()
        }),
        ..clipping("Cut cue 1", true, deleted())
    };
    assert_eq!(clipped(&cut(false)).as_deref(), Some("Cut cue 1 | u: undo"));
    assert_eq!(clipped(&cut(true)).as_deref(), Some("Cut cue 1"));
}

#[test]
fn the_show_mode_check_box_hints_the_next_field_go_and_panic_in_show_mode_too() {
    [Keymap::default(), desk("")]
        .into_iter()
        .for_each(|keymap| {
            let app = App {
                workspace: added().apply(Command::ShowMode(true)),
                ..held(keymap, false)
            };
            let hints = texts(&app);
            assert!(
                hints.iter().any(|hint| hint.contains("next field")),
                "{hints:?}"
            );
            assert!(hints.iter().any(|hint| hint.contains("GO")), "{hints:?}");
            assert!(hints.iter().any(|hint| hint.contains("Panic")), "{hints:?}");
            assert!(
                hints
                    .iter()
                    .all(|hint| !hint.contains("rename") && !hint.contains("standby")),
                "{hints:?}"
            );
        });
}

/// Returns the hint that tells what Enter does in a field of `app`, if any,
/// which is not the hint of a key that a user binds to Enter
fn enter(app: &App) -> Option<String> {
    texts(app).into_iter().find(|hint| {
        matches!(
            hint.as_str(),
            "Enter: flip" | "Enter: press" | "Enter: open" | "Enter: leave"
        )
    })
}

fn checked(keymap: Keymap) -> App {
    keyed(held(keymap, false), |keyboard| {
        keyboard.with_held(Held {
            any: true,
            check: true,
            ..Held::default()
        })
    })
}

#[test]
fn enter_is_hinted_to_flip_a_check_box_to_open_a_list_and_to_press_a_button() {
    for keymap in [Keymap::default(), desk("")] {
        let button = keyed(held(keymap.clone(), false), |keyboard| {
            keyboard.with_held(Held {
                any: true,
                button: true,
                ..Held::default()
            })
        });
        assert_eq!(
            enter(&checked(keymap.clone())).as_deref(),
            Some("Enter: flip")
        );
        assert_eq!(enter(&held(keymap, false)).as_deref(), Some("Enter: open"));
        assert_eq!(enter(&button).as_deref(), Some("Enter: press"));
        assert!(texts(&button).iter().all(|hint| hint != "Enter: open"));
    }
}

#[test]
fn enter_is_hinted_in_no_text_field_and_where_no_field_has_the_keyboard() {
    // Enter leaves a text field where the keymap binds no key to cancel, and its own hint says so
    assert_eq!(
        enter(&held(desk(""), true)).as_deref(),
        Some("Enter: leave")
    );
    assert_eq!(enter(&held(Keymap::default(), true)), None);
    assert_eq!(enter(&opened_with(desk(""), false, false)), None);
    let menu = keyed(checked(Keymap::default()), |keyboard| {
        keyboard.with_menu(true)
    });
    assert_eq!(enter(&menu), None);
    let help = App {
        help: true,
        ..checked(Keymap::default())
    };
    assert_eq!(enter(&help), None);
}

#[test]
fn enter_is_not_hinted_to_open_or_flip_where_a_key_binding_takes_it_over() {
    let mut app = checked(desk(""));
    let key = vec![sequence("Enter").expect("a key")];
    app.config.keymap = app.config.keymap.clone().with(Action::Go, key);
    assert_eq!(enter(&app), None);
}

#[test]
fn go_is_hinted_over_a_text_field_only_by_a_key_that_types_no_character() {
    let go = |app: &App| texts(app).into_iter().find(|hint| hint.ends_with(": GO"));
    for keymap in [Keymap::default(), desk("")] {
        assert_eq!(
            go(&held(keymap.clone(), false)).as_deref(),
            Some("Space: GO")
        );
        assert_eq!(go(&held(keymap.clone(), true)), None);
    }
    let mut app = held(Keymap::default(), true);
    let keys = vec![
        sequence("Space").expect("a key"),
        sequence("F9").expect("a key"),
    ];
    app.config.keymap = app.config.keymap.clone().with(Action::Go, keys);
    assert_eq!(go(&app).as_deref(), Some("F9: GO"));
}

#[test]
fn a_list_hints_the_key_of_hint_mode_but_a_text_field_does_not() {
    [Keymap::default(), desk("")]
        .into_iter()
        .for_each(|keymap| {
            assert!(texts(&held(keymap.clone(), false)).contains(&"f: hint".to_owned()));
            assert!(!texts(&held(keymap, true)).contains(&"f: hint".to_owned()));
        });
}
