// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of going back and forward between the places that jumps left

use iced::keyboard::{Key, Modifiers};

use super::{chord, show};
use crate::{
    back::{CueId, Place},
    front::{App, Message, Tab},
};

/// Returns the state after the messages, which a test sends one at a time
pub(super) fn after(app: App, messages: impl IntoIterator<Item = Message>) -> App {
    messages
        .into_iter()
        .fold(app, |app, message| app.next(message).0)
}

pub(super) fn cue(id: u32) -> Option<CueId> {
    Some(CueId(id))
}

/// Returns the list and the selection, the place that the operator sees
pub(super) fn place(app: &App) -> (usize, Option<CueId>) {
    (app.list, app.selected)
}

#[test]
fn switching_lists_is_a_jump_that_back_returns_from() {
    let app = after(show(cue(5)), [Message::SwitchList(1)]);
    assert_eq!(place(&app), (1, cue(20)));
    assert_eq!(place(&after(app, [Message::Back(1)])), (0, cue(5)));
}

/// Returns the state after the keys, each with the modifiers held
fn after_keys(app: App, keys: &[(&str, Modifiers)]) -> App {
    keys.iter().fold(app, |app, &(key, modifiers)| {
        app.typed(chord(Key::Character(key.into()), modifiers), false)
            .0
    })
}

#[test]
fn the_key_of_back_does_what_its_message_does() {
    let jumped = || after(show(cue(5)), [Message::SwitchList(1)]);
    let by_key = after_keys(jumped(), &[("o", Modifiers::CTRL)]);
    assert_eq!(place(&by_key), (0, cue(5)));
    assert_eq!(place(&by_key), place(&after(jumped(), [Message::Back(1)])));
    let again = after_keys(by_key, &[("i", Modifiers::CTRL)]);
    assert_eq!(place(&again), (1, cue(20)));
}

#[test]
fn back_and_forward_retrace_the_places() {
    let app = after(
        show(cue(5)),
        [Message::SwitchList(1), Message::Row(2), Message::Back(2)],
    );
    assert_eq!(place(&app), (0, cue(5)));
    let app = after(app, [Message::Forward(1)]);
    assert_eq!(place(&app), (1, cue(20)));
    let app = after(app, [Message::Forward(1)]);
    assert_eq!(place(&app), (1, cue(21)));
    assert_eq!(place(&after(app, [Message::Forward(1)])), (1, cue(21)));
}

#[test]
fn a_jump_after_going_back_keeps_the_places_ahead_behind() {
    let jumped = || {
        after(
            show(cue(5)),
            [Message::SwitchList(1), Message::Back(1), Message::Row(1)],
        )
    };
    let app = jumped();
    assert_eq!(place(&app), (0, cue(1)));
    assert!(!app.can_go_forward());
    assert_eq!(place(&after(jumped(), [Message::Forward(1)])), (0, cue(1)));
    let app = after(app, [Message::Back(1)]);
    assert_eq!(place(&app), (0, cue(5)));
    assert_eq!(place(&after(app, [Message::Back(1)])), (1, cue(20)));
}

#[test]
fn back_after_a_jump_visits_what_vim_visits() {
    // 10G 20G Control+o 30G Control+o Control+o, with rows for lines
    let app = after(
        show(cue(1)),
        [
            Message::Row(10),
            Message::Row(11),
            Message::Back(1),
            Message::Row(12),
            Message::Back(1),
        ],
    );
    assert_eq!(place(&app), (0, cue(10)));
    assert_eq!(place(&after(app, [Message::Back(1)])), (0, cue(11)));
}

