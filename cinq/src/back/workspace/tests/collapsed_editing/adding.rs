// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of adding cues next to collapsed groups

use super::*;

#[test]
fn cue_added_below_a_collapsed_group_lands_below_its_cues() {
    let before = closed();
    let added = before
        .clone()
        .apply(add(0, Kind::Memo, Place::After(CueId(2))));
    assert_rows(&added, &[1, 2, 3, 4, 5, 7, 6], &[0, 0, 1, 1, 2, 0, 0]);
    assert_undoes(&before, &added);
}

#[test]
fn cue_added_below_a_collapsed_group_in_a_group_stays_in_that_group() {
    let before = workspace(vec![
        group(1),
        within(1, group(2)),
        within(2, audio(3)),
        within(1, audio(4)),
    ])
    .apply(collapse(&[2], true));
    let added = before
        .clone()
        .apply(add(0, Kind::Memo, Place::After(CueId(2))));
    assert_rows(&added, &[1, 2, 3, 5, 4], &[0, 1, 2, 1, 1]);
    assert_undoes(&before, &added);
}

#[test]
fn cues_added_in_a_row_below_a_collapsed_group_stay_side_by_side() {
    let added = closed().apply(add_row(Kind::Memo, Place::After(CueId(2)), 3));
    assert_eq!(ids(&added), [1, 2, 3, 4, 5, 7, 8, 9, 6]);
    assert_eq!(depths(&added), [0, 0, 1, 1, 2, 0, 0, 0, 0]);
}

#[test]
fn group_added_below_a_collapsed_group_lies_beside_it() {
    let added = closed().apply(add(0, Kind::Group, Place::After(CueId(2))));
    assert_eq!(depths(&added), [0, 0, 1, 1, 2, 0, 0]);
    assert!(!added.cue(CueId(7)).is_some_and(|cue| cue.collapsed));
}

#[test]
fn cue_added_below_an_expanded_group_still_goes_into_it() {
    let open = closed().apply(collapse(&[2], false));
    let added = open.apply(add(0, Kind::Memo, Place::After(CueId(2))));
    assert_rows(&added, &[1, 2, 7, 3, 4, 5, 6], &[0, 0, 1, 1, 1, 2, 0]);
}

#[test]
fn cue_added_above_a_collapsed_group_takes_its_depth() {
    let added = closed().apply(add(0, Kind::Memo, Place::Before(CueId(2))));
    assert_rows(&added, &[1, 7, 2, 3, 4, 5, 6], &[0, 0, 0, 1, 1, 2, 0]);
}

#[test]
fn cue_added_at_the_end_below_a_collapsed_group_lies_outside_it() {
    let before = workspace(vec![group(1), within(1, group(2)), within(2, audio(3))])
        .apply(collapse(&[1], true));
    let added = before.apply(add(0, Kind::Stop, Place::End));
    assert_eq!(depths(&added), [0, 1, 2, 0]);
    assert_eq!(target_of(&added, 4), Some(CueId(1)));
}

#[test]
fn control_cue_added_below_a_collapsed_group_targets_the_group() {
    [Kind::Stop, Kind::Pause, Kind::Start, Kind::GoTo]
        .into_iter()
        .for_each(|kind| {
            let added = closed().apply(add(0, kind, Place::After(CueId(2))));
            assert_eq!(target_of(&added, 7), Some(CueId(2)), "{kind:?}");
            assert_eq!(depths(&added), [0, 0, 1, 1, 2, 0, 0]);
        });
}

#[test]
fn control_cue_added_next_to_a_collapsed_group_targets_what_the_table_shows_above() {
    let below = closed().apply(add(0, Kind::Stop, Place::Before(CueId(6))));
    assert_eq!(target_of(&below, 7), Some(CueId(2)));
    let above = closed().apply(add(0, Kind::Stop, Place::Before(CueId(2))));
    assert_eq!(target_of(&above, 7), Some(CueId(1)));
}

#[test]
fn control_cue_added_below_a_collapsed_group_that_it_would_lie_in_has_no_target() {
    let inside = closed().apply(add(0, Kind::Stop, Place::After(CueId(3))));
    assert_eq!(depths(&inside), [0, 0, 1, 1, 1, 2, 0]);
    assert_eq!(target_of(&inside, 7), None);
}

#[test]
fn fade_and_devamp_cues_added_below_a_collapsed_group_have_no_target() {
    [Kind::Fade, Kind::Devamp].into_iter().for_each(|kind| {
        let added = closed().apply(add(0, kind, Place::After(CueId(2))));
        assert_eq!(target_of(&added, 7), None, "{kind:?}");
    });
}
