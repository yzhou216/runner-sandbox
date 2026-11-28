// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of a key that ends no command, which drops the keys typed so far

use super::*;

#[test]
fn key_that_ends_no_command_drops_the_keys_typed_so_far_and_itself() {
    [
        // After g, after z, and after an operator, with counts around them
        "g w", "4 w", "2 g w", "z w", "z j", "3 z w", "d i", "d z", "2 d 3 z", "d g z", "d u",
        "d x", "d p", "d o", "d l", "d g t", "y x", "y p",
    ]
    .into_iter()
    .for_each(|text| assert_eq!(typed(text), Outcome::default(), "{text}"));
    // Next key starts afresh
    assert_actions(&[
        ("g w j", Action::Next, None),
        ("2 g j", Action::Next, Some(2)),
        ("z w j", Action::Next, None),
        ("d x j", Action::Next, None),
    ]);
}

#[test]
fn g_j_and_g_k_move_the_selection_as_j_and_k_do_for_a_vim_user_who_types_them() {
    assert_actions(&[
        ("g j", Action::Next, None),
        ("3 g k", Action::Previous, Some(3)),
    ]);
    assert_eq!(listed(Action::Next), "j, gj, ArrowDown, Control+n, +");
}

#[test]
fn digits_past_the_largest_count_are_not_kept() {
    let digits = "9 ".repeat(100);
    assert_eq!(typed(&digits).pending.len(), 20);
    let most = format!("{digits} j");
    assert_eq!(action(&most), Some((Action::Next, Some(usize::MAX))));
    // After an operator, the second count saturates the same way
    assert_eq!(typed(&format!("d {digits}")).pending.len(), 21);
    // Key that is no digit still ends the keys
    assert_eq!(typed(&format!("{digits} w")), Outcome::default());
}

#[test]
fn a_zero_that_starts_a_sequence_is_dropped_and_one_after_a_digit_counts() {
    assert_eq!(typed("0"), Outcome::default());
    assert_eq!(typed("0 j"), typed("j"));
    assert_eq!(action("1 0 j"), Some((Action::Next, Some(10))));
    // The zero after a digit stays in the keys that wait
    assert_eq!(typed("1 0").pending.len(), 2);
    // Zero that would start the count after an operator drops the operator
    assert_eq!(typed("d 0"), Outcome::default());
    assert_eq!(typed("2 d 0"), Outcome::default());
    assert_eq!(typed("d 1 0 j").motion, Some((Action::Next, Some(10))));
    assert_eq!(typed("d 1 0").pending.len(), 3);
}
