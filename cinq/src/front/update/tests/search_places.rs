// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of what happens to the search line and the selection when something
//! other than typing comes: clicks, going back and forward, moving between
//! fields, opening a show, and the jumps that searches make

use iced::{
    keyboard::{Key, key::Named},
    mouse,
};

use super::{
    after, press,
    search::{opened, opening, searched, typed},
    typing, windowed,
};
use crate::{
    back::CueId,
    front::{App, Message, message::Focus},
};

#[test]
fn a_click_closes_the_line_and_keeps_the_selection_where_it_is() {
    let typing = typed(opened(2), "wind");
    assert_eq!(typing.selected, Some(CueId(5)));
    let (clicked, _) = typing.next(Message::Clicked);
    assert!(clicked.search.is_none());
    assert_eq!(clicked.selected, Some(CueId(5)));
    // Nothing was accepted, so the keys of the next match do nothing yet
    assert_eq!(clicked.searched.text, "");
}

#[test]
fn selecting_a_cue_with_the_pointer_closes_the_line_in_either_order_of_the_click() {
    for messages in [
        vec![Message::Select(CueId(3))],
        vec![Message::Clicked, Message::Select(CueId(3))],
        vec![Message::Select(CueId(3)), Message::Clicked],
    ] {
        let app = after(typed(opened(2), "wind"), messages.clone());
        assert!(app.search.is_none(), "{messages:?}");
        assert_eq!(app.selected, Some(CueId(3)), "{messages:?}");
    }
}

/// Returns the state after the key of back
fn back(app: App) -> App {
    typing(app, "Control+o")
}

#[test]
fn accepting_a_search_is_a_jump_from_where_it_began() {
    let accepted = searched(opened(2), "wind");
    assert_eq!(accepted.selected, Some(CueId(5)));
    assert_eq!(back(accepted).selected, Some(CueId(2)));
    // A search that stayed on the cue is no jump
    assert!(!searched(opened(5), "wind").can_go_back());
}

#[test]
fn the_moves_of_typing_and_a_cancel_leave_the_history_as_it_was() {
    let typing = typed(typed(opened(2), "w"), "wind");
    assert!(!typing.can_go_back());
    let (cancelled, _) = typing.typed(press(Key::Named(Named::Escape)), false);
    assert_eq!(cancelled.selected, Some(CueId(2)));
    assert!(!cancelled.can_go_back());
    let (backspaced, _) =
        typed(typed(opened(2), "w"), "").handled(Message::Searching(String::new()));
    assert!(!backspaced.can_go_back());
    // Click keeps the preview and accepts nothing
    let (clicked, _) = typed(opened(2), "wind").next(Message::Clicked);
    assert!(!clicked.can_go_back());
}

#[test]
fn the_keys_of_next_and_previous_match_are_jumps() {
    let accepted = searched(opened(3), "storm");
    assert_eq!(accepted.selected, Some(CueId(4)));
    let next = accepted.handled(Message::SearchStep(1)).0;
    assert_eq!(next.selected, Some(CueId(1)));
    assert_eq!(back(next).selected, Some(CueId(4)));
    let previous = searched(opened(3), "storm")
        .handled(Message::SearchStep(-1))
        .0;
    assert_eq!(previous.selected, Some(CueId(1)));
    assert_eq!(back(previous).selected, Some(CueId(4)));
}

#[test]
fn an_empty_line_accepted_repeats_the_search_as_a_jump() {
    let again = opening(searched(opened(1), "storm"), false)
        .handled(Message::SearchAccepted)
        .0;
    assert_eq!(again.selected, Some(CueId(1)));
    assert_eq!(back(again).selected, Some(CueId(4)));
}

#[test]
fn going_back_closes_the_line_and_keeps_the_selection_where_the_search_put_it() {
    let closed = back(typed(opened(2), "wind"));
    assert!(closed.search.is_none());
    assert_eq!(closed.selected, Some(CueId(5)));
    // Escape after it has no search to put back
    let (escaped, _) = closed.typed(press(Key::Named(Named::Escape)), false);
    assert_eq!(escaped.selected, Some(CueId(5)));
}

#[test]
fn going_forward_and_the_buttons_of_the_mouse_close_the_line() {
    assert!(
        typing(typed(opened(2), "wind"), "Control+i")
            .search
            .is_none()
    );
    for button in [mouse::Button::Back, mouse::Button::Forward] {
        let (app, main) = windowed(typed(opened(2), "wind"));
        let (closed, _) = app.next(Message::SidePressed(main, button));
        assert!(closed.search.is_none(), "{button:?}");
        assert_eq!(closed.selected, Some(CueId(5)), "{button:?}");
    }
}

#[test]
fn moving_to_another_field_closes_the_line_and_keeps_the_selection() {
    for focus in [Focus::Next, Focus::Previous] {
        let (closed, _) = typed(opened(2), "wind").focused(focus.clone());
        assert!(closed.search.is_none(), "{focus:?}");
        assert_eq!(closed.selected, Some(CueId(5)), "{focus:?}");
    }
}
