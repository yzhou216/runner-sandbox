// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stand-in for a scrollable, for the tests of the operations that scroll one

use iced::{
    Point, Rectangle, Size, Vector,
    advanced::widget::{Id, Operation, operation::Outcome},
};

use crate::front::fixtures::Scrolled;

/// Returns the rectangle of 80 pixels across that starts at `y`
pub(super) fn at(y: f32, height: f32) -> Rectangle {
    Rectangle::new(Point::new(10.0, y), Size::new(80.0, height))
}

/// Returns the view of the scrollable and its content, which starts 300
/// pixels down and is 1000 high, where the view shows 100 of it
pub(super) fn viewport() -> (Rectangle, Rectangle) {
    (at(300.0, 100.0), at(300.0, 1000.0))
}

/// Runs what `outcome` chains to over the scrollable `id`, as iced does, once
/// the content has scrolled by `scrolled`, and returns where it scrolls to,
/// if it does
pub(super) fn scrolled_to(outcome: Outcome<()>, id: &Id, scrolled: f32) -> Option<f32> {
    let Outcome::Chain(mut chained) = outcome else {
        return None;
    };
    let (bounds, content) = viewport();
    let mut list = Scrolled::default();
    chained.scrollable(
        Some(id),
        bounds,
        content,
        Vector::new(0.0, scrolled),
        &mut list,
    );
    list.0
}

/// Visits a scrollable `id` with `operation`, as iced does before the
/// operation finishes
pub(super) fn visit(operation: &mut dyn Operation, id: &Id, scrolled: f32) {
    let (bounds, content) = viewport();
    operation.scrollable(
        Some(id),
        bounds,
        content,
        Vector::new(0.0, scrolled),
        &mut Scrolled::default(),
    );
}
