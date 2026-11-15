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
    // The first c is no operator, so a key after it drops both
    assert_eq!(typed("c j"), Outcome::default());
    assert_eq!(edit("2 c c"), Some(Action::ChangeName));
}

#[test]
fn keys_that_enter_the_name_at_its_start_or_over_all_of_it_have_no_cua_key() {
    let keymap = Keymap::from(Preset::Cua);
    assert_eq!(keymap.listed(Action::Edit), "F2");
    assert_eq!(keymap.listed(Action::EditStart), "none");
    assert_eq!(keymap.listed(Action::ChangeName), "none");
}
