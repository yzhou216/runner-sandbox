// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of moving cues up and down in a list with groups

use super::*;
use crate::back::Change;

#[test]
fn cue_moved_onto_a_group_goes_into_it() {
    let list = workspace(vec![audio(1), group(2), within(1, audio(3)), audio(4)]);
    let moved = list.apply(moving(1, 1));
    assert_rows(&moved, &[2, 1, 3, 4], &[0, 1, 1, 0]);
    assert!(matches!(moved.undoable(), Some(Change::Regrouped { .. })));
    let undone = moved.apply(Command::Undo);
    assert_rows(&undone, &[1, 2, 3, 4], &[0, 0, 1, 0]);
    let redone = undone.apply(Command::Redo);
    assert_rows(&redone, &[2, 1, 3, 4], &[0, 1, 1, 0]);
}

#[test]
fn cue_moved_within_a_group_is_a_plain_move() {
    let list = workspace(vec![group(1), within(1, audio(2)), within(1, audio(3))]);
    let moved = list.apply(moving(2, 1));
    assert_rows(&moved, &[1, 3, 2], &[0, 1, 1]);
    assert!(matches!(moved.undoable(), Some(Change::Moved { .. })));
}

#[test]
fn cue_moved_past_the_end_of_its_group_leaves_it() {
    let list = workspace(vec![group(1), within(1, audio(2)), audio(3)]);
    let moved = list.apply(moving(2, 1));
    assert_rows(&moved, &[1, 3, 2], &[0, 0, 0]);
    let up =
        workspace(vec![group(1), within(1, audio(2)), within(1, audio(3))]).apply(moving(2, -1));
    assert_rows(&up, &[2, 1, 3], &[0, 0, 1]);
}

#[test]
fn cue_moved_up_below_a_group_goes_into_it() {
    let list = workspace(vec![group(1), within(1, audio(2)), audio(3)]);
    let moved = list.apply(moving(3, -1));
    assert_rows(&moved, &[1, 3, 2], &[0, 1, 1]);
}

#[test]
fn group_moves_with_the_cues_that_it_holds() {
    let list = workspace(vec![group(1), within(1, audio(2)), audio(3), audio(4)]);
    let moved = list.clone().apply(moving(1, 1));
    assert_rows(&moved, &[3, 1, 2, 4], &[0, 0, 1, 0]);
    let again = moved.apply(moving(1, 1));
    assert_eq!(ids(&again), [3, 4, 1, 2]);
    let clamped = list.apply(moving(1, 10));
    assert_rows(&clamped, &[3, 4, 1, 2], &[0, 0, 0, 1]);
}

#[test]
fn group_moved_onto_a_group_takes_the_cues_it_holds_in_with_it() {
    let list = workspace(vec![
        group(1),
        within(1, audio(2)),
        group(3),
        within(1, audio(4)),
    ]);
    let moved = list.clone().apply(moving(1, 1));
    assert_rows(&moved, &[3, 1, 2, 4], &[0, 1, 2, 1]);
    let undone = moved.apply(Command::Undo);
    assert_eq!(ids(&undone), [1, 2, 3, 4]);
    assert_eq!(raw_depths(&undone), raw_depths(&list));
}

#[test]
fn group_moved_up_out_of_a_group_leaves_with_its_cues() {
    let list = workspace(vec![
        group(1),
        within(1, audio(2)),
        within(1, group(3)),
        within(2, audio(4)),
    ]);
    let moved = list.apply(moving(3, -2));
    assert_rows(&moved, &[3, 4, 1, 2], &[0, 1, 0, 1]);
}

#[test]
fn undo_brings_back_the_depth_that_a_cue_kept() {
    let list = workspace(vec![group(1), within(1, audio(2)), within(5, audio(3))]);
    assert_eq!(depths(&list), [0, 1, 1]);
    let moved = list.apply(moving(3, -1));
    assert_eq!(raw_depths(&moved), [0, 1, 1]);
    let undone = moved.apply(Command::Undo);
    assert_eq!(raw_depths(&undone), [0, 1, 5]);
}

#[test]
fn moves_of_one_cue_in_a_row_are_one_change() {
    let list = workspace(vec![
        audio(1),
        group(2),
        within(1, audio(3)),
        within(1, audio(4)),
        audio(5),
    ]);
    let moved = (0..3).fold(list.clone(), |list, _| list.apply(moving(1, 1)));
    assert_rows(&moved, &[2, 3, 4, 1, 5], &[0, 1, 1, 1, 0]);
    let undone = moved.apply(Command::Undo);
    assert_eq!(ids(&undone), ids(&list));
    assert_eq!(depths(&undone), depths(&list));
    assert!(undone.undoable().is_none());
    let redone = undone.apply(Command::Redo);
    assert_rows(&redone, &[2, 3, 4, 1, 5], &[0, 1, 1, 1, 0]);
}

#[test]
fn plain_move_followed_by_one_into_a_group_is_one_change() {
    let list = workspace(vec![audio(1), audio(2), group(3), within(1, audio(4))]);
    let moved = list.clone().apply(moving(1, 1)).apply(moving(1, 1));
    assert_rows(&moved, &[2, 3, 1, 4], &[0, 0, 1, 1]);
    let undone = moved.apply(Command::Undo);
    assert_eq!(ids(&undone), ids(&list));
    assert!(undone.undoable().is_none());
}

#[test]
fn moves_that_end_where_they_began_leave_nothing_to_undo() {
    let list = workspace(vec![audio(1), group(2), within(1, audio(3))]);
    let back = list.apply(moving(1, 1)).apply(moving(1, -1));
    assert_rows(&back, &[1, 2, 3], &[0, 0, 1]);
    assert!(back.undoable().is_none());
}
