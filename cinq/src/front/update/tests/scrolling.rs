// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of where the cue list scrolls to show the selected row

use crate::front::update::rows::scroll::{Span, scrolling_to};

#[test]
fn a_row_far_outside_the_view_scrolls_to_the_middle_and_one_near_it_by_the_least() {
    // View of 23 rows of 20 pixels from row 50, the last in view being row
    // 72, with the three rows of margin that Vim keeps by default
    let scrolling = |row: f32| {
        let span = Span {
            top: row * 20.0,
            height: 20.0,
            margin: 60.0,
        };
        scrolling_to(span, 1000.0, 460.0)
    };
    // In view, clear of the margin, nothing to do
    assert_eq!(scrolling(60.0), None);
    // One row below the view, as GO moves the standby by a row, the least scroll
    assert_eq!(scrolling(73.0), Some(1080.0));
    // Vim measured in a window of 23 rows with scrolloff 3: a jump 12 rows
    // below the view scrolls by the least, and 13 rows below centers
    assert_eq!(scrolling(84.0), Some(1300.0));
    assert_eq!(scrolling(85.0), Some(1480.0));
    // Above the view 6 rows scroll by the least, and 7 rows center
    assert_eq!(scrolling(44.0), Some(820.0));
    assert_eq!(scrolling(43.0), Some(640.0));
    // Row in view but within the margin scrolls by the least too
    assert_eq!(scrolling(52.0), Some(980.0));
}
