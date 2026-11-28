// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the keys that add and cut outputs while they show

use iced::keyboard::{Key, Modifiers, key::Named};

use super::{ALL, locked, names, on, on_clipboard, typed, without};
use crate::{
    back::Place,
    front::{
        Message,
        keymap::Action,
        update::tests::{after, chord, output_rows::at},
    },
};

#[test]
fn o_and_capital_o_add_an_output_below_or_above_and_a_count_adds_that_many() {
    let below = typed(on(2), "o");
    assert_eq!(at(&below), 3);
    assert_eq!(
        names(&below)[..4],
        ["House L", "House R", "Output 1", "Sub"]
    );
    let above = after(on(2), [Message::Add(Place::Before, 1)]);
    assert_eq!(at(&above), 2);
    assert_eq!(names(&above)[..3], ["House L", "Output 1", "House R"]);
    // Last of those added is the one selected, as with cues
    let three = typed(on(2), "3o");
    assert_eq!(names(&three).len(), 13);
    assert_eq!(at(&three), 5);
    assert_eq!(names(&three)[2..5], ["Output 1", "Output 2", "Output 3"]);
    assert_eq!(three.workspace.cues().count(), 14, "no cue is added");
}

#[test]
fn a_count_of_added_outputs_stops_at_sixty_four() {
    let many = typed(on(1), "9999o");
    assert_eq!(many.workspace.outputs().len(), 10 + 64);
    // One change takes them all back
    let undone = typed(many, "u");
    assert_eq!(undone.workspace.outputs().len(), 10);
}

#[test]
fn dd_x_and_delete_cut_the_output_and_select_the_one_that_takes_its_place() {
    let cut = typed(on(3), "dd");
    assert_eq!(names(&cut), without(&[2]));
    assert_eq!(at(&cut), 3);
    assert_eq!(on_clipboard(&cut), Some(("Cut the output Sub", true)));
    assert_eq!(names(&typed(on(3), "x")), without(&[2]));
    // Last output leaves the one before it selected
    assert_eq!(at(&typed(on(10), "dd")), 9);
    // Delete is a cut key of the vi keymap
    let delete = chord(Key::Named(Named::Delete), Modifiers::empty());
    assert_eq!(names(&on(3).typed(delete, false).0), without(&[2]));
}

#[test]
fn counts_and_motions_cut_the_outputs_that_they_reach() {
    assert_eq!(names(&typed(on(2), "3dd")), without(&[1, 2, 3]));
    assert_eq!(names(&typed(on(2), "dj")), without(&[1, 2]));
    assert_eq!(names(&typed(on(4), "dk")), without(&[2, 3]));
    assert_eq!(names(&typed(on(3), "2dj")), without(&[2, 3, 4]));
    let cut = after(on(3), [Message::Across(Action::Cut, Action::Last, None)]);
    assert_eq!(names(&cut), ["House L", "House R"]);
    let first = after(
        on(3),
        [Message::Across(Action::Delete, Action::First, None)],
    );
    assert_eq!(names(&first), ALL[3..]);
    // One change takes the outputs back
    assert_eq!(names(&typed(typed(on(2), "3dd"), "u")), ALL);
}

#[test]
fn a_cut_that_would_take_every_output_does_nothing_and_says_so() {
    let refused = typed(on(1), "9999dd");
    assert_eq!(names(&refused), ALL);
    assert_eq!(
        refused.notices.remark,
        Some("A show keeps at least one output")
    );
    assert!(refused.notices.clipped.is_none());
    // Last output of a show that has one
    let one = after(on(1), [Message::Across(Action::Delete, Action::Last, None)]);
    assert_eq!(names(&one), ALL, "the cut reaches all of them");
    let single = typed(on(2), "dG");
    assert_eq!(names(&single), ["House L"]);
    let still = typed(single, "dd");
    assert_eq!(names(&still), ["House L"]);
    assert_eq!(
        still.notices.remark,
        Some("A show keeps at least one output")
    );
}

#[test]
fn show_mode_locks_the_keys_that_add_and_cut_and_fills_no_register() {
    for keys in ["o", "dd", "x", "dj", "3dd"] {
        let kept = typed(locked(on(3)), keys);
        assert_eq!((names(&kept), at(&kept)), (ALL.to_vec(), 3), "{keys}");
        assert!(kept.notices.clipped.is_none(), "{keys}");
    }
    let deleted = after(locked(on(3)), [Message::Delete(1)]);
    assert_eq!(names(&deleted), ALL, "Delete");
}
