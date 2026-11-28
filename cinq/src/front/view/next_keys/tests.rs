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
        line("Cut the selected cue and the next"),
        Some("j, gj, ArrowDown, Control+n, +")
    );
    assert_eq!(
        line("Cut the selected cue and the previous"),
        Some("k, gk, ArrowUp, Control+p, -")
    );
    assert_eq!(line("Cut the selected cue to the clipboard"), Some("d"));
}

#[test]
fn a_prefix_lists_each_key_that_completes_a_binding() {
    let lines = listed(&after(&["g"]));
    assert!(lines.contains(&(
        "g".to_owned(),
        "Select the first cue or menu option, or the row counted".to_owned()
    )));
}

#[test]
fn the_box_names_outputs_while_they_show_and_cues_while_a_cue_list_does() {
    let outputs = App {
        outputs_shown: true,
        ..after(&["d"])
    };
    let lines = listed(&outputs);
    let has = |description: &str| lines.iter().any(|(_, shown)| shown == description);
    assert!(has("Cut the selected output and the next"));
    assert!(has("Cut the selected output to the clipboard"));
    assert!(!has("Cut the selected cue to the clipboard"));
    let cues = listed(&after(&["d"]));
    assert!(cues.iter().all(|(_, shown)| !shown.contains("output")));
}

#[test]
fn the_line_of_the_key_typed_again_comes_first() {
    let lines = listed(&after(&["g"]));
    assert_eq!(lines.first().map(|(keys, _)| keys.as_str()), Some("g"));
    assert_eq!(
        lines
            .iter()
            .skip(1)
            .map(|(keys, _)| keys.as_str())
            .collect::<Vec<_>>(),
        ["j", "k", "t", "T"]
    );
    let after_d = listed(&after(&["d"]));
    assert_eq!(after_d.first().map(|(keys, _)| keys.as_str()), Some("d"));
}

#[test]
fn the_doubled_line_counts_the_rows_that_the_count_takes() {
    let line = |keys: &[&str]| {
        listed(&after(keys))
            .into_iter()
            .find(|(shown, _)| shown == "d")
            .map(|(_, description)| description)
    };
    assert_eq!(
        line(&["3", "d"]).as_deref(),
        Some("Cut the selected cue and the next 2")
    );
    assert_eq!(
        line(&["d"]).as_deref(),
        Some("Cut the selected cue to the clipboard")
    );
    assert_eq!(
        line(&["1", "d"]).as_deref(),
        Some("Cut the selected cue to the clipboard")
    );
}
