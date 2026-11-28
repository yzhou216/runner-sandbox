// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the width of an open menu in a window narrower than its names

use super::super::overlay::{columns, width};
use crate::front::view::{char_width, widget::dropdown::PADDING};

#[test]
fn a_menu_takes_the_width_of_its_names_but_never_more_than_the_window() {
    let names = 80.0 * char_width();
    assert_eq!(width(100.0, names, 2.0 * names), names + PADDING.x());
    assert_eq!(width(100.0, names, 300.0), 300.0);
}

#[test]
fn a_menu_is_at_least_as_wide_as_its_list_where_the_window_allows() {
    assert_eq!(width(200.0, 50.0, 400.0), 200.0);
    assert_eq!(width(500.0, 50.0, 400.0), 400.0);
}

#[test]
fn the_names_of_a_narrow_menu_keep_the_columns_that_its_width_holds() {
    let window = 300.0;
    let kept = columns(width(100.0, 80.0 * char_width(), window));
    assert!(kept >= 1 && (kept as f32) * char_width() + PADDING.x() <= window);
    assert_eq!(columns(0.0), 0);
}
