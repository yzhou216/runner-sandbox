// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what the box of the keys that can follow lists

use iced::keyboard::{Key, Modifiers};

use super::listed;
use crate::front::{App, keymap::Press};

fn after(keys: &[&str]) -> App {
    App {
        keys: keys
            .iter()
            .filter_map(|key| Press::from_key(&Key::Character((*key).into()), Modifiers::empty()))
            .collect(),
        ..App::default()
    }
}

#[test]
fn the_keys_of_one_action_share_a_line_after_an_operator() {
    let lines = listed(&after(&["d"]));
    let line = |description: &str| {
        lines
            .iter()
            .find(|(_, shown)| shown == description)
            .map(|(keys, _)| keys.as_str())
    };
    assert_eq!(
        line("Cut the selected cue or output and the next"),
        Some("j, g j, ArrowDown, Control+n, +")
    );
    assert_eq!(
        line("Cut the selected cue or output and the previous"),
        Some("k, g k, ArrowUp, Control+p, -")
    );
    assert_eq!(
        line("Cut the selected cue, or output, to the clipboard"),
        Some("d")
    );
}

#[test]
fn a_prefix_lists_each_key_that_completes_a_binding() {
    let lines = listed(&after(&["g"]));
    assert!(lines.contains(&(
        "g".to_owned(),
        "Select the first cue, output, or menu option, or the row counted".to_owned()
    )));
}
