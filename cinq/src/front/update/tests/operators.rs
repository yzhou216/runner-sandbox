// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the operators that act on the rows a motion reaches, and of the
//! keys that type them

use super::*;
use crate::front::keymap::Action::{self, Copy, Cut, Delete, First, Last, Next, Previous};

/// Returns the identities of the cues of the first list in order
pub(super) fn rows(app: &App) -> Vec<u32> {
    app.workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.id.0)
        .collect()
}

pub(super) fn across(app: App, operator: Action, motion: Action, count: Option<usize>) -> App {
    app.handled(Message::Across(operator, motion, count)).0
}

/// Leaves show mode, deletes every cue, then pastes the clipboard and
/// returns the numbers of the cues pasted, which keep their numbers since no
/// other cue holds them
fn pasted(app: App) -> Vec<String> {
    let cleared = app
        .handled(Message::ShowMode(false))
        .0
        .handled(Message::Select(CueId(1)))
        .0
        .handled(Message::Delete(12))
        .0;
    let app = cleared.handled(Message::Paste(Place::After, 1)).0;
    app.workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.number.clone())
        .collect()
}

pub(super) fn typing(app: App, keys: &[&str]) -> App {
    keys.iter().fold(app, |app, &key| {
        app.typed(press(Key::Character(key.into())), false).0
    })
}

#[test]
fn cutting_down_takes_the_rows_in_one_change_and_fills_the_clipboard_in_order() {
    let app = across(show(Some(CueId(4))), Cut, Next, Some(2));
    assert_eq!(rows(&app), [1, 2, 3, 7, 8, 9, 10, 11, 12]);
    // The cue that took the place of the first row cut
    assert_eq!(app.selected, Some(CueId(7)));
    assert_eq!(pasted(app), ["4", "5", "6"]);
}

#[test]
fn cutting_several_rows_is_one_change_for_undo() {
    let app = across(show(Some(CueId(4))), Cut, Next, Some(2));
    let undone = app.handled(Message::Undo(1)).0;
    assert_eq!(rows(&undone), (1..=12).collect::<Vec<_>>());
}

#[test]
fn cutting_up_takes_the_rows_above_and_selects_the_cue_that_takes_their_place() {
    let app = across(show(Some(CueId(4))), Cut, Previous, Some(2));
    assert_eq!(rows(&app), [1, 5, 6, 7, 8, 9, 10, 11, 12]);
    assert_eq!(app.selected, Some(CueId(5)));
    assert_eq!(pasted(app), ["2", "3", "4"]);
}

#[test]
fn cutting_to_the_last_cue_leaves_the_selection_on_the_new_last_one() {
    let app = across(show(Some(CueId(10))), Cut, Last, None);
    assert_eq!(rows(&app), (1..=9).collect::<Vec<_>>());
    assert_eq!(app.selected, Some(CueId(9)));
    assert_eq!(pasted(app), ["10", "11", "12"]);
}

#[test]
fn cutting_to_the_first_cue_or_the_row_counted_goes_either_way() {
    let app = across(show(Some(CueId(3))), Cut, First, None);
    assert_eq!(rows(&app), (4..=12).collect::<Vec<_>>());
    assert_eq!(app.selected, Some(CueId(4)));
    // Row 2 is above the selection on row 5, and row 8 below it
    let up = across(show(Some(CueId(5))), Cut, First, Some(2));
    assert_eq!(rows(&up), [1, 6, 7, 8, 9, 10, 11, 12]);
    let down = across(show(Some(CueId(5))), Cut, Last, Some(8));
    assert_eq!(rows(&down), [1, 2, 3, 4, 9, 10, 11, 12]);
    assert_eq!(down.selected, Some(CueId(9)));
}

#[test]
fn copying_down_leaves_the_show_and_the_selection_alone() {
    let app = across(show(Some(CueId(4))), Copy, Next, Some(2));
    assert_eq!(rows(&app), (1..=12).collect::<Vec<_>>());
    assert_eq!(app.selected, Some(CueId(4)));
    assert!(app.workspace.latest_change().is_none());
    assert_eq!(pasted(app), ["4", "5", "6"]);
}

#[test]
fn copying_up_selects_the_first_row_copied_as_vim_does() {
    let app = across(show(Some(CueId(4))), Copy, Previous, Some(2));
    assert_eq!(app.selected, Some(CueId(2)));
    assert_eq!(rows(&app).len(), 12);
    assert_eq!(pasted(app), ["2", "3", "4"]);
    let last = across(show(Some(CueId(3))), Copy, First, None);
    assert_eq!(last.selected, Some(CueId(1)));
}

#[test]
fn deleting_over_a_motion_leaves_the_clipboard_alone() {
    let copied = across(show(Some(CueId(2))), Copy, Next, None);
    let app = across(
        copied.handled(Message::Select(CueId(6))).0,
        Delete,
        Next,
        Some(1),
    );
    assert_eq!(rows(&app), [1, 2, 3, 4, 5, 8, 9, 10, 11, 12]);
    assert_eq!(pasted(app), ["2", "3"]);
}

