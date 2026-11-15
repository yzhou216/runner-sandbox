// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding cues in a row as one change

use super::*;

fn add_in_a_row(at: Place, count: usize) -> Command {
    Command::Add {
        list: 0,
        kind: Kind::Memo,
        at,
        count,
    }
}

#[test]
fn cues_added_in_a_row_follow_one_another_with_the_next_numbers() {
    let added = workspace(vec![audio(1), audio(2), audio(3)])
        .apply(add_in_a_row(Place::After(CueId(1)), 3));
    assert_eq!(ids(&added), [1, 4, 5, 6, 2, 3]);
    let numbers: Vec<_> = added.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.number.clone())
        .collect();
    assert_eq!(numbers, ["1", "4", "5", "6", "2", "3"]);
    let before =
        workspace(vec![audio(1), audio(2)]).apply(add_in_a_row(Place::Before(CueId(2)), 2));
    assert_eq!(ids(&before), [1, 3, 4, 2]);
}

#[test]
fn cues_added_in_a_row_are_one_change_for_undo_and_redo() {
    let added = workspace(vec![audio(1), audio(2)]).apply(add_in_a_row(Place::End, 4));
    assert_eq!(ids(&added), [1, 2, 3, 4, 5, 6]);
    let undone = added.apply(Command::Undo);
    assert_eq!(ids(&undone), [1, 2]);
    let redone = undone.apply(Command::Redo);
    assert_eq!(ids(&redone), [1, 2, 3, 4, 5, 6]);
}

#[test]
fn only_the_first_of_the_cues_added_at_the_end_takes_standby() {
    let played = workspace(vec![audio(1)]).apply(GO);
    let added = played.apply(add_in_a_row(Place::End, 3));
    assert_eq!(standby(&added), Some(2));
    let undone = added.apply(Command::Undo);
    assert_eq!(standby(&undone), None);
}

#[test]
fn no_cues_are_added_to_a_list_that_is_not_there() {
    let shown = workspace(vec![audio(1)]);
    let added = shown.clone().apply(Command::Add {
        list: 5,
        kind: Kind::Memo,
        at: Place::End,
        count: 3,
    });
    assert_eq!(ids(&added), [1]);
    assert!(added.latest_change().is_none());
}
