// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the mouse in an open menu: the click beside it, the row under
//! the pointer, and the highlight that follows the selected option

use iced::{
    Event, Point, Size,
    advanced::layout::{Layout, Node},
    mouse,
};

use super::{super::rows::Rows, *};

#[test]
fn click_beside_the_menu_closes_it_and_acts_unless_it_lands_on_the_list() {
    assert_eq!(
        outcome(|choice, shell| choice.click_beside(false, shell)),
        closed(false, None)
    );
    assert_eq!(
        outcome(|choice, shell| choice.click_beside(true, shell)),
        closed(true, None)
    );
}

#[test]
fn mouse_highlights_the_row_under_it_and_picks_the_row_it_presses() {
    let rows = Node::with_children(
        Size::new(100.0, 90.0),
        (0..3)
            .map(|row| {
                Node::new(Size::new(100.0, 30.0)).move_to(Point::new(0.0, 30.0 * row as f32))
            })
            .collect(),
    );
    let at = |y| mouse::Cursor::Available(Point::new(10.0, y));
    let moved = Event::Mouse(mouse::Event::CursorMoved {
        position: Point::new(10.0, 40.0),
    });
    let press = Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left));
    let picked = outcome(|choice, shell| {
        let list = Rows(choice);
        list.before(&moved, Layout::new(&rows), at(40.0), shell);
        assert_eq!(choice.shared.highlight.get(), Some(1));
        assert!(!shell.is_event_captured());
        list.before(&press, Layout::new(&rows), at(100.0), shell);
        assert_eq!(choice.shared.highlight.get(), Some(1));
        list.before(&press, Layout::new(&rows), at(70.0), shell);
    });
    assert_eq!(picked, closed(true, Some(3)));
}

#[test]
fn highlight_follows_the_selected_option_until_a_key_moves_it() {
    let keymap = Keymap::default();
    let shared = Shared::default();
    let mut messages = Vec::new();
    let list =
        dropdown(OPTIONS, Some(OPTIONS[0]), Message::Step, &keymap).on_open(Message::Clicked);
    Choice::new(&list, &shared).open(&mut Shell::new(&mut messages));
    assert!(matches!(
        messages[..],
        [Message::MenuOpen(true), Message::Clicked]
    ));
    // Options listed afresh under the open menu put the selected one last
    let relisted = dropdown([3, 2, 1], Some(OPTIONS[0]), Message::Step, &keymap);
    let choice = Choice::new(&relisted, &shared);
    assert_eq!(choice.highlighted(), Some(2));
    typing(named(Named::ArrowUp))(choice, &mut Shell::new(&mut messages));
    assert_eq!(choice.highlighted(), Some(1));
}
