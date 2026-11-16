// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that enter the name of the selected cue

use super::*;

#[test]
fn keys_of_vim_enter_the_name_at_its_end_or_start_or_change_all_of_it() {
    let edit = |text: &str| action(text).map(|(action, _)| action);
    assert_eq!(edit("i"), Some(Action::Edit));
    assert_eq!(edit("a"), Some(Action::Edit));
    assert_eq!(edit("A"), Some(Action::Edit));
    assert_eq!(edit("I"), Some(Action::EditStart));
    assert_eq!(edit("S"), Some(Action::ChangeName));
    assert_eq!(edit("c c"), Some(Action::ChangeName));
    assert!(typed("c").pending.len() == 1 && typed("c").action.is_none());
    // First c is no operator, so a key after it drops both
    assert_eq!(typed("c j"), Outcome::default());
    assert_eq!(edit("2 c c"), Some(Action::ChangeName));
}

#[test]
fn cua_selects_the_name_with_f2_as_renaming_does_and_has_no_key_for_its_end_or_start() {
    let keymap = Keymap::from(Preset::Cua);
    assert_eq!(keymap.listed(Action::ChangeName), "F2");
    assert_eq!(keymap.listed(Action::Edit), "none");
    assert_eq!(keymap.listed(Action::EditStart), "none");
}

#[test]
fn vi_keeps_the_caret_at_the_end_for_the_keys_that_enter_the_name() {
    let keymap = Keymap::from(Preset::Vi);
    assert_eq!(keymap.listed(Action::Edit), "i, a, A");
    assert_eq!(keymap.listed(Action::ChangeName), "cc, S");
}
