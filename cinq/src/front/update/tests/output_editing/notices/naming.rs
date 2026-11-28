// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that enter the name of an output, and of typed text that
//! a cut and a paste keep

use super::*;

#[test]
fn the_keys_that_enter_the_name_focus_the_field_of_the_output() {
    let started = |app: App, caret| app.handled(Message::Focus(Focus::Name(caret))).1.units();
    for caret in [Caret::End, Caret::Start, Caret::All] {
        assert!(started(on(2), caret) > 0, "{caret:?}");
        assert_eq!(started(locked(on(2)), caret), 0, "{caret:?}");
    }
    let hidden = App {
        output: None,
        ..on(2)
    };
    assert_eq!(started(hidden, Caret::End), 0);
}

#[test]
fn the_key_that_changes_the_name_selects_the_name_of_the_output_and_tab_selects_the_next_field() {
    assert!(matches!(
        Action::ChangeName.message(None),
        Message::Focus(Focus::Name(Caret::All))
    ));
    // Moving by key starts the focus and then the selection of the text
    for focus in [Focus::Next, Focus::Previous] {
        assert!(
            on(2).handled(Message::Focus(focus.clone())).1.units() > 1,
            "{focus:?}"
        );
    }
}

/// Types the keys as the window delivers them, one message each, which the
/// update cycle settles typed text for, unlike [`typed`]
fn keys(app: App, keys: &str) -> App {
    keys.chars().fold(app, |app, key| {
        let press = press(letter(&key.to_string()));
        app.next(Message::Typed {
            press,
            repeat: false,
        })
        .0
    })
}

#[test]
fn channels_typed_before_a_cut_apply_to_the_output_that_is_cut_and_pasted_back() {
    let sub = on(3).workspace.outputs()[2].id;
    let typing = typed_setting(on(3), Setting::Channels(sub), "5-6");
    assert_eq!(typing.workspace.outputs()[2].channels.to_string(), "1-2");
    // The key that cuts settles the text, and nothing of it is left to
    // apply to the output that has gone
    let cut = keys(typing, "dd");
    assert_eq!(cut.typed, None);
    assert_eq!(names(&cut), without(&[2]));
    let back = keys(cut, "P");
    let pasted = back
        .workspace
        .output(sub)
        .expect("the output took its identity back");
    assert_eq!(pasted.channels.to_string(), "5-6");
}
