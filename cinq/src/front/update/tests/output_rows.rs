// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that move through rows acting on the outputs while they
//! show, by the code that the cue list uses, and never on the cue list out of
//! sight

use super::{
    history::{after, cue, place},
    paging::typed,
    press, show,
};
use crate::{
    back::{CueId, OutputId, Place},
    front::{
        App, Message, Setting,
        message::Height,
        update::{Measured, Rows, Spot, paging::viewport::Shape},
    },
};

/// Returns the show with ten outputs showing, the first selected, over a cue
/// list that has the third cue selected.  The outputs are named for the
/// searches, from House L to Booth.
pub(super) fn outputs() -> App {
    let names = [
        "House L",
        "House R",
        "Sub",
        "Monitor",
        "Front fill",
        "Balcony",
        "Lobby",
        "Stage",
        "FX",
        "Booth",
    ];
    let added = after(
        show(cue(3)),
        std::iter::once(Message::ShowOutputs)
            .chain((1..names.len()).map(|_| Message::Add(Place::After, 1))),
    );
    let named = names.iter().enumerate().fold(added, |app, (row, name)| {
        let id = app.workspace.outputs()[row].id;
        after(
            app,
            [Message::SettingTyped(
                Setting::OutputName(id),
                (*name).to_owned(),
            )],
        )
    });
    after(named, [Message::Row(1)])
}

/// Returns the row of the selected output, counted from one
pub(super) fn at(app: &App) -> usize {
    app.output_row().map_or(0, |row| row + 1)
}

/// Asserts that the cue list out of sight is where it was
fn cue_list_kept(app: &App) {
    assert_eq!(place(app), (0, cue(3)));
    assert!(app.outputs_shown);
}

#[test]
fn the_keys_of_the_next_and_previous_row_with_counts_step_through_the_outputs() {
    let pressed = |keys: &[&str]| {
        let app = keys.iter().fold(outputs(), |app, key| {
            app.typed(press(super::paging::letter(key)), false).0
        });
        (at(&app), app)
    };
    assert_eq!(pressed(&["j"]).0, 2);
    assert_eq!(pressed(&["j", "j", "k"]).0, 2);
    assert_eq!(pressed(&["3", "j"]).0, 4);
    // Stops at the ends, as in the cue list
    assert_eq!(pressed(&["k"]).0, 1);
    assert_eq!(pressed(&["9", "9", "j"]).0, 10);
    cue_list_kept(&pressed(&["5", "j"]).1);
}

#[test]
fn first_and_last_with_counts_select_an_output_as_a_jump_that_back_returns_from() {
    let row = |count: usize| at(&after(outputs(), [Message::Row(count)]));
    assert_eq!((row(1), row(4), row(usize::MAX)), (1, 4, 10));
    let last = after(outputs(), [Message::Row(usize::MAX)]);
    cue_list_kept(&last);
    let back = after(last, [Message::Back(1)]);
    assert!(back.outputs_shown);
    assert_eq!(at(&back), 1);
}

/// Returns the outputs measured as ten rows of twenty pixels in a view of
/// five, scrolled by `rows` rows
fn measured(rows: f32) -> App {
    let app = outputs();
    let shape = Shape {
        view: 100.0,
        content: 200.0,
        scrolled: rows * 20.0,
    };
    let serial = app.measures.serial();
    after(
        app,
        [Message::Measured(Measured::Rows {
            serial,
            rows: Rows::Outputs,
            count: 10,
            shape,
        })],
    )
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
    let app = outputs();
    let shape = Shape {
        view: 100.0,
        content: 200.0,
        scrolled: 0.0,
    };
    let serial = app.measures.serial();
    let cues = after(
        app,
        [Message::Measured(Measured::Rows {
            serial,
            rows: Rows::Cues(0),
            count: 10,
            shape,
        })],
    );
    // The key waits for a measure of the outputs, and moves nothing yet
    let (waiting, started) = typed(cues, "d", &[]);
    assert_eq!((at(&waiting), started > 0), (1, true));
}

