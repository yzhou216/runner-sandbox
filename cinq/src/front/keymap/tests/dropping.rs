// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of a key that completes or continues no command, which drops the
//! keys typed so far, and of the keys of Vim for the next and previous row

use super::*;

#[test]
fn key_that_ends_no_command_drops_the_keys_typed_so_far_and_itself() {
    let dropped = Outcome::default();
    assert_eq!(typed("d i"), dropped);
    assert_eq!(typed("g w"), dropped);
    assert_eq!(typed("4 w"), dropped);
    assert_eq!(typed("2 g w"), dropped);
    // The next key starts afresh
    assert_eq!(action("g w j"), Some((Action::Next, None)));
    assert_eq!(action("2 g j"), Some((Action::Next, Some(2))));
}

#[test]
fn g_j_and_g_k_move_the_selection_as_j_and_k_do_for_a_vim_user_who_types_them() {
    assert_eq!(action("g j"), Some((Action::Next, None)));
    assert_eq!(action("3 g k"), Some((Action::Previous, Some(3))));
    assert_eq!(
        Keymap::default().listed(Action::Next),
        "j, g j, ArrowDown, Control+n"
    );
}
