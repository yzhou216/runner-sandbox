// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of telling a text field from a drop-down list that has the keyboard

use super::*;

struct Widget(bool);

impl Focusable for Widget {
    fn is_focused(&self) -> bool {
        self.0
    }

    fn focus(&mut self) {}

    fn unfocus(&mut self) {}
}

impl TextInput for Widget {
    fn text(&self) -> &str {
        ""
    }

    fn move_cursor_to_front(&mut self) {}

    fn move_cursor_to_end(&mut self) {}

    fn move_cursor_to(&mut self, _position: usize) {}

    fn select_all(&mut self) {}

    fn select_range(&mut self, _start: usize, _end: usize) {}
}

/// Visits widgets as iced does, a text field reporting as a text input
/// before it reports as focusable and a list only as focusable, and returns
/// whether a text field and whether a list has the keyboard
fn holding(widgets: &[(bool, bool)]) -> (bool, bool) {
    let mut focus = HasFocus::default();
    widgets.iter().for_each(|&(text, focused)| {
        if text {
            focus.text_input(None, Rectangle::default(), &mut Widget(focused));
        }
        focus.focusable(None, Rectangle::default(), &mut Widget(focused));
    });
    (focus.text, focus.list)
}

#[test]
fn a_text_field_with_the_keyboard_is_no_list_and_a_list_with_it_is_no_text_field() {
    assert_eq!(holding(&[(true, true), (false, false)]), (true, false));
    assert_eq!(holding(&[(true, false), (false, true)]), (false, true));
    assert_eq!(holding(&[(true, false), (false, false)]), (false, false));
}
