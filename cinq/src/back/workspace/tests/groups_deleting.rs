// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of deleting, cutting, and copying groups, which take the cues that a
//! group holds

use super::*;
use crate::back::Change;

#[test]
fn deleting_a_group_takes_every_cue_that_it_holds() {
    let deleted = nested().apply(delete(&[2]));
    assert_eq!(ids(&deleted), [1, 6]);
}

#[test]
fn deleting_a_cue_in_a_group_leaves_the_group_and_the_rest() {
    let deleted = nested().apply(delete(&[3]));
    assert_eq!(ids(&deleted), [1, 2, 4, 5, 6]);
    let inner = nested().apply(delete(&[4]));
    assert_eq!(ids(&inner), [1, 2, 3, 6]);
}

#[test]
fn undo_brings_back_a_deleted_group_with_its_depths() {
    let list = nested();
    let deleted = list.clone().apply(delete(&[2]));
    let undone = deleted.apply(Command::Undo);
    assert_eq!(ids(&undone), ids(&list));
    assert_eq!(depths(&undone), [0, 0, 1, 1, 2, 0]);
    let redone = undone.apply(Command::Redo);
    assert_eq!(ids(&redone), [1, 6]);
    let again = redone.apply(Command::Undo);
    assert_eq!(depths(&again), [0, 0, 1, 1, 2, 0]);
}

#[test]
fn group_and_a_cue_in_it_deleted_together_are_deleted_once() {
    let deleted = nested().apply(delete(&[3, 2, 2]));
    assert_eq!(ids(&deleted), [1, 6]);
    let Some(Change::Deleted { cues, .. }) = deleted.undoable() else {
        panic!("the deletion is recorded");
    };
    assert_eq!(cues.len(), 4);
}

#[test]
fn cut_takes_a_group_as_deletion_does() {
    let cut = nested().apply(Command::Cut(vec![CueId(4)]));
    assert_eq!(ids(&cut), [1, 2, 3, 6]);
    assert!(matches!(
        cut.undoable(),
        Some(Change::Deleted { cut: true, .. })
    ));
}

#[test]
fn standby_in_a_deleted_group_moves_past_it_and_undo_moves_it_back() {
    let list = nested().apply(move_standby(5));
    let deleted = list.apply(delete(&[2]));
    assert_eq!(standby(&deleted), Some(6));
    let undone = deleted.apply(Command::Undo);
    assert_eq!(standby(&undone), Some(5));
}

#[test]
fn deleting_a_group_stops_the_cues_that_it_holds() {
    let fired = workspace(vec![group(1), within(1, audio(2)), within(1, audio(3))]).apply(GO);
    assert!(running(&fired).ends_with(&[2, 3]));
    let deleted = fired.apply(delete(&[1]));
    assert!(running(&deleted).is_empty());
}

#[test]
fn members_of_cues_come_once_each_in_the_order_of_the_list() {
    let list = nested();
    let with = |ids: &[u32]| -> Vec<u32> {
        list.lists()[0]
            .with_members(&cue_ids(ids))
            .iter()
            .map(|id| id.0)
            .collect()
    };
    assert_eq!(with(&[2]), [2, 3, 4, 5]);
    assert_eq!(with(&[6, 4, 4]), [4, 5, 6]);
    assert_eq!(with(&[5, 2, 99]), [2, 3, 4, 5]);
    assert_eq!(with(&[1]), [1]);
    assert!(with(&[99]).is_empty());
}
