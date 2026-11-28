// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of paging, scrolling, and selecting by the rows of the outputs in view

use super::{
    super::{after, paging::typed},
    at, cue_list_kept, outputs,
};
use crate::front::{
    App, Message,
    message::Height,
    update::{Measured, Rows, rows::paging::viewport::Shape},
};

/// Returns the outputs after a measure of `rows` as ten rows of twenty
/// pixels in a view of five, scrolled by `scrolled` pixels
fn measure(rows: Rows, scrolled: f32) -> App {
    let app = outputs();
    let measured = Measured::Rows {
        serial: app.measures.serial(),
        rows,
        count: 10,
        shape: Shape {
            view: 100.0,
            content: 200.0,
            scrolled,
        },
    };
    after(app, [Message::Measured(measured)])
}

/// Returns the outputs measured as ten rows of twenty pixels in a view of
/// five, scrolled by `rows` rows
fn measured(rows: f32) -> App {
    measure(Rows::Outputs, rows * 20.0)
}

#[test]
fn half_pages_and_whole_pages_move_through_the_outputs_by_what_is_in_view() {
    // Five rows fit, so that half a page is two rows and a whole page three
    let (half, _) = typed(measured(0.0), "d", &[]);
    assert_eq!(at(&half), 3);
    let (back, _) = typed(half, "u", &[]);
    assert_eq!(at(&back), 1);
    let (whole, _) = typed(measured(0.0), "f", &[]);
    assert_eq!(at(&whole), 4);
    cue_list_kept(&whole);
}

#[test]
fn a_measure_of_the_cue_list_does_not_hold_for_the_outputs() {
    // Key waits for a measure of the outputs, and moves nothing yet
    let (waiting, started) = typed(measure(Rows::Cues(0), 0.0), "d", &[]);
    assert_eq!((at(&waiting), started > 0), (1, true));
}

#[test]
fn scrolling_and_the_rows_in_view_select_outputs_without_touching_the_cue_list() {
    let scrolled = after(measured(0.0), [Message::Scroll(2)]);
    // Selection goes to the first row that is still in view
    assert_eq!(at(&scrolled), 3);
    let row = |height| {
        let app = after(measured(2.0), [Message::InView(height, None)]);
        cue_list_kept(&app);
        at(&app)
    };
    // Rows 3 to 7 are in view
    assert_eq!(
        [Height::Top, Height::Middle, Height::Bottom].map(row),
        [3, 5, 7]
    );
    let placed = after(measured(0.0), [Message::Place(Height::Top, Some(6))]);
    assert_eq!(at(&placed), 6);
}

#[test]
fn the_rows_in_view_of_the_outputs_count_a_row_that_most_of_it_shows() {
    let row = |scrolled, height| {
        let app = after(measured(scrolled), [Message::InView(height, None)]);
        at(&app)
    };
    let heights = [Height::Top, Height::Middle, Height::Bottom];
    // Offsets of 8 and 12 pixels clip the first row by less and by more than
    // half, and show the row below the view by less and by more than half
    assert_eq!(heights.map(|height| row(0.4, height)), [1, 3, 5]);
    assert_eq!(heights.map(|height| row(0.6, height)), [2, 4, 6]);
}
