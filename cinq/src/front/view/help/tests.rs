// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which actions the help page lists

use super::{bound, counts, first_steps, names_fit};
use crate::front::{
    fixtures::{in_one_run, wide},
    keymap::{Action, Keymap, declared, desk},
};

#[test]
fn actions_with_no_key_in_the_keymap_in_use_have_no_row() {
    let vi = bound(&Keymap::default());
    assert!(!vi.contains(&Action::Delete));
    assert!(vi.contains(&Action::Cut) && vi.contains(&Action::EditStart));
    let own = bound(&declared("delete = \"Delete\"\nnext = \"ArrowDown\""));
    assert!(own.contains(&Action::Delete));
    assert!(!own.contains(&Action::HalfPageDown) && !own.contains(&Action::SearchBackward));
}

#[test]
fn rows_keep_the_order_of_the_actions_with_go_and_panic_first() {
    let vi = bound(&Keymap::default());
    assert_eq!(vi[..3], [Action::Go, Action::Panic, Action::Standby]);
}

#[test]
fn first_steps_name_the_keys_of_the_keymap_in_use_in_at_most_five_lines() {
    let vi = first_steps(&Keymap::default());
    assert!(vi.contains("Space is GO") && vi.contains("Escape is Panic"));
    assert!(vi.contains("j and k move") && vi.contains("cc names it"));
    assert!(vi.contains("o adds a cue") && vi.contains("u undoes"));
    assert!(vi.contains("q closes this page") && vi.contains("Control+s saves the show"));
    assert!(
        vi.contains("Tab moves through the fields to the File field, where Enter chooses a file")
            && vi.contains("f labels each place to click")
    );
    let own = first_steps(&desk(
        "next = \"ArrowDown\"\nprevious = \"ArrowUp\"\nchange_name = \"F2\"\nadd_below = \"Control+a\"\nundo = \"Control+z\"\nclose_help = \"Control+w\"",
    ));
    assert!(own.contains("ArrowDown and ArrowUp move") && own.contains("F2 names it"));
    assert!(own.contains("Control+a adds a cue") && own.contains("Control+z undoes"));
    assert!(own.contains("Control+w closes this page"));
    assert!(own.contains("Tab moves through the fields to the File field"));
    // Page is 960 pixels wide in a font 9 pixels wide
    assert!([&vi, &own].iter().all(|steps| steps.len() <= 5 * 100));
}

#[test]
fn only_a_keymap_with_sequences_says_how_to_type_them() {
    assert!(Keymap::default().has_sequences());
    assert!(!declared("next = \"ArrowDown\"").has_sequences());
}

#[test]
fn the_count_paragraph_names_the_outputs_that_a_count_acts_on() {
    let vi = counts(&Keymap::default());
    assert!(vi.contains("adds that many cues or outputs") && vi.contains("as in 3 j"));
}

#[test]
fn the_names_in_the_configuration_file_give_way_in_a_window_too_narrow_for_them() {
    [Keymap::default(), desk("")].iter().for_each(|keymap| {
        assert!(names_fit(keymap, wide(160.0)), "{}", keymap.name());
        assert!(!names_fit(keymap, wide(60.0)), "{}", keymap.name());
        // Every width from the narrowest that keeps the names keeps them
        in_one_run(60..160, |room| names_fit(keymap, wide(room)));
    });
}

#[test]
fn the_keys_of_a_menu_follow_cancel_and_precede_the_moves_that_act_there_too() {
    let vi = bound(&Keymap::default());
    let at = |action| vi.iter().position(|&listed| listed == action).unwrap();
    assert_eq!(at(Action::Pick), at(Action::Cancel) + 1);
    assert_eq!(at(Action::CloseMenu), at(Action::Pick) + 1);
    assert_eq!(at(Action::Next), at(Action::CloseMenu) + 1);
}
