// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of going back and forward between the places that jumps have left

use super::{after, show, typing};
use crate::{
    back::{CueId, Place},
    front::{App, Message, Spot, Tab},
};

pub(super) fn cue(id: u32) -> Option<CueId> {
    Some(CueId(id))
}

/// Returns the state after the jump from cue 5 to the second list
pub(super) fn jumped() -> App {
    after(show(cue(5)), [Message::SwitchList(1)])
}

/// Returns the list and the selection, the place that the operator sees
pub(super) fn place(app: &App) -> (usize, Option<CueId>) {
    (app.list, app.selected)
}

#[test]
fn switching_lists_is_a_jump_that_back_returns_from_by_its_message_or_its_key() {
    assert_eq!(place(&jumped()), (1, cue(20)));
    let by_message = after(jumped(), [Message::Back(1)]);
    assert_eq!(place(&by_message), (0, cue(5)));
    let by_key = typing(jumped(), "Control+o");
    assert_eq!(place(&by_key), place(&by_message));
    assert_eq!(place(&typing(by_key, "Control+i")), (1, cue(20)));
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
    let backed = || after(jumped(), [Message::Back(1)]);
    let leaves = |app: App, message| {
        let before = place(&app);
        let (moved, task) = app.next(message);
        (place(&moved) != before, task.units())
    };
    // Reveal with the measure that follows it, the unfocus, and the scroll of
    // the sidebar to the list
    assert_eq!(leaves(jumped(), Message::Back(1)), (true, 4));
    assert_eq!(leaves(backed(), Message::Forward(1)), (true, 4));
    // Nothing to walk to, so the keyboard stays where it is and no unfocus
    assert_eq!(leaves(show(cue(5)), Message::Back(1)), (false, 2));
    let forward = after(backed(), [Message::Forward(1)]);
    assert_eq!(leaves(forward, Message::Forward(1)), (false, 2));
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
    let app = typing(app, "2 Control+o");
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

#[test]
fn the_places_that_back_and_forward_go_to_are_known_before_they_are_taken() {
    let app = jumped();
    assert_eq!(app.back_to(), Some(Spot::Cue(CueId(5))));
    assert_eq!(app.forward_to(), None);
    let back = after(app, [Message::Back(1)]);
    assert_eq!(back.back_to(), None);
    assert_eq!(back.forward_to(), Some(Spot::Cue(CueId(20))));
}

#[test]
fn the_way_out_of_the_outputs_goes_to_the_list_that_showed_where_no_jump_left_one() {
    let opened = show(cue(5));
    assert_eq!(opened.back_to(), None);
    let outputs = App {
        outputs_shown: true,
        ..opened
    };
    assert_eq!(outputs.back_to(), Some(Spot::List(0)));
}
