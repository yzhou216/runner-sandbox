// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the hints while hint mode is on

use super::{opened_with, texts};
use crate::front::{
    App,
    keymap::{Keymap, desk},
    update::Hints,
};

fn labelling(keymap: Keymap) -> Vec<String> {
    texts(&App {
        hints: Some(Hints::default()),
        ..opened_with(keymap, false, false)
    })
}

#[test]
fn hint_mode_hints_only_what_acts_there_and_backspace_leaves_where_no_key_cancels() {
    assert_eq!(
        labelling(Keymap::default()),
        [
            "Type a label",
            "Control+c: cancel",
            "Space: GO",
            "Escape: Panic"
        ]
    );
    assert_eq!(
        labelling(desk(""))[..2],
        ["Type a label", "Backspace: leave"]
    );
}

#[test]
fn backspace_takes_a_letter_back_once_one_is_typed() {
    let typed = |keymap| {
        texts(&App {
            hints: Some(Hints::after("s")),
            ..opened_with(keymap, false, false)
        })
        .into_iter()
        .take(2)
        .collect::<Vec<_>>()
    };
    assert_eq!(typed(desk("")), ["s", "Backspace: back"]);
    assert_eq!(typed(Keymap::default()), ["s", "Control+c: cancel"]);
}
