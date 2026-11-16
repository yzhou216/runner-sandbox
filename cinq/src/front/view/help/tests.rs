// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of which actions the help page lists

use super::{bound, counts, first_steps};
use crate::front::keymap::{Action, Keymap, Preset};

#[test]
fn actions_with_no_key_in_the_keymap_in_use_have_no_row() {
    let vi = bound(&Keymap::from(Preset::Vi));
    assert!(!vi.contains(&Action::Delete));
    assert!(vi.contains(&Action::Cut) && vi.contains(&Action::EditStart));
    let cua = bound(&Keymap::from(Preset::Cua));
    assert!(cua.contains(&Action::Delete));
    assert!(!cua.contains(&Action::HalfPageDown) && !cua.contains(&Action::SearchBackward));
}

#[test]
fn rows_keep_the_order_of_the_actions_with_go_and_panic_first() {
    let vi = bound(&Keymap::from(Preset::Vi));
    assert_eq!(vi[..3], [Action::Go, Action::Panic, Action::Standby]);
}

#[test]
fn first_steps_name_the_keys_of_the_keymap_in_use_in_at_most_five_lines() {
    let steps = |preset| first_steps(&Keymap::from(preset));
    let vi = steps(Preset::Vi);
    assert!(vi.contains("Space is GO") && vi.contains("Escape is Panic"));
    assert!(vi.contains("j and k move") && vi.contains("cc names it"));
    assert!(vi.contains("o adds a cue") && vi.contains("u undoes"));
    let cua = steps(Preset::Cua);
    assert!(cua.contains("ArrowDown and ArrowUp move") && cua.contains("F2 names it"));
    assert!(cua.contains("Control+n adds a cue") && cua.contains("Control+z undoes"));
    // The page is 960 pixels wide in a font 9 pixels wide
    assert!([&vi, &cua].iter().all(|steps| steps.len() <= 5 * 100));
}

#[test]
fn the_count_paragraph_names_the_outputs_that_a_count_acts_on() {
    let vi = counts(&Keymap::from(Preset::Vi));
    assert!(vi.contains("adds that many cues or outputs") && vi.contains("as in 3 j"));
}
