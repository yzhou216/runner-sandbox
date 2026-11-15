// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the history of places, which know nothing of the show

use super::*;
use crate::back::OutputId;

fn cue(id: u32) -> Spot {
    Spot::Cue(CueId(id))
}

fn everywhere(_: Spot) -> bool {
    true
}

/// Returns the history that jumps from each of `places` in turn leave
fn jumped_from(places: impl IntoIterator<Item = Spot>) -> History {
    places.into_iter().fold(History::default(), History::jumped)
}

#[test]
fn back_and_forward_retrace_the_places() {
    let history = jumped_from([cue(1), cue(2), cue(3)]);
    let (spot, history) = history.back(cue(4), 1, everywhere).expect("a place behind");
    assert_eq!(spot, cue(3));
    let (spot, history) = history.back(spot, 2, everywhere).expect("places behind");
    assert_eq!(spot, cue(1));
    let (spot, history) = history.forward(spot, 1, everywhere).expect("a place ahead");
    assert_eq!(spot, cue(2));
    let (spot, history) = history.forward(spot, 5, everywhere).expect("places ahead");
    assert_eq!(spot, cue(4));
    assert_eq!(history.forward(spot, 1, everywhere), None);
    assert!(history.can_go_back(spot, everywhere));
    assert!(!history.can_go_forward(spot, everywhere));
}

#[test]
fn a_count_past_the_oldest_place_goes_to_the_oldest() {
    let history = jumped_from([cue(1), cue(2), cue(3)]);
    let (spot, _) = history.back(cue(4), 50, everywhere).expect("places behind");
    assert_eq!(spot, cue(1));
}

#[test]
fn a_place_is_remembered_once() {
    let history = jumped_from([cue(1), cue(2), cue(1), cue(2)]);
    assert_eq!(history.behind, [cue(1), cue(2)]);
}

#[test]
fn a_jump_after_going_back_keeps_the_places_ahead_behind() {
    let history = jumped_from([cue(1), cue(2)]);
    let (spot, history) = history.back(cue(3), 1, everywhere).expect("a place behind");
    assert!(history.can_go_forward(spot, everywhere));
    let history = history.jumped(spot);
    assert_eq!(history.ahead, []);
    assert_eq!(history.behind, [cue(1), cue(3), cue(2)]);
}

#[test]
fn going_back_after_a_jump_visits_what_vim_visits() {
    // 10G 20G Control+o 30G Control+o Control+o, from the first line
    let history = jumped_from([cue(1), cue(10)]);
    let (spot, history) = history
        .back(cue(20), 1, everywhere)
        .expect("a place behind");
    assert_eq!(spot, cue(10));
    let history = history.jumped(spot);
    let (spot, history) = history
        .back(cue(30), 1, everywhere)
        .expect("a place behind");
    assert_eq!(spot, cue(10));
    let (spot, _) = history.back(spot, 1, everywhere).expect("a place behind");
    assert_eq!(spot, cue(20));
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
    let (_, history) = history
        .back(cue(151), 1, everywhere)
        .expect("places behind");
    assert_eq!(history.behind.len() + history.ahead.len(), 100);
}

#[test]
fn a_way_back_that_no_jump_recorded_is_remembered_to_go_forward_to() {
    let history = History::default().left(Spot::Output(OutputId(1)));
    assert!(!history.can_go_back(cue(1), everywhere));
    let (spot, _) = history
        .forward(cue(1), 1, everywhere)
        .expect("a place ahead");
    assert_eq!(spot, Spot::Output(OutputId(1)));
}

#[test]
fn an_output_is_a_place_apart_from_a_cue_and_a_list() {
    let history = jumped_from([Spot::Output(OutputId(1)), Spot::Outputs, Spot::List(0)]);
    assert_eq!(history.behind.len(), 3);
}
