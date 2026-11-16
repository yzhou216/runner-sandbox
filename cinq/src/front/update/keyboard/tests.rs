// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of finding the widget that has the keyboard

use iced::Rectangle;

use super::*;

struct Field(bool);

impl Focusable for Field {
    fn is_focused(&self) -> bool {
        self.0
    }

    fn focus(&mut self) {}

    fn unfocus(&mut self) {}
}

/// Visits one widget per entry of `fields`, as iced does for each window in
/// turn, each with the identity it names, and returns what `holding` finds
fn found(mut holding: Holding, fields: &[(Option<&'static str>, bool)]) -> bool {
    fields.iter().for_each(|&(id, focused)| {
        let id = id.map(Id::new);
        holding.focusable(id.as_ref(), Rectangle::default(), &mut Field(focused));
    });
    matches!(holding.finish(), Outcome::Some(true))
}

#[test]
fn a_focused_widget_anywhere_means_the_keyboard_is_held() {
    let holding = || Holding::anywhere();
    assert!(found(
        holding(),
        &[(None, false), (None, true), (None, false)]
    ));
    assert!(!found(holding(), &[(None, false), (None, false)]));
    assert!(!found(holding(), &[]));
}

#[test]
fn only_the_focused_widget_of_the_named_field_counts_for_that_field() {
    let holding = || Holding::in_field("file");
    assert!(found(
        holding(),
        &[(Some("name"), false), (Some("file"), true)]
    ));
    assert!(!found(
        holding(),
        &[(Some("name"), true), (Some("file"), false)]
    ));
    assert!(!found(holding(), &[(None, true)]));
}
