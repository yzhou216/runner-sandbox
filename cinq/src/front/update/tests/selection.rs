// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the selection after deleting, undo, and standby move

use super::show;
use crate::{
    back::{CueId, Place},
    front::{App, Message},
};

#[test]
fn deleting_selects_the_cue_after_or_else_the_one_before() {
    let deleting = |list: usize, selected: u32, count: usize| {
        let app = App {
            list,
            ..show(Some(CueId(selected)))
        };
        app.handled(Message::Delete(count)).0.selected
    };
    assert_eq!(deleting(0, 4, 2), Some(CueId(6)));
    assert_eq!(deleting(0, 11, 5), Some(CueId(10)));
    assert_eq!(deleting(1, 20, 2), None);
}

#[test]
fn undoing_an_addition_or_a_paste_selects_the_cue_that_was_selected_when_it_was_made() {
    let selected_after = |messages: Vec<Message>| {
        let app = [Message::Select(CueId(4)), Message::Copy(1)]
            .into_iter()
            .chain(messages)
            .fold(show(Some(CueId(4))), |app, message| app.handled(message).0);
        app.selected
    };
    let undo = |message| selected_after(vec![message, Message::Undo(1)]);
    // Below and above, as `o u`, `O u`, `p u`, and `P u` of Vim end
    assert_eq!(undo(Message::Add(Place::After, 1)), Some(CueId(4)));
    assert_eq!(undo(Message::Add(Place::Before, 1)), Some(CueId(4)));
    assert_eq!(undo(Message::Paste(Place::After, 1)), Some(CueId(4)));
    assert_eq!(undo(Message::Paste(Place::Before, 1)), Some(CueId(4)));
    // Paste of several cues, and the selection moved before the undo
    let moved = selected_after(vec![
        Message::Paste(Place::After, 3),
        Message::Select(CueId(9)),
        Message::Undo(1),
    ]);
    assert_eq!(moved, Some(CueId(4)));
    // Redo selects what was added again
    let redone = selected_after(vec![
        Message::Add(Place::After, 1),
        Message::Undo(1),
        Message::Redo(1),
    ]);
    assert_eq!(redone, Some(CueId(22)));
}

#[test]
fn redoing_a_counted_addition_puts_all_its_cues_back_with_the_first_selected() {
    let (added, _) = show(Some(CueId(4))).handled(Message::Add(Place::After, 3));
    let (redone, _) = added.handled(Message::Undo(1)).0.handled(Message::Redo(1));
    assert_eq!(redone.workspace.lists()[0].cues.len(), 15);
    assert_eq!(redone.selected, Some(CueId(22)));
}

#[test]
fn standby_selected_without_a_selected_cue_leaves_standby_alone() {
    let app = show(None);
    let standby = app.workspace.lists()[0].standby;
    let (after, _) = app.handled(Message::StandbySelected);
    assert_eq!(after.workspace.lists()[0].standby, standby);
    assert!(after.workspace.undoable().is_none());
}

#[test]
fn selection_on_standby_moves_on_with_it_and_stays_elsewhere() {
    let going = |selected| show(Some(CueId(selected))).handled(Message::Go).0.selected;
    assert_eq!(going(1), Some(CueId(2)));
    assert_eq!(going(5), Some(CueId(5)));
}

#[test]
fn undo_and_redo_of_a_standby_move_select_the_cue_that_standby_is_on() {
    let moved = show(Some(CueId(4)))
        .handled(Message::StandbySelected)
        .0
        .handled(Message::Select(CueId(9)))
        .0;
    let undone = moved.handled(Message::Undo(1)).0;
    assert_eq!(undone.selected, Some(CueId(1)));
    let redone = undone.handled(Message::Redo(1)).0;
    assert_eq!(redone.selected, Some(CueId(4)));
}
