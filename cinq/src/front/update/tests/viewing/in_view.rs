// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of selecting the rows in view and of reaching a row in view

use super::{
    super::{paging::shaped, show},
    pressed,
};
use crate::{
    back::CueId,
    front::{
        App, Message,
        message::{Height, Spot},
        update::rows::{paging::viewport::Shape, viewing::in_view},
    },
};

/// Returns the row that `height` selects in the list of [`shaped`] scrolled
/// by `scrolled` rows
fn in_view_at(height: Height, count: Option<usize>, scrolled: f32) -> usize {
    in_view(height, count, shaped(scrolled), 60)
        .expect("rows to select")
        .row
}

#[test]
fn top_middle_and_bottom_select_the_rows_in_view_without_scrolling() {
    // Rows 10 to 29 are in view, and the middle is the upper of the two
    assert_eq!(in_view_at(Height::Top, None, 10.0), 10);
    assert_eq!(in_view_at(Height::Middle, None, 10.0), 19);
    assert_eq!(in_view_at(Height::Bottom, None, 10.0), 29);
    let landing = in_view(Height::Top, None, shaped(10.0), 60).expect("rows to select");
    assert_eq!(landing.offset, 200.0);
}

#[test]
fn a_count_counts_the_rows_from_the_top_or_the_bottom_and_stops_in_view() {
    assert_eq!(in_view_at(Height::Top, Some(3), 10.0), 12);
    assert_eq!(in_view_at(Height::Bottom, Some(3), 10.0), 27);
    assert_eq!(in_view_at(Height::Top, Some(99), 10.0), 29);
    assert_eq!(in_view_at(Height::Bottom, Some(99), 10.0), 10);
    // Count has no meaning for the middle
    assert_eq!(in_view_at(Height::Middle, Some(3), 10.0), 19);
}

#[test]
fn a_list_shorter_than_its_view_has_its_middle_among_its_rows() {
    let shape = Shape {
        content: 140.0,
        ..shaped(0.0)
    };
    let row = |height| in_view(height, None, shape, 7).map(|landing| landing.row);
    assert_eq!(
        [Height::Top, Height::Middle, Height::Bottom].map(row),
        [Some(0), Some(3), Some(6)]
    );
    assert!(in_view(Height::Top, None, shaped(0.0), 0).is_none());
}

fn reaching(app: App, from: Option<CueId>, row: usize) -> App {
    let from = from.map_or(Spot::List(0), Spot::Cue);
    app.next(Message::Reached { from, row }).0
}

#[test]
fn reaching_a_row_in_view_is_a_jump_that_back_returns_from() {
    let reached = reaching(show(Some(CueId(2))), Some(CueId(2)), 8);
    assert_eq!(reached.selected, Some(CueId(9)));
    let back = reached.handled(Message::Back(1)).0;
    assert_eq!(back.selected, Some(CueId(2)));
    // Staying where it was leaves no place to go back to
    let stayed = reaching(show(Some(CueId(2))), Some(CueId(2)), 1);
    assert!(!stayed.can_go_back());
}

#[test]
fn reaching_is_dropped_when_a_key_moved_the_selection_meanwhile() {
    let moved = reaching(show(Some(CueId(5))), Some(CueId(2)), 8);
    assert_eq!(moved.selected, Some(CueId(5)));
    assert!(!moved.can_go_back());
}

#[test]
fn keys_of_top_middle_and_bottom_measure_the_cue_list_before_the_selection_moves() {
    let typed = |keys| {
        let (app, started) = pressed(keys);
        (app.selected, started)
    };
    ["H", "M", "L"]
        .into_iter()
        .for_each(|key| assert_eq!(typed(key), (Some(CueId(2)), 2), "{key}"));
    assert_eq!(typed("3 H"), (Some(CueId(2)), 2));
}
