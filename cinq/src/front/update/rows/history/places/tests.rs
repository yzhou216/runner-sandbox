// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the history of places, which know nothing of the show

use super::*;
use crate::back::{CueId, OutputId};

fn cue(id: u32) -> Spot {
    Spot::Cue(CueId(id))
}

fn everywhere(_: Spot) -> bool {
    true
}

/// Returns the history after a jump that leaves each of `places` in turn
fn jumped_from(places: impl IntoIterator<Item = Spot>) -> History {
    places.into_iter().fold(History::default(), History::jumped)
}

/// Goes `count` places back from `here`, where there is a place behind
fn back(history: History, here: Spot, count: usize) -> (Spot, History) {
    history
        .back(here, count, everywhere)
        .expect("a place behind")
}

/// Goes `count` places forward from `here`, where there is a place ahead
fn forward(history: History, here: Spot, count: usize) -> (Spot, History) {
    history
        .forward(here, count, everywhere)
        .expect("a place ahead")
}

#[test]
fn back_and_forward_retrace_the_places() {
    let history = jumped_from([cue(1), cue(2), cue(3)]);
    let (spot, history) = back(history, cue(4), 1);
    assert_eq!(spot, cue(3));
    let (spot, history) = back(history, spot, 2);
    assert_eq!(spot, cue(1));
    let (spot, history) = forward(history, spot, 1);
    assert_eq!(spot, cue(2));
    let (spot, history) = forward(history, spot, 5);
    assert_eq!(spot, cue(4));
    assert_eq!(history.forward(spot, 1, everywhere), None);
    assert!(history.can_go_back(spot, everywhere));
    assert!(!history.can_go_forward(spot, everywhere));
}

#[test]
fn a_count_past_the_oldest_place_goes_to_the_oldest() {
    let (spot, _) = back(jumped_from([cue(1), cue(2), cue(3)]), cue(4), 50);
    assert_eq!(spot, cue(1));
}

#[test]
fn a_place_is_remembered_once() {
    let history = jumped_from([cue(1), cue(2), cue(1), cue(2)]);
    assert_eq!(history.behind, [cue(1), cue(2)]);
}

#[test]
fn a_jump_after_going_back_keeps_the_places_ahead_behind() {
    let (spot, history) = back(jumped_from([cue(1), cue(2)]), cue(3), 1);
    assert!(history.can_go_forward(spot, everywhere));
    let history = history.jumped(spot);
    assert_eq!(history.ahead, []);
    assert_eq!(history.behind, [cue(1), cue(3), cue(2)]);
}

#[test]
fn going_back_after_a_jump_visits_what_vim_visits() {
    // 10G 20G Control+o 30G Control+o Control+o, from the first line
    let (spot, history) = back(jumped_from([cue(1), cue(10)]), cue(20), 1);
    assert_eq!(spot, cue(10));
    let (spot, history) = back(history.jumped(spot), cue(30), 1);
    assert_eq!(spot, cue(10));
    assert_eq!(back(history, spot, 1).0, cue(20));
}

#[test]
fn places_that_are_gone_or_here_are_skipped_and_dropped() {
    let history = jumped_from([cue(1), cue(2), cue(3)]);
    let alive = |spot| spot != cue(2);
    let (spot, history) = history.back(cue(3), 2, alive).expect("a place behind");
    assert_eq!(spot, cue(1));
    assert_eq!((history.behind, history.ahead), (Vec::new(), vec![cue(3)]));
    assert_eq!(History::default().back(cue(1), 1, everywhere), None);
}

#[test]
fn the_history_keeps_the_last_hundred_places() {
    let history = jumped_from((1..=150).map(cue));
    assert_eq!(history.behind.len(), 100);
    assert_eq!(history.behind.first(), Some(&cue(51)));
    // Going back remembers the place left, which still counts against the limit
    let (_, history) = back(history, cue(151), 1);
    assert_eq!(history.behind.len() + history.ahead.len(), 100);
}

#[test]
fn a_way_back_that_no_jump_recorded_is_remembered_to_go_forward_to() {
    let history = History::default().left(Spot::Output(OutputId(1)));
    assert!(!history.can_go_back(cue(1), everywhere));
    assert_eq!(forward(history, cue(1), 1).0, Spot::Output(OutputId(1)));
}

#[test]
fn an_output_is_a_place_apart_from_a_cue_and_a_list() {
    let history = jumped_from([Spot::Output(OutputId(1)), Spot::Outputs, Spot::List(0)]);
    assert_eq!(history.behind.len(), 3);
}
