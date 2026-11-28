// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of pasting cues that lie in groups

use super::*;

/// Returns the cues of `ids` with the cues that they hold, as a copy takes them
fn copied(workspace: &Workspace, ids: &[u32]) -> Vec<Cue> {
    workspace.lists()[0]
        .with_members(&cue_ids(ids))
        .into_iter()
        .filter_map(|id| workspace.cue(id).cloned())
        .collect()
}

/// Returns a list with a group that holds a sound and a group of its own
fn nested() -> Workspace {
    workspace(vec![
        audio(1),
        group(2),
        within(1, audio(3)),
        within(1, group(4)),
        within(2, audio(5)),
    ])
}

#[test]
fn pasted_group_keeps_its_shape_at_the_top() {
    let list = nested();
    let clipboard = copied(&list, &[2]);
    assert_eq!(clipboard.len(), 4);
    let pasted = list.apply(paste(clipboard, Place::After(CueId(1))));
    assert_eq!(ids(&pasted), [1, 6, 7, 8, 9, 2, 3, 4, 5]);
    assert_eq!(depths(&pasted), [0, 0, 1, 1, 2, 0, 1, 1, 2]);
}

#[test]
fn pasted_group_goes_into_the_group_that_it_lands_below() {
    let list = nested();
    let clipboard = copied(&list, &[2]);
    let pasted = list.apply(paste(clipboard, Place::After(CueId(2))));
    assert_eq!(ids(&pasted), [1, 2, 6, 7, 8, 9, 3, 4, 5]);
    assert_eq!(depths(&pasted), [0, 0, 1, 2, 2, 3, 1, 1, 2]);
}

#[test]
fn cues_copied_from_inside_a_group_paste_at_the_depth_of_the_place() {
    let list = nested();
    let clipboard = copied(&list, &[3, 4]);
    assert_eq!(clipboard.len(), 3);
    let top = list
        .clone()
        .apply(paste(clipboard.clone(), Place::After(CueId(1))));
    assert_eq!(depths(&top), [0, 0, 0, 1, 0, 1, 1, 2]);
    let end = list.clone().apply(paste(clipboard.clone(), Place::End));
    assert_eq!(depths(&end), [0, 0, 1, 1, 2, 2, 2, 3]);
    let inside = list.apply(paste(clipboard, Place::Before(CueId(3))));
    assert_eq!(ids(&inside), [1, 2, 6, 7, 8, 3, 4, 5]);
    assert_eq!(depths(&inside), [0, 0, 1, 1, 2, 1, 1, 2]);
}

#[test]
fn clipboard_pasted_several_times_starts_each_round_at_the_depth() {
    let list = nested();
    let once = copied(&list, &[2]);
    let twice: Vec<Cue> = once.iter().chain(&once).cloned().collect();
    let pasted = list.apply(paste(twice, Place::After(CueId(2))));
    assert_eq!(depths(&pasted)[2..10], [1, 2, 2, 3, 1, 2, 2, 3]);
}

#[test]
fn undo_and_redo_of_a_paste_keep_the_depths() {
    let list = nested();
    let clipboard = copied(&list, &[2]);
    let pasted = list.clone().apply(paste(clipboard, Place::After(CueId(2))));
    let undone = pasted.apply(Command::Undo);
    assert_eq!(ids(&undone), ids(&list));
    assert_eq!(depths(&undone), depths(&list));
    let redone = undone.apply(Command::Redo);
    assert_eq!(depths(&redone), [0, 0, 1, 2, 2, 3, 1, 1, 2]);
}

#[test]
fn control_cue_pasted_with_its_group_targets_the_copy() {
    let list = workspace(vec![group(1), within(1, stop_cue(2, 1)), audio(3)]);
    let both = list.clone().apply(paste(copied(&list, &[1]), Place::End));
    assert_eq!(ids(&both), [1, 2, 3, 4, 5]);
    assert_eq!(target_of(&both, 5), Some(CueId(4)));
    let alone = list.clone().apply(paste(copied(&list, &[2]), Place::End));
    assert_eq!(target_of(&alone, 4), Some(CueId(1)));
}

#[test]
fn cut_group_keeps_its_numbers_where_it_is_pasted() {
    let list = nested();
    let clipboard = copied(&list, &[2]);
    let cut = list.apply(Command::Cut(clipboard.iter().map(|cue| cue.id).collect()));
    assert_eq!(ids(&cut), [1]);
    let pasted = cut.apply(paste(clipboard, Place::End));
    let numbers: Vec<_> = pasted.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.number.clone())
        .collect();
    assert_eq!(numbers, ["1", "2", "3", "4", "5"]);
    assert_eq!(depths(&pasted), [0, 0, 1, 1, 2]);
}

#[test]
fn cue_copied_after_a_nested_group_pastes_beside_it_and_not_inside() {
    let list = workspace(vec![
        group(1),
        within(1, group(2)),
        within(2, audio(3)),
        within(1, audio(4)),
    ]);
    let clipboard = copied(&list, &[2, 4]);
    assert_eq!(clipboard.len(), 3);
    let pasted = workspace(vec![audio(9)]).apply(paste(clipboard, Place::End));
    assert_eq!(depths(&pasted), [0, 0, 1, 0]);
}
