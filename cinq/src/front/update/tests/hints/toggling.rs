// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of turning hint mode on and off by its action, and of the labels that it shows for the places in view

use super::*;

#[test]
fn the_action_turns_hint_mode_on_and_a_second_press_turns_it_off() {
    let (on, task) = show(None).hinting();
    assert!(on.hints.is_some());
    assert!(
        task.units() > 0,
        "the widgets are asked where the places lie"
    );
    let (off, task) = on.hinting();
    assert!(off.hints.is_none());
    assert_eq!(task.units(), 0);
}

#[test]
fn places_that_arrive_after_the_mode_went_off_get_no_labels() {
    let (on, _) = show(None).hinting();
    let off = on.without_hints();
    assert!(off.hinted(vec![place(Act::Reach, 0.0)]).hints.is_none());
    let none = show(None).hinting().0.hinted(Vec::new());
    assert!(none.hints.is_none(), "no place in view leaves the mode");
}

#[test]
fn each_label_shows_left_of_its_place_at_its_height() {
    let app = labelled();
    let corners: Vec<_> = app.hints.iter().flat_map(|hints| hints.labels()).collect();
    let left_of = |x| {
        hint::corners(&[(
            Rectangle::new(Point::new(x, 10.0), Size::new(20.0, 20.0)),
            1,
        )])[0]
    };
    assert_eq!(
        corners,
        [
            ("s", left_of(0.0)),
            ("a", left_of(60.0)),
            ("d", left_of(120.0)),
        ]
    );
    assert_eq!(
        corners[0].1,
        Point::new(20.0, 10.0),
        "right of the place that lies at the window edge"
    );
    assert!(corners[1].1.x < 60.0 && corners[2].1.x < 120.0);
}

#[test]
fn keys_of_the_action_act_once_a_press_and_over_the_help_page() {
    use crate::front::keymap::Action;
    assert!(Action::Hint.acts_once_a_press());
    assert!(Action::Hint.acts_over_help());
    assert!(!Action::Hint.acts_in_fields());
}
