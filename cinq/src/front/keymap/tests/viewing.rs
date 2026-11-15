// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that select a row in view and put the selected row
//! in view

use super::*;

#[test]
fn z_waits_for_the_key_that_puts_the_selected_row_in_view() {
    assert_eq!(typed("z").pending.len(), 1);
    assert_eq!(action("z z"), Some((Action::RowToMiddle, None)));
    assert_eq!(action("z t"), Some((Action::RowToTop, None)));
    assert_eq!(action("z b"), Some((Action::RowToBottom, None)));
    assert_eq!(Keymap::default().conflicts(), Vec::<String>::new());
    assert_eq!(
        Keymap::from(Preset::Cua).keys(Action::RowToTop),
        [] as [Vec<Press>; 0]
    );
}

#[test]
fn a_key_after_z_that_puts_no_row_in_view_drops_the_z_and_itself() {
    let dropped = Outcome::default();
    assert_eq!(typed("z w"), dropped);
    assert_eq!(typed("z j"), dropped);
    assert_eq!(typed("3 z w"), dropped);
    // The next key starts afresh
    assert_eq!(action("z w j"), Some((Action::Next, None)));
}

#[test]
fn h_m_and_l_select_the_rows_in_view_and_take_the_count_of_h_and_l() {
    assert_eq!(action("H"), Some((Action::TopRow, None)));
    assert_eq!(action("M"), Some((Action::MiddleRow, None)));
    assert_eq!(action("3 L"), Some((Action::BottomRow, Some(3))));
}
