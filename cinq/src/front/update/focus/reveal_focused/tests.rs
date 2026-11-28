// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of scrolling the sends to the field that Tab reached

use super::{
    super::scrolled::{at, scrolled_to, visit},
    *,
};
use crate::front::fixtures::Field;

/// Visits a window as iced does, with a field outside the scrollable `id` and
/// one inside it, and returns where it scrolls to, if it does, with `label`
/// above each field.  The content starts 300 pixels down and the view shows
/// 100 of it from `scrolled` on.
fn scrolling_in(
    id: &'static str,
    label: f32,
    outside: bool,
    field_y: f32,
    scrolled: f32,
) -> Option<f32> {
    let mut operation = Revealing::new(id, label);
    Operation::focusable(
        &mut operation,
        None,
        at(20.0, 30.0),
        &mut Field::new(outside),
    );
    let id = Id::new(id);
    visit(&mut operation, &id, scrolled);
    Operation::traverse(&mut operation, &mut |inner: &mut dyn Operation| {
        inner.focusable(None, at(field_y, 30.0), &mut Field::new(!outside));
    });
    scrolled_to(Operation::finish(&operation), &id, scrolled)
}

/// Scrolls the sends, each of which lies in one row with its name, so that
/// no label shows above its field
fn scrolling(outside: bool, field_y: f32, scrolled: f32) -> Option<f32> {
    scrolling_in(view::SENDS, 0.0, outside, field_y, scrolled)
}

#[test]
fn the_sends_scroll_only_to_a_field_of_theirs_that_lies_out_of_view() {
    // Field 450 pixels down is 150 below the start of the content, and
    // ends at 180, which a view of 100 shows from 80 on
    assert_eq!(scrolling(false, 450.0, 0.0), Some(80.0));
    // Above the view, the field scrolls down to its own top, with no room
    // spared for a label that is not there
    assert_eq!(scrolling(false, 350.0, 200.0), Some(50.0));
    // In view, or outside the sends, they stay where they are
    assert_eq!(scrolling(false, 310.0, 0.0), None);
    assert_eq!(scrolling(true, 450.0, 0.0), None);
}

#[test]
fn a_list_of_the_sidebar_below_the_view_scrolls_the_sidebar_to_its_title() {
    // Title and the gap below it show with the list, which leaves the same
    // 80 pixels as for any field below the view
    let title = view::line_height() + view::SPACING;
    let sidebar =
        |field_y, scrolled| scrolling_in(view::SIDEBAR_SCROLL, title, false, field_y, scrolled);
    assert_eq!(sidebar(450.0, 0.0), Some(80.0));
    assert_eq!(sidebar(350.0, 200.0), Some(50.0 - title));
    assert_eq!(sidebar(300.0 + title + 10.0, 0.0), None, "in view");
}
