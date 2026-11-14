// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the outputs view: reaching it past the last cue list, the keys
//! of cues acting on outputs there, and the settings typed into fields

use super::show;
use crate::{
    back::{CueId, OutputId, Place},
    front::{App, Message, Setting},
};

fn names(app: &App) -> Vec<&str> {
    app.workspace
        .outputs()
        .iter()
        .map(|output| output.name.as_str())
        .collect()
}

fn after(app: App, messages: impl IntoIterator<Item = Message>) -> App {
    messages
        .into_iter()
        .fold(app, |app, message| app.handled(message).0)
}

#[test]
fn outputs_lie_past_the_last_cue_list_and_back() {
    let shown = after(
        show(Some(CueId(1))),
        [Message::StepList(1), Message::StepList(1)],
    );
    assert!(shown.outputs_shown);
    assert_eq!(shown.output, Some(OutputId(1)));
    let past = after(shown, [Message::StepList(1)]);
    assert!(past.outputs_shown, "nothing lies past the outputs");
    let back = after(past, [Message::StepList(-1)]);
    assert!(!back.outputs_shown);
    assert_eq!(back.list, 1);
}

#[test]
fn keys_of_cues_add_move_and_delete_outputs_while_they_show() {
    let added = after(
        show(None),
        [
            Message::ShowOutputs,
            Message::Add(Place::After),
            Message::Add(Place::Before),
        ],
    );
    assert_eq!(names(&added), ["Main", "Output 2", "Output 1"]);
    assert_eq!(added.output, Some(OutputId(3)), "the new output shows");
    assert_eq!(added.workspace.cues().count(), 14, "no cue is added");
    let moved = after(added, [Message::Move(-1)]);
    assert_eq!(names(&moved), ["Output 2", "Main", "Output 1"]);
    let deleted = after(moved, [Message::Delete(1)]);
    assert_eq!(names(&deleted), ["Main", "Output 1"]);
    assert_eq!(deleted.output, Some(OutputId(1)), "the next output shows");
}

fn typed(app: App, setting: Setting, text: &str) -> App {
    after(app, [Message::SettingTyped(setting, text.to_owned())])
}

#[test]
fn typed_settings_apply_once_they_read_and_show_as_typed_meanwhile() {
    let main = OutputId(1);
    let app = after(show(None), [Message::ShowOutputs]);
    let app = typed(app, Setting::Channels(main), "3-");
    assert_eq!(app.workspace.outputs()[0].channels.to_string(), "1-2");
    assert_eq!(app.typed, Some((Setting::Channels(main), "3-".to_owned())));
    let app = typed(app, Setting::Channels(main), "3-4");
    assert_eq!(app.workspace.outputs()[0].channels.to_string(), "3-4");
    let app = typed(app, Setting::NewCues(main), "");
    assert_eq!(app.workspace.outputs()[0].new_cues, None);
}

#[test]
fn typed_sends_apply_and_an_empty_field_takes_the_send_away() {
    let main = OutputId(1);
    let cue = CueId(1);
    let sends = |app: &App| {
        let cue = app.workspace.cue(cue).expect("the cue is there");
        app.workspace.sends(cue).get(&main).copied()
    };
    let app = typed(show(None), Setting::Send(cue, main), "-6");
    assert_eq!(sends(&app), Some(-6.0));
    let app = typed(app, Setting::Send(cue, main), "");
    assert_eq!(sends(&app), None);
}

#[test]
fn warning_about_an_output_shows_the_outputs_with_it_selected() {
    let added = after(
        show(None),
        [Message::ShowOutputs, Message::Add(Place::After)],
    );
    let back = after(added, [Message::StepList(-1)]);
    assert!(!back.outputs_shown);
    let inspected = after(back, [Message::InspectOutput(OutputId(2))]);
    assert!(inspected.outputs_shown);
    assert_eq!(inspected.output, Some(OutputId(2)));
}
