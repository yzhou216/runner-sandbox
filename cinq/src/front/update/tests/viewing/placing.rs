// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of putting the selected row at the top, middle, or bottom of the view

use super::{
    super::{paging::shaped, show},
    pressed,
};
use crate::{
    back::CueId,
    front::{
        Message,
        message::Height,
        update::rows::{paging::viewport::Shape, viewing::place},
    },
};

/// Returns the offset in rows that puts `current` at `height` in the list
/// of [`shaped`]
fn placed(height: Height, current: usize) -> f32 {
    let landing = place(height, shaped(0.0), 60, current).expect("a row to place");
    assert_eq!(landing.row, current);
    landing.offset / 20.0
}

#[test]
fn z_puts_the_selected_row_at_the_top_middle_or_bottom_of_the_view() {
    // Twenty rows in view, so that row 30 sits at 30, 20, and 11 rows down
    assert_eq!(placed(Height::Top, 30), 30.0);
    assert_eq!(placed(Height::Middle, 30), 20.5);
    assert_eq!(placed(Height::Bottom, 30), 11.0);
}

#[test]
fn z_stops_at_either_end_of_the_list() {
    assert_eq!(placed(Height::Middle, 3), 0.0);
    assert_eq!(placed(Height::Bottom, 3), 0.0);
    assert_eq!(placed(Height::Top, 55), 40.0);
    assert_eq!(placed(Height::Middle, 58), 40.0);
    assert!(place(Height::Top, shaped(0.0), 0, 0).is_none());
    // List that fits its view does not scroll
    let fits = Shape {
        content: 100.0,
        ..shaped(0.0)
    };
    assert_eq!(place(Height::Top, fits, 5, 3).map(|l| l.offset), Some(0.0));
}

#[test]
fn keys_of_z_measure_the_cue_list_and_leave_the_selection_alone() {
    // First key starts nothing, as it waits for the second
    assert_eq!(pressed("z").1, 0);
    ["z z", "z t", "z b"].into_iter().for_each(|keys| {
        let (app, started) = pressed(keys);
        assert_eq!(
            (app.selected, app.keys.len(), started),
            (Some(CueId(2)), 0, 2),
            "{keys}"
        );
    });
    // Without a selected cue there is no row to place
    let (_, task) = show(None).handled(Message::Place(Height::Top, None));
    assert_eq!(task.units(), 2);
}

#[test]
fn a_count_before_the_keys_of_z_selects_that_row_first_and_is_no_jump() {
    let placed = |count| {
        show(Some(CueId(2)))
            .handled(Message::Place(Height::Top, count))
            .0
    };
    assert_eq!(placed(None).selected, Some(CueId(2)));
    assert_eq!(placed(Some(5)).selected, Some(CueId(5)));
    // Row past the end is the last, as for the keys of the first and last
    assert_eq!(placed(Some(99)).selected, Some(CueId(12)));
    assert!(!placed(Some(5)).can_go_back());
    // Key reaches the message with the count
    let (typed, started) = pressed("5 z z");
    assert_eq!((typed.selected, started), (Some(CueId(5)), 2));
}