#[test]
fn motion_that_cannot_move_does_nothing() {
    let first = show(Some(CueId(1)));
    let last = show(Some(CueId(12)));
    [(first, Previous), (last, Next)]
        .into_iter()
        .for_each(|(app, motion)| {
            let selected = app.selected;
            let app = across(across(app, Cut, motion, None), Copy, motion, Some(3));
            assert_eq!(rows(&app).len(), 12);
            assert_eq!(app.selected, selected);
            assert!(app.workspace.latest_change().is_none());
            assert!(pasted(app).is_empty(), "the clipboard stayed empty");
        });
}

#[test]
fn no_selection_does_nothing() {
    let none = across(show(None), Cut, Next, None);
    assert_eq!(rows(&none).len(), 12);
    let none = across(show(None), Copy, Next, None);
    assert!(pasted(none).is_empty());
}

#[test]
fn show_mode_locks_cutting_and_deleting_over_a_motion() {
    let (locked, _) = show(Some(CueId(4))).handled(Message::ShowMode(true));
    let refused = across(across(locked, Cut, Next, Some(2)), Delete, Next, Some(2));
    assert_eq!(rows(&refused).len(), 12);
    assert_eq!(refused.selected, Some(CueId(4)));
    assert!(refused.workspace.latest_change().is_none());
    assert!(
        pasted(refused).is_empty(),
        "cut copied nothing while locked"
    );
}

#[test]
fn show_mode_lets_copying_over_a_motion_act() {
    let (locked, _) = show(Some(CueId(4))).handled(Message::ShowMode(true));
    let copied = across(locked, Copy, Previous, Some(2));
    assert_eq!(copied.selected, Some(CueId(2)));
    assert_eq!(rows(&copied).len(), 12);
    assert!(copied.workspace.latest_change().is_none());
    assert_eq!(pasted(copied), ["2", "3", "4"]);
}

#[test]
fn keys_of_vi_copy_over_a_motion_in_show_mode_but_cut_nothing() {
    let (locked, _) = show(Some(CueId(4))).handled(Message::ShowMode(true));
    let cut = typing(locked, &["d", "j"]);
    assert_eq!(rows(&cut).len(), 12);
    assert_eq!(cut.selected, Some(CueId(4)));
    assert!(cut.keys.is_empty());
    let copied = typing(cut, &["y", "G"]);
    assert_eq!(rows(&copied).len(), 12);
    assert_eq!(
        pasted(copied),
        ["4", "5", "6", "7", "8", "9", "10", "11", "12"]
    );
}

#[test]
fn keys_of_vi_cut_and_copy_over_a_motion_with_counts_on_both_sides() {
    let cut = typing(show(Some(CueId(2))), &["d", "j"]);
    assert_eq!(rows(&cut), [1, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    assert!(cut.keys.is_empty());
    let cut = typing(show(Some(CueId(2))), &["2", "d", "3", "j"]);
    assert_eq!(rows(&cut), [1, 9, 10, 11, 12]);
    let cut = typing(show(Some(CueId(5))), &["d", "G"]);
    assert_eq!(rows(&cut), [1, 2, 3, 4]);
    let cut = typing(show(Some(CueId(5))), &["d", "3", "G"]);
    assert_eq!(rows(&cut), [1, 2, 6, 7, 8, 9, 10, 11, 12]);
    let cut = typing(show(Some(CueId(5))), &["d", "g", "g"]);
    assert_eq!(rows(&cut), (6..=12).collect::<Vec<_>>());
    let copied = typing(show(Some(CueId(5))), &["y", "k"]);
    assert_eq!(copied.selected, Some(CueId(4)));
    assert_eq!(rows(&copied).len(), 12);
}

#[test]
fn operator_typed_halfway_shows_in_the_keys_and_acts_only_when_complete() {
    let app = typing(show(Some(CueId(2))), &["2", "d"]);
    assert_eq!(app.keys.len(), 2);
    let app = typing(app, &["3"]);
    assert_eq!(app.keys.len(), 3);
    assert_eq!(rows(&app).len(), 12);
    let app = typing(app, &["j"]);
    assert!(app.keys.is_empty());
    assert_eq!(rows(&app).len(), 12 - 7);
}

#[test]
fn operator_dropped_by_a_stray_key_cuts_nothing() {
    let app = typing(show(Some(CueId(2))), &["d", "w", "j"]);
    assert_eq!(rows(&app).len(), 12);
    assert_eq!(app.selected, Some(CueId(3)));
}

#[test]
fn help_page_ignores_an_operator_with_its_motion() {
    let app = App {
        help: true,
        ..show(Some(CueId(2)))
    };
    let app = typing(app, &["d", "j"]);
    assert_eq!(rows(&app).len(), 12);
    assert_eq!(app.selected, Some(CueId(2)));
}
