// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of scrolling the sends to the field that Tab reached

use iced::{
    Point, Size,
    widget::operation::{AbsoluteOffset, RelativeOffset},
};

use super::*;

struct Field(bool);

impl Focusable for Field {
    fn is_focused(&self) -> bool {
        self.0
    }

    fn focus(&mut self) {}

    fn unfocus(&mut self) {}
}

/// Scrollable that tells where it was scrolled to
#[derive(Default)]
struct Scrolled(Option<f32>);

impl Scrollable for Scrolled {
    fn snap_to(&mut self, _offset: RelativeOffset<Option<f32>>) {}

    fn scroll_to(&mut self, offset: AbsoluteOffset<Option<f32>>) {
        self.0 = offset.y;
    }

    fn scroll_by(
        &mut self,
        _offset: AbsoluteOffset,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
    ) {
    }
}

fn at(y: f32, height: f32) -> Rectangle {
    Rectangle::new(Point::new(10.0, y), Size::new(80.0, height))
}

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
    Operation::focusable(&mut operation, None, at(20.0, 30.0), &mut Field(outside));
    let (bounds, content) = (at(300.0, 100.0), at(300.0, 1000.0));
    let mut list = Scrolled::default();
    Operation::scrollable(
        &mut operation,
        Some(&Id::new(id)),
        bounds,
        content,
        Vector::new(0.0, scrolled),
        &mut list,
    );
    Operation::traverse(&mut operation, &mut |inner: &mut dyn Operation| {
        inner.focusable(None, at(field_y, 30.0), &mut Field(!outside));
    });
    let Outcome::Chain(mut chained) = Operation::finish(&operation) else {
        return None;
    };
    chained.scrollable(
        Some(&Id::new(id)),
        bounds,
        content,
        Vector::new(0.0, scrolled),
        &mut list,
    );
    list.0
}

fn sends_label() -> f32 {
    view::line_height() + view::SPACING / 2.0
}

fn scrolling(outside: bool, field_y: f32, scrolled: f32) -> Option<f32> {
    scrolling_in(view::SENDS, sends_label(), outside, field_y, scrolled)
}

#[test]
fn a_field_below_the_view_scrolls_up_to_the_bottom_of_the_view() {
    // Field 450 pixels down is 150 below the start of the content, and
    // ends at 180, which a view of 100 shows from 80 on
    assert_eq!(scrolling(false, 450.0, 0.0), Some(80.0));
}

#[test]
fn a_field_above_the_view_scrolls_down_to_its_name() {
    assert_eq!(scrolling(false, 350.0, 200.0), Some(50.0 - sends_label()));
}

#[test]
fn a_field_in_view_leaves_the_sends_where_they_are() {
    assert_eq!(scrolling(false, 300.0 + sends_label() + 10.0, 0.0), None);
}

#[test]
fn a_field_outside_the_sends_leaves_them_where_they_are() {
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
