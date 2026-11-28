// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the row that the selection takes however the offset rounds

use crate::front::{
    message::Height,
    update::rows::{
        paging::viewport::Shape,
        viewing::{in_view, scroll},
    },
};

/// Returns the shape of the list of 60 rows that the runners laid out, 31.8
/// pixels tall each in a view of 400, scrolled by `scrolled` pixels
fn runner(scrolled: f32) -> Shape {
    Shape {
        view: 400.0,
        content: 60.0 * 31.8,
        scrolled,
    }
}

/// Returns the rows that four scrolls by `rows` rows take the selection
/// through, starting at offset `start` on row `current`, the list reporting
/// each offset in whole pixels after `drift` pixels are added to it
fn through(rows: isize, drift: f32, start: f32, current: usize) -> Vec<usize> {
    (0..4)
        .scan((start, current), |(scrolled, current), _| {
            let landing = scroll(rows, runner(*scrolled), 60, Some(*current))?;
            *scrolled = (landing.offset + drift).round();
            *current = landing.row;
            Some(landing.row)
        })
        .collect()
}

#[test]
fn a_row_with_most_of_it_showing_is_in_view_whatever_the_rounding_of_the_offset() {
    let rows = |scrolled| runner(scrolled).rows_in_view(60);
    // Row 1 is clipped by 0.2 pixels at the top and row 13 shows 14.4 of 31.8
    assert_eq!(rows(32.0), Some((1, 13)));
    assert_eq!(rows(64.0), Some((2, 14)));
    assert_eq!(rows(0.0), Some((0, 12)));
    // Less than half of the first row shows, and more than half of the last
    assert_eq!(rows(20.0), Some((1, 12)));
    assert_eq!(rows(10.0), Some((0, 12)));
    assert_eq!(runner(0.0).rows_in_view(0), None);
}

#[test]
fn control_e_and_y_move_the_selection_one_row_at_a_time_whatever_the_rounding() {
    let bottom = 40.0 * 31.8;
    [0.0, 0.2, 0.4, -0.3].into_iter().for_each(|drift| {
        assert_eq!(through(1, drift, 0.0, 0), [1, 2, 3, 4], "E, drift {drift}");
        assert_eq!(
            through(-1, drift, bottom, 52),
            [51, 50, 49, 48],
            "Y, drift {drift}"
        );
    });
    // Row that stays in view keeps the selection
    assert_eq!(through(1, 0.0, 0.0, 9), [9, 9, 9, 9]);
    assert_eq!(through(-1, 0.0, bottom, 45), [45, 45, 45, 45]);
}

#[test]
fn h_and_l_select_the_rows_that_show_most_of_themselves_at_the_runner_offsets() {
    let row =
        |height, scrolled| in_view(height, None, runner(scrolled), 60).map(|landing| landing.row);
    assert_eq!(row(Height::Top, 32.0), Some(1));
    assert_eq!(row(Height::Top, 64.0), Some(2));
    assert_eq!(row(Height::Top, 95.8), Some(3));
    assert_eq!(row(Height::Bottom, 32.0), Some(13));
    assert_eq!(row(Height::Bottom, 64.0), Some(14));
    assert_eq!(row(Height::Bottom, 95.8), Some(15));
}
