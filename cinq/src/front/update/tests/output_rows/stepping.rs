// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of stepping through the outputs, and of the landings and jumps there

use super::{
    super::{after, typing},
    at, cue_list_kept, outputs,
};
use crate::{
    back::{CueId, OutputId},
    front::{Message, Spot},
};

#[test]
fn the_keys_of_the_next_and_previous_row_with_counts_step_through_the_outputs() {
    let pressed = |keys| {
        let app = typing(outputs(), keys);
        (at(&app), app)
    };
    assert_eq!(pressed("j").0, 2);
    assert_eq!(pressed("j j k").0, 2);
    assert_eq!(pressed("3 j").0, 4);
    // Stops at the ends, as in the cue list
    assert_eq!(pressed("k").0, 1);
    assert_eq!(pressed("9 9 j").0, 10);
    cue_list_kept(&pressed("5 j").1);
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
