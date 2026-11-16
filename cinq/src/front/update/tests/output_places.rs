// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the outputs view in the places that going back and forward walk
//! along

use iced::keyboard::{Key, Modifiers};

use super::{
    chord,
    history::{after, cue, place},
    show,
};
use crate::{
    back::{OutputId, Place},
    front::{App, Message, Tab},
};

fn output(id: u32) -> Option<OutputId> {
    Some(OutputId(id))
}

/// Returns the show with the outputs shown from a selected cue, after it
/// gained two more outputs, the third of them selected
fn with_outputs() -> App {
    after(
        show(cue(5)),
        [
            Message::ShowOutputs,
            Message::Add(Place::After, 1),
            Message::Add(Place::After, 1),
        ],
    )
}

#[test]
fn showing_the_outputs_is_a_jump_that_back_returns_from_to_the_cue_and_its_tab() {
    let before = after(show(cue(5)), [Message::Tab(Tab::Levels)]);
    let shown = after(before, [Message::ShowOutputs]);
    assert!(shown.outputs_shown);
    let back = after(shown, [Message::Back(1)]);
    assert!(!back.outputs_shown);
    assert_eq!((place(&back), back.tab), ((0, cue(5)), Tab::Levels));
    let forward = after(back, [Message::Forward(1)]);
    assert!(forward.outputs_shown);
    assert_eq!(forward.output, output(1));
}

#[test]
fn the_key_of_back_leaves_the_outputs_for_the_cue_list() {
    let shown = after(show(cue(5)), [Message::StepList(1), Message::StepList(1)]);
    assert!(shown.outputs_shown);
    let key = |app: App, key: &str| {
        app.typed(chord(Key::Character(key.into()), Modifiers::CTRL), false)
            .0
    };
    // The second list was the place that the jump to the outputs left
    let back = key(shown, "o");
    assert_eq!((back.outputs_shown, place(&back)), (false, (1, cue(20))));
    let again = key(key(back, "o"), "i");
    assert_eq!(place(&again), (1, cue(20)));
    assert!(!again.outputs_shown);
    assert!(key(again, "i").outputs_shown);
}

#[test]
fn back_from_the_outputs_with_nothing_recorded_leaves_for_the_cue_list_as_left() {
    let shown = App {
        outputs_shown: true,
        output: output(1),
        selected: cue(7),
        list: 1,
        ..show(None)
    };
    assert!(shown.can_go_back());
    let back = after(shown, [Message::Back(1)]);
    assert_eq!((back.outputs_shown, place(&back)), (false, (1, cue(7))));
    assert!(!back.can_go_back());
    let forward = after(back, [Message::Forward(1)]);
    assert!(forward.outputs_shown);
    assert_eq!(forward.output, output(1));
}

#[test]
fn back_has_nowhere_to_go_from_a_cue_list_with_nothing_recorded() {
    let app = show(cue(5));
    assert!(!app.can_go_back());
    assert_eq!(place(&after(app, [Message::Back(1)])), (0, cue(5)));
}

#[test]
fn a_warning_button_and_a_jump_to_a_row_of_outputs_are_jumps() {
    let inspected = after(show(cue(5)), [Message::InspectOutput(OutputId(1))]);
    assert!(inspected.outputs_shown);
    assert_eq!(place(&after(inspected, [Message::Back(1)])), (0, cue(5)));
    let jumped = after(with_outputs(), [Message::Row(1)]);
    assert_eq!(jumped.output, output(1));
    let back = after(jumped, [Message::Back(1)]);
    assert!(back.outputs_shown);
    assert_eq!(back.output, output(3));
}

#[test]
fn stepping_through_the_outputs_is_no_jump() {
    let stepped = after(with_outputs(), [Message::Step(-1), Message::Step(-1)]);
    assert_eq!(stepped.output, output(1));
    let back = after(stepped, [Message::Back(1)]);
    assert!(!back.outputs_shown);
    assert_eq!(place(&back), (0, cue(5)));
}

#[test]
fn leaving_the_outputs_for_a_cue_list_remembers_the_output_that_was_selected() {
    let left = after(with_outputs(), [Message::Step(-1), Message::SwitchList(1)]);
    assert!(!left.outputs_shown);
    let back = after(left, [Message::Back(1)]);
    assert!(back.outputs_shown);
    assert_eq!(back.output, output(2));
}

#[test]
fn a_place_on_an_output_that_was_deleted_is_skipped() {
    let jumped = after(
        with_outputs(),
        [
            Message::Row(2),
            Message::Row(3),
            Message::Step(-1),
            Message::Delete(1),
        ],
    );
    assert_eq!(jumped.output, output(3));
    // Outputs 3 and 2 were left behind, and the second one is gone, so
    // that only the way out of the outputs is left
    let back = after(jumped, [Message::Back(1)]);
    assert!(!back.outputs_shown);
    assert_eq!(place(&back), (0, cue(5)));
}
