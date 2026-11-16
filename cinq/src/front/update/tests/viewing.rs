// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of scrolling the cue list a row at a time, of selecting the rows in
//! its view, and of putting the selected row in view, in the cue list and on
//! the help page

use iced::Task;

use super::{
    paging::{letter, shaped, typed},
    show,
};
use crate::{
    back::CueId,
    front::{
        App, Message,
        message::Height,
        update::{
            Spot,
            paging::viewport::Shape,
            viewing::{in_view, place, scroll},
        },
    },
};

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
    let open = App {
        help: true,
        ..show(Some(CueId(2)))
    };
    let (after, started) = typed(open, "e", &[]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 1));
    let (after, started) = typed(after, "y", &["3"]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 1));
}

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
    let typed = |key: &str, count: &[&str]| {
        let app = count.iter().fold(show(Some(CueId(2))), |app, digit| {
            app.typed(super::press(letter(digit)), false).0
        });
        let (after, task) = app.typed(super::press(letter(key)), false);
        (after.selected, task.units())
    };
    ["H", "M", "L"]
        .into_iter()
        .for_each(|key| assert_eq!(typed(key, &[]), (Some(CueId(2)), 2), "{key}"));
    assert_eq!(typed("H", &["3"]), (Some(CueId(2)), 2));
}

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
    let keys = |second: &str| {
        let app = show(Some(CueId(2)));
        let (app, first) = app.typed(super::press(letter("z")), false);
        let (app, task) = app.typed(super::press(letter(second)), false);
        (app.selected, app.keys.len(), first.units(), task.units())
    };
    ["z", "t", "b"]
        .into_iter()
        .for_each(|key| assert_eq!(keys(key), (Some(CueId(2)), 0, 0, 2), "z {key}"));
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
    let (typed, task) = ["5", "z", "z"]
        .iter()
        .fold((show(Some(CueId(2))), Task::none()), |(app, _), key| {
            app.typed(super::press(letter(key)), false)
        });
    assert_eq!((typed.selected, task.units()), (Some(CueId(5)), 2));
}

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
fn control_e_moves_the_selection_one_row_at_a_time_whatever_the_rounding_of_the_offset() {
    [0.0, 0.2, 0.4, -0.3]
        .into_iter()
        .for_each(|drift| assert_eq!(through(1, drift, 0.0, 0), [1, 2, 3, 4], "drift {drift}"));
    // Row that stays in view keeps the selection
    assert_eq!(through(1, 0.0, 0.0, 9), [9, 9, 9, 9]);
}

#[test]
fn control_y_moves_the_selection_one_row_at_a_time_whatever_the_rounding_of_the_offset() {
    let bottom = 40.0 * 31.8;
    [0.0, 0.2, 0.4, -0.3].into_iter().for_each(|drift| {
        assert_eq!(
            through(-1, drift, bottom, 52),
            [51, 50, 49, 48],
            "drift {drift}"
        );
    });
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
