// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of moving by half a page or a whole page, in the cue list and on
//! the help page

use iced::keyboard::{Key, Modifiers, key::Named};

use super::{chord, show};
use crate::{
    back::CueId,
    front::{
        App, Message,
        message::Page,
        update::paging::{rows, rows_fitting},
    },
};

fn letter(key: &str) -> Key {
    Key::Character(key.into())
}

/// Types a key with Control held in a state of the interface, which the
/// tests give twelve cues, and returns it with the number of tasks that it
/// started
fn typed(app: App, key: &str, count: &[&str]) -> (App, usize) {
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
    // A count sets the rows of a half page, which a later press keeps
    assert_eq!(rows(Page::HalfDown, Some(3), Some(3), 9), 3);
    assert_eq!(rows(Page::HalfUp, None, Some(3), 9), 3);
    // A whole page is the rows that fit, and a count counts pages
    assert_eq!(rows(Page::WholeDown, None, Some(3), 9), 9);
    assert_eq!(rows(Page::WholeUp, Some(2), None, 9), 18);
    assert_eq!(rows(Page::WholeDown, None, None, 0), 1);
    assert_eq!(rows(Page::WholeDown, Some(usize::MAX), None, 9), usize::MAX);
}

#[test]
fn rows_that_fit_come_from_the_average_height_of_a_row() {
    assert_eq!(rows_fitting(500.0, 1200.0, 30), 12);
    // Two rows that wrap make the others count for less
    assert_eq!(rows_fitting(100.0, 500.0, 10), 2);
    assert_eq!(rows_fitting(100.0, 0.0, 0), 0);
}

#[test]
fn keys_of_a_page_measure_the_cue_list_before_the_selection_moves() {
    let app = show(Some(CueId(2)));
    let (after, started) = typed(app, "d", &[]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 1));
    let (after, started) = typed(after, "f", &[]);
    assert_eq!((after.selected, started), (Some(CueId(2)), 1));
}

#[test]
fn count_before_half_a_page_sets_how_many_rows_it_moves_from_then_on() {
    let (counted, _) = typed(show(Some(CueId(2))), "d", &["5"]);
    assert_eq!(counted.scroll, Some(5));
    let (kept, _) = typed(counted, "u", &[]);
    assert_eq!(kept.scroll, Some(5));
    // A count before a whole page counts pages, so it sets nothing
    let (pages, _) = typed(show(Some(CueId(2))), "f", &["5"]);
    assert_eq!(pages.scroll, None);
}

#[test]
fn step_of_a_page_moves_the_selection_and_stops_at_either_end() {
    let step = |selected, rows: isize| {
        show(Some(CueId(selected)))
            .handled(Message::Step(rows))
            .0
            .selected
    };
    assert_eq!(step(2, 5), Some(CueId(7)));
    assert_eq!(step(2, -5), Some(CueId(1)));
    assert_eq!(step(10, isize::MAX), Some(CueId(12)));
}

#[test]
fn keys_of_a_page_scroll_the_help_page_and_move_no_selection() {
    let open = || App {
        help: true,
        ..show(Some(CueId(2)))
    };
    ["d", "u", "f", "b"].into_iter().for_each(|key| {
        let (after, started) = typed(open(), key, &[]);
        assert_eq!(
            (after.selected, after.scroll, started),
            (Some(CueId(2)), None, 1)
        );
    });
    // A count sets nothing, since it counts lines for that press alone
    let (after, started) = typed(open(), "d", &["4"]);
    assert_eq!((after.scroll, started), (None, 1));
}

#[test]
fn named_keys_of_a_page_act_in_the_cue_list_and_on_the_help_page() {
    let page = |help: bool, named: Named| {
        let app = App {
            help,
            ..show(Some(CueId(2)))
        };
        let (_, task) = app.typed(super::press(Key::Named(named)), false);
        task.units()
    };
    assert_eq!(
        [Named::PageDown, Named::PageUp].map(|named| page(false, named)),
        [1, 1]
    );
    assert_eq!(
        [Named::PageDown, Named::PageUp].map(|named| page(true, named)),
        [1, 1]
    );
}