#[test]
fn going_back_or_forward_takes_the_keyboard_out_of_a_field() {
    let jumped = || after(show(cue(5)), [Message::SwitchList(1)]);
    let backed = || after(jumped(), [Message::Back(1)]);
    let leaves = |app: App, message| {
        let before = place(&app);
        let (moved, task) = app.next(message);
        (place(&moved) != before, task.units())
    };
    // The reveal and the unfocus
    assert_eq!(leaves(jumped(), Message::Back(1)), (true, 2));
    assert_eq!(leaves(backed(), Message::Forward(1)), (true, 2));
    // Nothing to walk to, so the keyboard stays where it is
    assert_eq!(leaves(show(cue(5)), Message::Back(1)), (false, 1));
    let forward = after(backed(), [Message::Forward(1)]);
    assert_eq!(leaves(forward, Message::Forward(1)), (false, 1));
}

#[test]
fn a_place_is_remembered_once() {
    let rows = [Message::Row(1), Message::Row(usize::MAX)];
    let app = after(show(cue(5)), rows.iter().cloned().cycle().take(4));
    assert_eq!(place(&app), (0, cue(12)));
    let app = after(app, [Message::Back(1)]);
    assert_eq!(place(&app), (0, cue(1)));
    assert_eq!(place(&after(app, [Message::Back(1)])), (0, cue(5)));
}

#[test]
fn a_count_goes_that_many_places_back() {
    let app = after(
        show(cue(5)),
        [Message::Row(1), Message::Row(2), Message::Row(3)],
    );
    let app = after_keys(app, &[("2", Modifiers::empty()), ("o", Modifiers::CTRL)]);
    assert_eq!(place(&app), (0, cue(1)));
}

#[test]
fn moving_by_rows_editing_and_looking_elsewhere_record_nothing() {
    let app = after(
        show(cue(5)),
        [
            Message::Step(2),
            Message::Add(Place::After, 1),
            Message::Move(1),
            Message::Delete(1),
            Message::Undo(1),
            Message::Redo(1),
            Message::Go,
            Message::Tab(Tab::Levels),
            Message::StepTab(1),
            Message::ShowTab(2),
            Message::ToggleHelp,
            Message::CloseHelp,
            Message::Select(CueId(2)),
        ],
    );
    assert!(!app.can_go_back());
    let there = place(&app);
    assert_eq!(place(&after(app, [Message::Back(1)])), there);
}

#[test]
fn selecting_in_another_list_is_a_jump_and_in_the_same_list_is_not() {
    let same = after(show(cue(5)), [Message::Select(CueId(7))]);
    assert!(!same.can_go_back());
    let other = after(show(cue(5)), [Message::Select(CueId(21))]);
    assert_eq!(place(&other), (1, cue(21)));
    assert_eq!(place(&after(other, [Message::Back(1)])), (0, cue(5)));
}

#[test]
fn a_jump_to_the_place_already_shown_records_nothing() {
    let app = after(show(cue(12)), [Message::Row(usize::MAX), Message::Row(12)]);
    assert!(!app.can_go_back());
    let app = after(show(cue(1)), [Message::SwitchList(0)]);
    assert!(!app.can_go_back());
}

#[test]
fn a_place_whose_cue_was_deleted_is_skipped() {
    let app = after(
        show(cue(5)),
        [
            Message::Row(1),
            Message::Row(12),
            Message::Select(CueId(1)),
            Message::Delete(1),
        ],
    );
    assert_eq!(app.workspace.list_of(CueId(1)), None);
    assert_eq!(place(&after(app, [Message::Back(1)])), (0, cue(5)));
}

#[test]
fn a_place_follows_its_cue_as_it_moves() {
    let app = after(
        show(cue(5)),
        [
            Message::Row(12),
            Message::Select(CueId(5)),
            Message::Move(3),
            Message::Select(CueId(12)),
            Message::Back(1),
        ],
    );
    assert_eq!(place(&app), (0, cue(5)));
    assert_eq!(app.selected_row(), Some(7));
}

#[test]
fn a_list_without_a_selected_cue_is_a_place_by_its_index() {
    let app = App {
        selected: None,
        ..show(None)
    };
    let app = after(
        app,
        [Message::SwitchList(1), Message::Row(2), Message::Back(2)],
    );
    assert_eq!(place(&app), (0, None));
}
