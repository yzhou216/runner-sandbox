// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of moving by half a page or a whole page, in the cue list and on
//! the help page

mod kept;
mod keys;

use iced::keyboard::{Key, Modifiers};

use super::chord;
use crate::front::{
    App,
    message::Page,
    update::paging::{landing, rows, rows_fitting, viewport::Shape},
};

pub(super) fn letter(key: &str) -> Key {
    Key::Character(key.into())
}

/// Types a key with Control held in a state of the interface, which the
/// tests give twelve cues, and returns it with the number of tasks that it
/// started
pub(super) fn typed(app: App, key: &str, count: &[&str]) -> (App, usize) {
    let app = count.iter().fold(app, |app, digit| {
        app.typed(super::press(letter(digit)), false).0
    });
    let (app, task) = app.typed(chord(letter(key), Modifiers::CTRL), false);
    (app, task.units())
}

#[test]
fn rows_of_a_page_follow_the_rows_that_fit_and_the_counts() {
    // Half a page is half the rows that fit, and at least one
    assert_eq!(rows(Page::HalfDown, None, None, 9), 4);
    assert_eq!(rows(Page::HalfUp, None, None, 1), 1);
    assert_eq!(rows(Page::HalfDown, None, None, 0), 1);
    // Count sets the rows of a half page, which a later press keeps
    assert_eq!(rows(Page::HalfDown, Some(3), Some(3), 9), 3);
    assert_eq!(rows(Page::HalfUp, None, Some(3), 9), 3);
    // Whole page is the rows that fit less two, and a count counts pages
    assert_eq!(rows(Page::WholeDown, None, Some(3), 9), 7);
    assert_eq!(rows(Page::WholeUp, Some(2), None, 9), 14);
    assert_eq!(rows(Page::WholeDown, None, None, 0), 1);
    assert_eq!(rows(Page::WholeDown, None, None, 2), 1);
    assert_eq!(rows(Page::WholeDown, Some(usize::MAX), None, 9), usize::MAX);
}

#[test]
fn rows_that_fit_come_from_the_average_height_of_a_row() {
    assert_eq!(rows_fitting(500.0, 1200.0, 30), 12);
    // Two rows that wrap make the others count for less
    assert_eq!(rows_fitting(100.0, 500.0, 10), 2);
    assert_eq!(rows_fitting(100.0, 0.0, 0), 0);
}

/// Returns the shape of a list of 60 rows of 20 pixels in a view of 400,
/// which shows 20 rows, scrolled by `rows` rows
pub(super) fn shaped(rows: f32) -> Shape {
    Shape {
        view: 400.0,
        content: 1200.0,
        scrolled: rows * 20.0,
    }
}

/// Returns the offset in rows and the row that a page lands on, in the list
/// of [`shaped`]
fn landed(page: Page, count: Option<usize>, scrolled: f32, current: usize) -> (f32, usize) {
    let landing = landing(page, count, None, shaped(scrolled), 60, Some(current));
    let landing = landing.expect("rows to land on");
    (landing.offset / 20.0, landing.row)
}

#[test]
fn half_a_page_scrolls_by_the_rows_that_the_selection_moves() {
    // Selection keeps its row on the screen, here the first
    assert_eq!(landed(Page::HalfDown, None, 0.0, 0), (10.0, 10));
    assert_eq!(landed(Page::HalfDown, None, 10.0, 10), (20.0, 20));
    assert_eq!(landed(Page::HalfUp, None, 20.0, 25), (10.0, 15));
    // At the end the view stops and the selection goes on to the last cue
    assert_eq!(landed(Page::HalfDown, None, 35.0, 45), (40.0, 55));
    assert_eq!(landed(Page::HalfDown, None, 40.0, 55), (40.0, 59));
    let huge = landing(
        Page::HalfDown,
        None,
        Some(usize::MAX),
        shaped(0.0),
        60,
        Some(3),
    );
    assert_eq!(
        huge.map(|landing| (landing.offset, landing.row)),
        Some((800.0, 59))
    );
    // At the top likewise
    assert_eq!(landed(Page::HalfUp, None, 5.0, 12), (0.0, 2));
    assert_eq!(landed(Page::HalfUp, None, 0.0, 2), (0.0, 0));
}

#[test]
fn a_page_scrolls_by_a_page_less_two_rows_and_selects_the_row_in_view() {
    // Eighteen rows, with the first in view as the first row down
    assert_eq!(landed(Page::WholeDown, None, 0.0, 3), (18.0, 18));
    assert_eq!(landed(Page::WholeDown, Some(2), 0.0, 3), (36.0, 36));
    // Selection never goes back for a page down
    assert_eq!(landed(Page::WholeDown, None, 0.0, 30), (18.0, 30));
    // Last row in view going up
    assert_eq!(landed(Page::WholeUp, None, 30.0, 49), (12.0, 31));
    // And never forward of the selection
    assert_eq!(landed(Page::WholeUp, None, 30.0, 20), (12.0, 20));
    // View stops at either end, and the selection goes to the last or first cue
    assert_eq!(landed(Page::WholeDown, None, 30.0, 40), (40.0, 59));
    assert_eq!(landed(Page::WholeUp, None, 10.0, 15), (0.0, 0));
    assert!(landing(Page::WholeDown, None, None, shaped(0.0), 0, None).is_none());
}
