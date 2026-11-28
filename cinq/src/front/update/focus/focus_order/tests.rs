// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the order in which Tab visits the fields

use super::*;
use crate::front::fixtures::Field;

/// Reads a window as iced visits it: a field of the toolbar, the inspector
/// with two fields, and a field of the sidebar, the second field of the
/// inspector having the keyboard if `focused` names it
fn window(focused: Option<usize>) -> Vec<Slot> {
    let mut reading = Reading::new("inspector");
    let mut fields: Vec<Field> = (0..4).map(|at| Field::new(focused == Some(at))).collect();
    let [toolbar, first, second, sidebar] = &mut fields[..] else {
        unreachable!("the window has four fields")
    };
    reading.focusable(None, Rectangle::default(), toolbar);
    reading.container(Some(&Id::new("inspector")), Rectangle::default());
    reading.traverse(&mut |reading| {
        reading.focusable(None, Rectangle::default(), first);
        reading.focusable(None, Rectangle::default(), second);
    });
    reading.focusable(None, Rectangle::default(), sidebar);
    reading.slots
}

#[test]
fn the_fields_of_the_inspector_are_those_inside_its_container() {
    let inspector: Vec<_> = window(None).iter().map(|slot| slot.inspector).collect();
    assert_eq!(inspector, [false, true, true, false]);
}

#[test]
fn tab_from_no_field_goes_first_to_the_inspector_then_to_the_toolbar_and_the_sidebar() {
    let visited = |mut at: Option<usize>| {
        (0..5)
            .map(|_| {
                at = chosen(&window(at), false).filter(|&place| place < 4);
                at
            })
            .collect::<Vec<_>>()
    };
    // Places of the window: toolbar 0, inspector 1 and 2, sidebar 3, and
    // after the last field no field, which holds the place 4
    assert_eq!(
        visited(None),
        [Some(1), Some(2), Some(0), Some(3), None],
        "the round ends with no field, which the next Tab starts again from"
    );
}

#[test]
fn tab_past_the_last_field_and_shift_tab_before_the_first_leave_every_field() {
    assert_eq!(chosen(&window(Some(3)), false), Some(4));
    assert_eq!(chosen(&window(Some(1)), true), Some(4));
}

#[test]
fn shift_tab_goes_the_other_way_round_from_the_last_field() {
    assert_eq!(chosen(&window(None), true), Some(3));
    assert_eq!(chosen(&window(Some(3)), true), Some(0));
    assert_eq!(chosen(&window(Some(0)), true), Some(2));
    assert_eq!(chosen(&window(Some(2)), true), Some(1));
}

#[test]
fn a_window_with_no_field_has_none_to_move_to() {
    assert_eq!(chosen(&[], false), None);
}

#[test]
fn giving_focuses_one_field_and_takes_the_keyboard_from_the_rest() {
    let mut fields: Vec<Field> = (0..3).map(|_| Field::new(true)).collect();
    let mut giving = Giving {
        visited: 0,
        target: 1,
    };
    fields.iter_mut().for_each(|field| {
        Operation::<()>::focusable(&mut giving, None, Rectangle::default(), field);
    });
    let focused: Vec<_> = fields.iter().map(|field| field.focused).collect();
    assert_eq!(focused, [false, true, false]);
}
