// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the operators that act on the rows a motion reaches, and of the
//! keys that type them

use super::*;
use crate::front::keymap::Action::{self, Copy, Cut, Delete, First, Last, Next, Previous};

/// Returns the identities of the cues of the first list in order
pub(super) fn rows(app: &App) -> Vec<u32> {
    crate::back::ids(&app.workspace)
}

pub(super) fn across(app: App, operator: Action, motion: Action, count: Option<usize>) -> App {
    app.handled(Message::Across(operator, motion, count)).0
}

/// Leaves show mode, deletes every cue, then pastes the clipboard and
/// returns the numbers of the cues pasted, which keep their numbers since no
/// other cue holds them
fn pasted(app: App) -> Vec<String> {
    let app = after(
        app,
        [
            Message::ShowMode(false),
            Message::Select(CueId(1)),
            Message::Delete(12),
            Message::Paste(Place::After, 1),
        ],
    );
    app.workspace.lists()[0]
        .cues
        .iter()
        .map(|cue| cue.number.clone())
        .collect()
}

#[test]
fn cutting_down_takes_the_rows_in_one_change_and_fills_the_clipboard_in_order() {
    let app = across(show(Some(CueId(4))), Cut, Next, Some(2));
    assert_eq!(rows(&app), [1, 2, 3, 7, 8, 9, 10, 11, 12]);
    // Cue that took the place of the first row cut
    assert_eq!(app.selected, Some(CueId(7)));
    // One change takes all the rows
    let undone = across(show(Some(CueId(4))), Cut, Next, Some(2)).handled(Message::Undo(1));
    assert_eq!(rows(&undone.0), (1..=12).collect::<Vec<_>>());
    assert_eq!(pasted(app), ["4", "5", "6"]);
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
    let to_first = across(show(Some(CueId(3))), Copy, First, None);
    assert_eq!(to_first.selected, Some(CueId(1)));
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

/// Returns the show in show mode with cue 4 selected
fn locked() -> App {
    show(Some(CueId(4))).handled(Message::ShowMode(true)).0
}

#[test]
fn show_mode_lets_copying_over_a_motion_act() {
    let copied = across(locked(), Copy, Previous, Some(2));
    assert_eq!(copied.selected, Some(CueId(2)));
    assert_eq!(rows(&copied).len(), 12);
    assert!(copied.workspace.latest_change().is_none());
    assert_eq!(pasted(copied), ["2", "3", "4"]);
}

#[test]
fn keys_of_vi_copy_over_a_motion_in_show_mode_but_cut_nothing() {
    let cut = typing(locked(), "d j");
    assert_eq!(rows(&cut).len(), 12);
    assert_eq!(cut.selected, Some(CueId(4)));
    assert!(cut.keys.is_empty());
    let copied = typing(cut, "y G");
    assert_eq!(rows(&copied).len(), 12);
    assert_eq!(
        pasted(copied),
        ["4", "5", "6", "7", "8", "9", "10", "11", "12"]
    );
}

#[test]
fn keys_of_vi_cut_and_copy_over_a_motion_with_counts_on_both_sides() {
    let cut = typing(show(Some(CueId(2))), "d j");
    assert_eq!(rows(&cut), [1, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    assert!(cut.keys.is_empty());
    let cut = typing(show(Some(CueId(2))), "2 d 3 j");
    assert_eq!(rows(&cut), [1, 9, 10, 11, 12]);
    let cut = typing(show(Some(CueId(5))), "d G");
    assert_eq!(rows(&cut), [1, 2, 3, 4]);
    let cut = typing(show(Some(CueId(5))), "d 3 G");
    assert_eq!(rows(&cut), [1, 2, 6, 7, 8, 9, 10, 11, 12]);
    let cut = typing(show(Some(CueId(5))), "d g g");
    assert_eq!(rows(&cut), (6..=12).collect::<Vec<_>>());
    let copied = typing(show(Some(CueId(5))), "y k");
    assert_eq!(copied.selected, Some(CueId(4)));
    assert_eq!(rows(&copied).len(), 12);
}

#[test]
fn operator_typed_halfway_shows_in_the_keys_and_acts_only_when_complete() {
    let app = typing(show(Some(CueId(2))), "2 d");
    assert_eq!(app.keys.len(), 2);
    let app = typing(app, "3");
    assert_eq!(app.keys.len(), 3);
    assert_eq!(rows(&app).len(), 12);
    let app = typing(app, "j");
    assert!(app.keys.is_empty());
    assert_eq!(rows(&app).len(), 12 - 7);
}

#[test]
fn operator_dropped_by_a_stray_key_cuts_nothing() {
    let app = typing(show(Some(CueId(2))), "d w j");
    assert_eq!(rows(&app).len(), 12);
    assert_eq!(app.selected, Some(CueId(3)));
}

#[test]
fn help_page_ignores_an_operator_with_its_motion() {
    let app = helping(show(Some(CueId(2))));
    let app = typing(app, "d j");
    assert_eq!(rows(&app).len(), 12);
    assert_eq!(app.selected, Some(CueId(2)));
}

#[test]
fn cutting_or_deleting_the_standby_cue_moves_standby_to_the_cue_that_takes_its_place() {
    let standby = |app: &App| app.workspace.lists()[0].standby;
    let first = || show(Some(CueId(1)));
    assert_eq!(standby(&first()), Some(CueId(1)));
    let cut = across(first(), Cut, Next, None);
    assert_eq!(standby(&cut), Some(CueId(3)));
    let deleted = across(first(), Delete, Next, None);
    assert_eq!(standby(&deleted), Some(CueId(3)));
    let one = first().handled(Message::Cut(1)).0;
    assert_eq!(standby(&one), Some(CueId(2)));
    let several = first().handled(Message::Delete(3)).0;
    assert_eq!(standby(&several), Some(CueId(4)));
    // Standby on the last cue has nothing below it to move to
    let last = show(Some(CueId(12)))
        .handled(Message::StandbySelected)
        .0
        .handled(Message::Cut(1))
        .0;
    assert_eq!(standby(&last), None);
}

#[test]
fn an_operator_typed_halfway_is_dropped_when_the_keyboard_goes_to_another_window_and_back() {
    let waiting = typing(show(Some(CueId(4))), "d");
    assert_eq!(waiting.keys.len(), 1);
    let (back, _) = waiting.next(Message::WindowFocused(iced::window::Id::unique()));
    assert!(back.keys.is_empty());
    assert_eq!(rows(&typing(back, "j")).len(), 12);
}
