// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that select a row in view or put it there

use super::*;

#[test]
fn z_waits_for_the_key_that_puts_the_selected_row_in_view_which_may_be_enter_dot_or_minus() {
    assert_eq!(typed("z").pending.len(), 1);
    assert_actions(&[
        ("z z", Action::RowToMiddle, None),
        ("z t", Action::RowToTop, None),
        ("z b", Action::RowToBottom, None),
        ("z Enter", Action::RowToTop, None),
        ("z .", Action::RowToMiddle, None),
        ("z -", Action::RowToBottom, None),
        // Count goes with them, and Enter alone stays the key of standby
        ("5 z .", Action::RowToMiddle, Some(5)),
        ("Enter", Action::Standby, None),
    ]);
}

#[test]
fn h_m_and_l_select_the_rows_in_view_and_take_the_count_of_h_and_l() {
    assert_eq!(action("H"), Some((Action::TopRow, None)));
    assert_eq!(action("M"), Some((Action::MiddleRow, None)));
    assert_eq!(action("3 L"), Some((Action::BottomRow, Some(3))));
}

#[test]
fn plus_and_minus_move_to_the_next_and_previous_cue_as_in_vim() {
    assert_eq!(action("+"), Some((Action::Next, None)));
    assert_eq!(action("3 -"), Some((Action::Previous, Some(3))));
    // And they are motions after an operator, as the other keys of the moves
    assert_eq!(
        typed("d +").across(),
        Some((Action::Cut, Action::Next, None))
    );
}
