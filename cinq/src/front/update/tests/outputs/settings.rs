// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the settings of outputs typed into fields

use std::time::Instant;

use super::super::{after, output_editing::names, show, typed_setting};
use crate::{
    back::{CueId, OutputId, Place},
    front::{App, Focus, Message, Setting},
};

/// Returns the interface with the outputs shown, Main selected
fn outputs_shown() -> App {
    after(show(None), [Message::ShowOutputs])
}

/// Returns the interface once the operator has left the field, as Enter
/// does by giving the keyboard to the cue list
fn left(app: App) -> App {
    after(app, [Message::Focus(Focus::Cues)])
}

#[test]
fn typed_names_apply_once_they_read_and_show_as_typed_meanwhile() {
    let main = OutputId(1);
    let app = typed_setting(outputs_shown(), Setting::OutputName(main), "House");
    assert_eq!(app.workspace.outputs()[0].name, "House");
    assert_eq!(
        app.typed,
        Some((Setting::OutputName(main), "House".to_owned()))
    );
    let app = typed_setting(app, Setting::OutputName(main), "");
    assert_eq!(app.workspace.outputs()[0].name, "House", "no name reads");
}

#[test]
fn typed_new_cues_level_applies_once_the_field_is_left() {
    let main = OutputId(1);
    let app = typed_setting(outputs_shown(), Setting::NewCues(main), "-");
    assert_eq!(app.workspace.outputs()[0].new_cues, Some(0.0));
    assert_eq!(app.typed, Some((Setting::NewCues(main), "-".to_owned())));
    let app = typed_setting(app, Setting::NewCues(main), "-6");
    assert_eq!(app.workspace.outputs()[0].new_cues, Some(0.0));
    let app = left(app);
    assert_eq!(app.workspace.outputs()[0].new_cues, Some(-6.0));
    let app = left(typed_setting(app, Setting::NewCues(main), ""));
    assert_eq!(app.workspace.outputs()[0].new_cues, None);
}

fn channels(app: &App) -> String {
    app.workspace.outputs()[0].channels.to_string()
}

#[test]
fn channels_apply_at_enter_not_at_each_prefix_that_reads() {
    let main = OutputId(1);
    // Each of 1, 12, and 12-13 reads, and 12 would ask a device for 12
    // channels on the way to the pair
    let app = ["1", "12", "12-", "12-13"]
        .into_iter()
        .fold(outputs_shown(), |app, text| {
            typed_setting(app, Setting::Channels(main), text)
        });
    assert_eq!(channels(&app), "1-2");
    assert_eq!(
        app.typed,
        Some((Setting::Channels(main), "12-13".to_owned()))
    );
    // Enter in the field gives the keyboard to the cue list, as it does in
    // every other text field, and that applies the channels
    let app = app.next(Message::Focus(Focus::Cues)).0;
    assert_eq!(channels(&app), "12-13");
    assert_eq!(app.typed, None);
}

#[test]
fn channels_apply_when_the_operator_goes_on_to_anything_else() {
    let main = OutputId(1);
    let typing = |text: &str| typed_setting(outputs_shown(), Setting::Channels(main), text);
    let next = |app: App, message| app.next(message).0;
    // Another field, a key outside any field, a click, or any other message
    let field = typed_setting(typing("3-4"), Setting::OutputName(main), "House");
    assert_eq!(channels(&field), "3-4");
    let elsewhere = next(typing("3-4"), Message::Step(1));
    assert_eq!(channels(&elsewhere), "3-4");
    assert_eq!(elsewhere.typed, None);
    let clicked = next(typing("3-4"), Message::Clicked);
    assert_eq!(channels(&clicked), "3-4", "a click anywhere settles them");
    assert_eq!(clicked.typed, None, "so that no key after it applies them");
    let tick = next(typing("3-4"), Message::Tick(Instant::now()));
    assert_eq!(channels(&tick), "1-2", "a tick is not the operator");
    assert_eq!(channels(&next(typing("3-"), Message::Step(1))), "1-2");
}

#[test]
fn typed_sends_apply_and_an_empty_field_takes_the_send_away() {
    let main = OutputId(1);
    let cue = CueId(1);
    let sends = |app: &App| {
        let cue = app.workspace.cue(cue).expect("the cue is there");
        app.workspace.sends(cue).get(&main).copied()
    };
    let app = left(typed_setting(show(None), Setting::Send(cue, main), "-6"));
    assert_eq!(sends(&app), Some(-6.0));
    let app = left(typed_setting(app, Setting::Send(cue, main), ""));
    assert_eq!(sends(&app), None);
}

#[test]
fn editing_a_send_keeps_the_sends_to_outputs_that_have_gone() {
    let (main, sub, cue) = (OutputId(1), OutputId(2), CueId(1));
    let app = after(
        show(None),
        [Message::ShowOutputs, Message::Add(Place::After, 1)],
    );
    let app = typed_setting(app, Setting::Send(cue, sub), "-3");
    let app = after(app, [Message::Delete(1)]);
    assert_eq!(names(&app), ["Main"]);
    let app = left(typed_setting(app, Setting::Send(cue, main), "-6"));
    let kept = app.workspace.cue(cue).and_then(|cue| cue.sends.clone());
    assert_eq!(kept.and_then(|sends| sends.get(&sub).copied()), Some(-3.0));
}

#[test]
fn enter_in_a_field_that_reads_no_value_leaves_it_and_tells_why() {
    let main = OutputId(1);
    let app = typed_setting(outputs_shown(), Setting::Channels(main), "x");
    let app = app.next(Message::Focus(Focus::Cues)).0;
    assert_eq!(channels(&app), "1-2");
    assert_eq!(app.typed, None);
    assert!(app.notices.unapplied.is_some());
}
