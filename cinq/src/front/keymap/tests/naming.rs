// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that enter the name of the selected cue

use super::*;

#[test]
fn keys_of_vim_enter_the_name_at_its_end_or_start_or_change_all_of_it() {
    assert_actions(&[
        ("i", Action::Edit, None),
        ("a", Action::Edit, None),
        ("A", Action::Edit, None),
        ("I", Action::EditStart, None),
        ("S", Action::ChangeName, None),
        ("c c", Action::ChangeName, None),
        ("2 c c", Action::ChangeName, Some(2)),
    ]);
    assert!(typed("c").pending.len() == 1 && typed("c").action.is_none());
    // First c is no operator, so a key after it drops both
    assert_eq!(typed("c j"), Outcome::default());
}

#[test]
fn vi_lists_the_keys_that_enter_the_name_as_the_help_page_shows_them() {
    assert_eq!(listed(Action::Edit), "i, a, A");
    assert_eq!(listed(Action::ChangeName), "cc, S");
}
