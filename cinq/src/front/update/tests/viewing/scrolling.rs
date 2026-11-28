// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of scrolling a row at a time, in the cue list and on the help page

use super::super::{
    helping,
    paging::{shaped, typed},
    show,
};
use crate::{back::CueId, front::update::rows::viewing::scroll};

/// Returns the offset in rows and the row that scrolling the list of
/// [`shaped`] by `rows` rows leaves, with the selection on `current`
fn scrolled(rows: isize, scrolled: f32, current: usize) -> (f32, usize) {
    let landing = scroll(rows, shaped(scrolled), 60, Some(current));
    let landing = landing.expect("rows to scroll");
    (landing.offset / 20.0, landing.row)
}

#[test]
fn scrolling_a_row_keeps_the_selection_on_its_cue_while_it_stays_in_view() {
    assert_eq!(scrolled(1, 10.0, 15), (11.0, 15));
    assert_eq!(scrolled(-1, 10.0, 15), (9.0, 15));
    assert_eq!(scrolled(5, 10.0, 25), (15.0, 25));
}

#[test]
fn scrolling_a_row_moves_a_selection_that_leaves_the_view_to_the_nearest_row() {
    // Rows 10 to 29 are in view, then 11 to 30, and 9 to 28
    assert_eq!(scrolled(1, 10.0, 10), (11.0, 11));
    assert_eq!(scrolled(-1, 10.0, 29), (9.0, 28));
    assert_eq!(scrolled(5, 10.0, 12), (15.0, 15));
    assert_eq!(scrolled(-5, 10.0, 28), (5.0, 24));
}

#[test]
fn scrolling_a_row_stops_at_either_end_of_the_list() {
    assert_eq!(scrolled(-3, 1.0, 5), (0.0, 5));
    assert_eq!(scrolled(100, 35.0, 36), (40.0, 40));
    assert_eq!(scrolled(100, 40.0, 59), (40.0, 59));
    assert_eq!(scrolled(isize::MAX, 0.0, 0), (40.0, 40));
    assert_eq!(scrolled(isize::MIN, 40.0, 59), (0.0, 19));
    assert!(scroll(1, shaped(0.0), 0, None).is_none());
}

#[test]
fn keys_of_a_row_scroll_measure_the_cue_list_and_move_no_selection_themselves() {
    let (after, started) = typed(show(Some(CueId(2))), "e", &[]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 2));
    let (after, started) = typed(after, "y", &["3"]);
    assert_eq!(
        (after.selected, after.scroll, started),
        (Some(CueId(2)), None, 2)
    );
}

#[test]
fn keys_of_a_row_scroll_scroll_the_help_page_a_line_and_move_no_selection() {
    let open = helping(show(Some(CueId(2))));
    let (after, started) = typed(open, "e", &[]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 1));
    let (after, started) = typed(after, "y", &["3"]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 1));
}
