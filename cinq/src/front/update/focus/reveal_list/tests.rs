// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of scrolling the sidebar back to its current list

use super::{
    super::scrolled::{at, scrolled_to, visit},
    *,
};

/// Visits a sidebar whose content starts 300 pixels down and whose view shows
/// 100 of it from `scrolled` on, with the current list `entry_y` pixels down,
/// and returns where it scrolls to, if it does
fn scrolling(entry_y: f32, scrolled: f32) -> Option<f32> {
    let id = Id::new(view::SIDEBAR_SCROLL);
    let mut operation = Current::default();
    visit(&mut operation, &id, scrolled);
    operation.container(Some(&Id::new(view::CURRENT_LIST)), at(entry_y, 20.0));
    scrolled_to(operation.finish(), &id, scrolled)
}

#[test]
fn a_sidebar_scrolled_past_its_current_list_scrolls_back_to_it() {
    // List 20 pixels down lies above a view that starts 500 down, and the
    // gap of the column stays above it
    assert_eq!(scrolling(320.0, 500.0), Some(20.0 - view::SPACING));
}

#[test]
fn a_current_list_in_view_leaves_the_sidebar_where_it_is() {
    assert_eq!(scrolling(320.0, 0.0), None);
}
