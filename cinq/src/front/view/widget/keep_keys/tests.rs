// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of telling a text field from a drop-down list that has the keyboard

use super::*;
use crate::front::fixtures::Field;

/// Visits widgets as iced does, a text field reporting as a text input
/// before it reports as focusable and a list only as focusable, and returns
/// whether a text field and whether a list has the keyboard
fn holding(widgets: &[(bool, bool)]) -> (bool, bool) {
    let mut focus = HasFocus::default();
    widgets.iter().for_each(|&(text, focused)| {
        if text {
            focus.text_input(None, Rectangle::default(), &mut Field::new(focused));
        }
        focus.focusable(None, Rectangle::default(), &mut Field::new(focused));
    });
    (focus.text, focus.list)
}

#[test]
fn a_text_field_with_the_keyboard_is_no_list_and_a_list_with_it_is_no_text_field() {
    assert_eq!(holding(&[(true, true), (false, false)]), (true, false));
    assert_eq!(holding(&[(true, false), (false, true)]), (false, true));
    assert_eq!(holding(&[(true, false), (false, false)]), (false, false));
}
