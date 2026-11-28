// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the register of outputs, which the keys that copy and paste fill

use super::{ALL, locked, names, on, on_clipboard, typed};
use crate::{
    back::{CueId, Edit, OutputId, Place, Sends},
    front::{
        App, Message,
        update::{
            editing::{Clipboard, Register},
            tests::{after, history::cue, output_rows::at, sequence, show},
        },
    },
};

/// Returns the app with the cue 3 sending to the output Sub alone
fn sending_to_sub() -> App {
    let sends: Sends = [(OutputId(3), -4.0)].into_iter().collect();
    after(on(3), [Message::Edit(CueId(3), Edit::Sends(Some(sends)))])
}

#[test]
fn a_cut_output_pasted_back_takes_back_its_identity_as_dd_p_returns_a_line() {
    let cut = typed(sending_to_sub(), "dd");
    let cue_sends = |app: &App| {
        let cue = app.workspace.cue(CueId(3)).expect("the cue is there");
        app.workspace.sends(cue)
    };
    assert!(cue_sends(&cut).is_empty());
    // Pasting below the output that took its place puts it after that one
    let back = typed(typed(cut, "k"), "p");
    assert_eq!(names(&back), ALL);
    assert_eq!(at(&back), 3);
    assert_eq!(
        cue_sends(&back).into_iter().collect::<Vec<_>>(),
        [(OutputId(3), -4.0)]
    );
    // Pasting above puts it before the one that took its place
    let above = after(
        typed(sending_to_sub(), "dd"),
        [Message::Paste(Place::Before, 1)],
    );
    assert_eq!(names(&above), ALL);
}

#[test]
fn a_copy_pasted_is_an_output_of_its_own_with_a_free_name() {
    let copied = typed(sending_to_sub(), "yy");
    assert_eq!(
        on_clipboard(&copied),
        Some(("Copied the output Sub", false))
    );
    assert_eq!(names(&copied), ALL, "copying changes nothing");
    let pasted = typed(copied, "p");
    assert_eq!(names(&pasted)[2..5], ["Sub", "Sub 2", "Monitor"]);
    assert_eq!(at(&pasted), 4);
    let copy = &pasted.workspace.outputs()[3];
    assert_eq!(copy.new_cues, None);
    assert_ne!(copy.id, OutputId(3));
    // The copy has no sends, since no cue sends to it
    let cue = pasted.workspace.cue(CueId(3)).expect("the cue is there");
    let sends: Vec<_> = pasted.workspace.sends(cue).into_iter().collect();
    assert_eq!(sends, [(OutputId(3), -4.0)]);
    // Count pastes that many, each under a name of its own
    let twice = typed(typed(on(3), "yy"), "2p");
    assert_eq!(names(&twice)[2..6], ["Sub", "Sub 2", "Sub 3", "Monitor"]);
    // Above, with capital P
    let above = after(typed(on(3), "yy"), [Message::Paste(Place::Before, 1)]);
    assert_eq!(names(&above)[2..4], ["Sub 2", "Sub"]);
}

#[test]
fn a_copy_with_the_motions_and_capital_y_takes_the_outputs_that_they_reach() {
    let two = typed(on(2), "yj");
    assert_eq!(on_clipboard(&two), Some(("Copied 2 outputs", false)));
    let pasted = typed(two, "p");
    assert_eq!(
        names(&pasted)[1..5],
        ["House R", "House R 2", "Sub 2", "Sub"]
    );
    let line = after(on(2), [Message::Copy(1)]);
    assert_eq!(
        on_clipboard(&line),
        Some(("Copied the output House R", false))
    );
}

#[test]
fn copying_acts_in_show_mode_and_pasting_does_not() {
    let copied = typed(locked(on(3)), "yy");
    assert!(copied.notices.clipped.is_some());
    assert_eq!(names(&typed(copied, "p")), ALL);
}

#[test]
fn outputs_have_a_register_apart_from_cues() {
    // Cue copied in the cue list survives a copy of an output
    let cues = after(
        show(cue(2)),
        [Message::Copy(1), Message::ShowOutputs, Message::Copy(1)],
    );
    let back = after(
        cues,
        [Message::StepList(-1), Message::Paste(Place::After, 1)],
    );
    assert_eq!(back.workspace.cues().count(), 15, "the cue is pasted");
    // Nothing in the register of outputs pastes nothing, and tells so
    let empty = after(
        show(cue(2)),
        [
            Message::Copy(1),
            Message::ShowOutputs,
            Message::Paste(Place::After, 1),
        ],
    );
    assert_eq!(empty.workspace.outputs().len(), 1);
    assert_eq!(empty.notices.remark, Some("The clipboard is empty"));
}

#[test]
fn a_cut_output_pasted_in_another_show_is_a_copy_there() {
    let cut = typed(sending_to_sub(), "dd");
    let register = Register {
        show: cut.show.id().wrapping_add(1),
        cut: true,
        outputs: cut.clipboard.outputs.outputs.clone(),
    };
    let elsewhere = App {
        clipboard: Clipboard {
            outputs: register,
            ..cut.clipboard
        },
        ..cut
    };
    let pasted = typed(typed(elsewhere, "k"), "p");
    let sub = &pasted.workspace.outputs()[2];
    assert_eq!(sub.name, "Sub");
    // Identity that the other show gave it means nothing here
    assert_ne!(sub.id, OutputId(3));
    let cue = pasted.workspace.cue(CueId(3)).expect("the cue is there");
    assert!(pasted.workspace.sends(cue).is_empty());
}

/// Returns the state after the keys, which go through the dispatch that
/// drops the notices, where [`typed`] does not
fn pressed(app: App, keys: &str) -> App {
    let presses = sequence(keys).expect("keys in the tests are valid");
    after(
        app,
        presses.into_iter().map(|press| Message::Typed {
            press,
            repeat: false,
        }),
    )
}

#[test]
fn a_copy_notice_does_not_cross_between_the_cues_and_the_outputs() {
    // Outputs have a register of their own, so a notice of a copy in one no
    // longer fits the other
    let outputs = after(show(cue(2)), [Message::ShowOutputs, Message::Copy(1)]);
    assert!(outputs.notices.clipped.is_some());
    let cues = pressed(outputs, "h");
    assert!(!cues.outputs_shown);
    assert!(cues.notices.clipped.is_none());
    let copied = after(show(cue(2)), [Message::Copy(1)]);
    assert!(copied.notices.clipped.is_some());
    // Cue lists share a register, so the notice stays from one to the next
    let preshow = pressed(copied, "l");
    assert!(!preshow.outputs_shown);
    assert!(preshow.notices.clipped.is_some());
    let outputs = pressed(preshow, "l");
    assert!(outputs.outputs_shown);
    assert!(outputs.notices.clipped.is_none());
}
