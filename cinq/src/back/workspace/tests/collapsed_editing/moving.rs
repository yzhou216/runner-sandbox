// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of pasting, moving, and indenting cues next to collapsed groups

use super::*;

#[test]
fn cues_pasted_below_a_collapsed_group_land_below_its_cues() {
    let before = closed();
    let clipboard = vec![group(10), within(1, audio(11))];
    let pasted = before
        .clone()
        .apply(paste(clipboard, Place::After(CueId(2))));
    assert_eq!(ids(&pasted), [1, 2, 3, 4, 5, 7, 8, 6]);
    assert_eq!(depths(&pasted), [0, 0, 1, 1, 2, 0, 1, 0]);
    assert_undoes(&before, &pasted);
    let above = before.apply(paste(vec![audio(10)], Place::Before(CueId(2))));
    assert_eq!(ids(&above), [1, 7, 2, 3, 4, 5, 6]);
}

#[test]
fn block_moved_down_past_a_collapsed_group_lands_below_its_cues() {
    let before = closed();
    let moved = before.clone().apply(moving(1, 1));
    assert_rows(&moved, &[2, 3, 4, 5, 1, 6], &[0, 1, 1, 2, 0, 0]);
    assert!(matches!(moved.undoable(), Some(Change::Moved { .. })));
    assert_undoes(&before, &moved);
}

#[test]
fn block_moved_up_past_a_collapsed_group_lands_above_it() {
    let before = closed();
    let moved = before.clone().apply(moving(6, -1));
    assert_rows(&moved, &[1, 6, 2, 3, 4, 5], &[0, 0, 0, 1, 1, 2]);
    assert_undoes(&before, &moved);
}

#[test]
fn collapsed_group_moves_as_one_row_with_its_cues() {
    let before = closed();
    let down = before.clone().apply(moving(2, 1));
    assert_rows(&down, &[1, 6, 2, 3, 4, 5], &[0, 0, 0, 1, 1, 2]);
    assert!(matches!(down.undoable(), Some(Change::Regrouped { .. })));
    assert_undoes(&before, &down);
    let up = before.clone().apply(moving(2, -1));
    assert_rows(&up, &[2, 3, 4, 5, 1, 6], &[0, 1, 1, 2, 0, 0]);
    assert_undoes(&before, &up);
}

#[test]
fn move_counts_each_collapsed_group_as_one_row() {
    let before = workspace(vec![
        audio(1),
        group(2),
        within(1, audio(3)),
        group(4),
        within(1, audio(5)),
        audio(6),
    ])
    .apply(collapse(&[2], true))
    .apply(collapse(&[4], true));
    let moved = before.clone().apply(moving(1, 2));
    assert_rows(&moved, &[2, 3, 4, 5, 1, 6], &[0, 1, 0, 1, 0, 0]);
    assert_undoes(&before, &moved);
    let clamped = before.clone().apply(moving(1, 10));
    assert_eq!(ids(&clamped), [2, 3, 4, 5, 6, 1]);
    let past_top = before.apply(moving(6, -10));
    assert_eq!(ids(&past_top), [6, 1, 2, 3, 4, 5]);
}

#[test]
fn move_past_a_collapsed_group_in_a_group_stays_in_that_group() {
    let before = workspace(vec![
        group(1),
        within(1, audio(2)),
        within(1, group(3)),
        within(2, audio(4)),
        within(1, audio(5)),
        audio(6),
    ])
    .apply(collapse(&[3], true));
    let moved = before.clone().apply(moving(2, 1));
    assert_rows(&moved, &[1, 3, 4, 2, 5, 6], &[0, 1, 2, 1, 1, 0]);
    assert!(matches!(moved.undoable(), Some(Change::Moved { .. })));
    assert_undoes(&before, &moved);
}

#[test]
fn move_next_to_an_expanded_group_works_as_before() {
    let open = closed().apply(collapse(&[2], false));
    let moved = open.clone().apply(moving(1, 1));
    assert_rows(&moved, &[2, 1, 3, 4, 5, 6], &[0, 1, 1, 1, 2, 0]);
    assert_undoes(&open, &moved);
}

#[test]
fn row_below_a_collapsed_group_indents_into_it_as_its_last_cue() {
    let before = closed();
    let indented = before.clone().apply(Command::Indent(vec![CueId(6)]));
    assert_rows(&indented, &[1, 2, 3, 4, 5, 6], &[0, 0, 1, 1, 2, 1]);
    assert_eq!(
        indented.lists()[0]
            .groups()
            .group_of(CueId(6))
            .map(|g| g.id),
        Some(CueId(2))
    );
    assert!(matches!(
        indented.undoable(),
        Some(Change::Regrouped { .. })
    ));
    assert_undoes(&before, &indented);
}

#[test]
fn collapsed_group_below_a_collapsed_group_indents_into_it_with_its_cues() {
    let before = workspace(vec![
        group(1),
        within(1, audio(2)),
        group(3),
        within(1, audio(4)),
    ])
    .apply(collapse(&[1], true))
    .apply(collapse(&[3], true));
    let indented = before.clone().apply(Command::Indent(vec![CueId(3)]));
    assert_eq!(depths(&indented), [0, 1, 1, 2]);
    assert_undoes(&before, &indented);
}