#[test]
fn scrolling_and_the_rows_in_view_select_outputs_without_touching_the_cue_list() {
    let scrolled = after(measured(0.0), [Message::Scroll(2)]);
    // The selection goes to the first row that is still in view
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
fn a_landing_from_another_place_is_dropped() {
    let from = Spot::Output(OutputId(99));
    let app = after(outputs(), [Message::Landed { from, row: 5 }]);
    assert_eq!(at(&app), 1);
    let from = Spot::Cue(CueId(3));
    let app = after(outputs(), [Message::Reached { from, row: 5 }]);
    assert_eq!(at(&app), 1);
}

#[test]
fn page_keys_found_by_the_spot_of_the_outputs_land_there() {
    let app = outputs();
    let from = app.spot();
    assert_eq!(from, Spot::Output(OutputId(1)));
    let landed = after(app, [Message::Landed { from, row: 5 }]);
    assert_eq!(at(&landed), 6);
    let reached = after(
        landed,
        [Message::Reached {
            from: Spot::Output(OutputId(6)),
            row: 8,
        }],
    );
    assert_eq!(at(&reached), 9);
    // Reaching is a jump
    assert_eq!(at(&after(reached, [Message::Back(1)])), 6);
}

/// Returns the row of the selected output and the state, after the messages
fn found(messages: impl IntoIterator<Item = Message>) -> (usize, App) {
    let app = after(outputs(), messages);
    (at(&app), app)
}

#[test]
fn a_search_selects_the_outputs_by_name_as_it_selects_cues() {
    let search = |text: &str| {
        found([
            Message::Search { backward: false },
            Message::Searching(text.to_owned()),
        ])
    };
    let (row, app) = search("sub");
    assert_eq!(row, 3);
    assert!(
        app.search.is_some(),
        "the line stays open while it is typed"
    );
    // The first output after the selection that matches, in any case
    assert_eq!(search("HOUSE").0, 2);
    assert_eq!(search("o").0, 2);
    // No output matches, so that the selection stays where the search began
    let (row, missed) = search("zzz");
    assert_eq!(row, 1);
    assert!(missed.search.as_ref().is_some_and(|search| search.warned));
    cue_list_kept(&missed);
}

#[test]
fn accepting_a_search_in_the_outputs_is_a_jump_and_next_and_previous_match_repeat_it() {
    let accepted = after(
        outputs(),
        [
            Message::Search { backward: false },
            Message::Searching("o".to_owned()),
            Message::SearchAccepted,
        ],
    );
    assert!(accepted.search.is_none());
    assert_eq!(at(&accepted), 2);
    // Names holding an o, after House R: Monitor, Front fill is no match
    let next = after(accepted, [Message::SearchStep(1)]);
    assert_eq!(at(&next), 4);
    let previous = after(next, [Message::SearchStep(-1)]);
    assert_eq!(at(&previous), 2);
    cue_list_kept(&previous);
    // The accepted search and the steps are jumps for going back
    let back = after(previous, [Message::Back(1)]);
    assert!(back.outputs_shown);
    assert_eq!(at(&back), 4);
}

#[test]
fn a_search_that_wraps_in_the_outputs_says_so_and_one_without_a_match_says_that() {
    let last = after(outputs(), [Message::Row(usize::MAX)]);
    let wrapped = after(
        last,
        [
            Message::Search { backward: false },
            Message::Searching("house".to_owned()),
            Message::SearchAccepted,
        ],
    );
    assert_eq!(at(&wrapped), 1);
    assert_eq!(wrapped.remark, Some("Search continued at the top"));
    let nothing = after(outputs(), [Message::SearchStep(1)]);
    assert_eq!(nothing.remark, Some("No output matches"));
    assert_eq!(at(&nothing), 1);
}

#[test]
fn cancelling_a_search_in_the_outputs_puts_the_selection_back() {
    let app = after(
        outputs(),
        [
            Message::Search { backward: false },
            Message::Searching("booth".to_owned()),
            Message::Panic,
        ],
    );
    assert!(app.search.is_none());
    assert_eq!(at(&app), 1);
}

#[test]
fn showing_the_cue_list_again_closes_the_search_line_of_the_outputs() {
    let app = after(
        outputs(),
        [
            Message::Search { backward: false },
            Message::Searching("sub".to_owned()),
            Message::SwitchList(0),
        ],
    );
    assert!(app.search.is_none());
    assert!(!app.outputs_shown);
}
