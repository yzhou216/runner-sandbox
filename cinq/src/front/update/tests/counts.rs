// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the actions that follow the count typed before their keys: undo
//! and redo by several changes, and the inspector tab by number or by steps

use super::show;
use crate::{
    back::{CueId, Turn},
    front::{App, Message, Tab, keymap::Action},
};

/// Returns the state after three deletions in a row, of the cues 4, 5, and
/// 6 of the twelve in the first list
fn deleted_thrice() -> App {
    (0..3).fold(show(Some(CueId(4))), |app, _| {
        app.handled(Message::Delete(1)).0
    })
}

fn cues(app: &App) -> usize {
    app.workspace.lists()[0].cues.len()
}

fn latest(app: &App) -> Option<(Turn, Option<CueId>)> {
    app.workspace
        .latest_change()
        .map(|(turn, change)| (turn, change.cue()))
}

#[test]
fn undo_by_a_count_takes_back_that_many_changes_and_tells_the_last() {
    let app = deleted_thrice();
    assert_eq!(cues(&app), 9);
    let (undone, _) = app.handled(Message::Undo(2));
    assert_eq!(cues(&undone), 11);
    assert_eq!(undone.selected, Some(CueId(5)));
    assert_eq!(latest(&undone), Some((Turn::Undone, Some(CueId(5)))));
}

#[test]
fn undo_by_more_than_there_is_stops_at_the_oldest_change() {
    let (undone, _) = deleted_thrice().handled(Message::Undo(usize::MAX));
    assert_eq!(cues(&undone), 12);
    assert!(undone.workspace.undoable().is_none());
    assert_eq!(latest(&undone), Some((Turn::Undone, Some(CueId(4)))));
}

#[test]
fn redo_by_a_count_makes_that_many_changes_again_and_stops_at_the_newest() {
    let (undone, _) = deleted_thrice().handled(Message::Undo(3));
    let (redone, _) = undone.handled(Message::Redo(2));
    assert_eq!(cues(&redone), 10);
    assert_eq!(latest(&redone), Some((Turn::Redone, Some(CueId(5)))));
    let (redone, _) = redone.handled(Message::Redo(usize::MAX));
    assert_eq!(cues(&redone), 9);
    assert!(redone.workspace.redoable().is_none());
}

#[test]
fn undo_and_redo_with_nothing_to_turn_change_nothing() {
    let (undone, _) = show(Some(CueId(4))).handled(Message::Undo(3));
    assert_eq!((cues(&undone), undone.selected), (12, Some(CueId(4))));
    assert!(undone.workspace.latest_change().is_none());
    let (redone, _) = deleted_thrice().handled(Message::Redo(3));
    assert_eq!(cues(&redone), 9);
}

#[test]
fn tab_by_a_count_is_the_tab_of_that_number_or_the_last() {
    let shown = |action: Action, count: Option<usize>, tab: Tab| {
        App {
            tab,
            ..App::default()
        }
        .handled(action.message(count))
        .0
        .tab
    };
    assert_eq!(shown(Action::NextTab, Some(1), Tab::Levels), Tab::Basics);
    assert_eq!(shown(Action::NextTab, Some(2), Tab::Basics), Tab::Time);
    assert_eq!(shown(Action::NextTab, Some(3), Tab::Basics), Tab::Levels);
    assert_eq!(shown(Action::NextTab, Some(9), Tab::Basics), Tab::Levels);
    assert_eq!(shown(Action::NextTab, None, Tab::Levels), Tab::Basics);
    assert_eq!(shown(Action::NextTab, None, Tab::Basics), Tab::Time);
}

#[test]
fn tab_back_by_a_count_wraps_as_a_single_step_does() {
    let shown = |count: Option<usize>, tab: Tab| {
        App {
            tab,
            ..App::default()
        }
        .handled(Action::PreviousTab.message(count))
        .0
        .tab
    };
    assert_eq!(shown(None, Tab::Basics), Tab::Levels);
    assert_eq!(shown(Some(2), Tab::Levels), Tab::Basics);
    assert_eq!(shown(Some(4), Tab::Levels), Tab::Time);
    // A count as large as a number can hold, which a sum would overflow
    assert_eq!(shown(Some(usize::MAX), Tab::Time), Tab::Time.stepped(-1));
}
