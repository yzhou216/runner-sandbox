// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the edits of one field that undo takes back one visit at a time:
//! keys typed in a row make one change, until the keyboard leaves the field or
//! enters it again

use super::{
    history::{after, cue},
    output_editing::on,
    show,
};
use crate::{
    back::{CueId, Edit},
    front::{App, Caret, Focus, Message, Setting},
};

/// Types `text` into the name of the selected output, one key at a time, with
/// the reports of the widgets that the keyboard stays in the field
fn into_output(app: App, text: &str) -> App {
    let id = app.output.expect("an output is selected");
    (1..=text.len()).fold(app, |app, end| {
        let typed = Setting::OutputName(id);
        after(
            app,
            [
                Message::SettingTyped(typed, text[..end].to_owned()),
                Message::Held(true),
            ],
        )
    })
}

fn output_name(app: &App) -> &str {
    &app.workspace.outputs()[2].name
}

/// Messages that tell that the keyboard left the field, or came into it
fn between() -> [Vec<Message>; 4] {
    [
        vec![Message::Held(false)],
        vec![Message::Focus(Focus::Name(Caret::End))],
        vec![Message::Focus(Focus::Next)],
        vec![Message::Clicked],
    ]
}

#[test]
fn two_visits_to_the_name_of_an_output_undo_as_two_steps() {
    between().into_iter().for_each(|leaving| {
        let first = into_output(on(3), "Sub-one");
        let second = into_output(after(first, leaving.clone()), "Sub-two");
        let once = after(second, [Message::Undo(1)]);
        assert_eq!(output_name(&once), "Sub-one", "{leaving:?}");
        assert_eq!(output_name(&after(once, [Message::Undo(1)])), "Sub");
    });
}

#[test]
fn keys_typed_in_one_visit_undo_as_one_step_whatever_the_widgets_report() {
    let typed = into_output(on(3), "Sub-one");
    assert_eq!(output_name(&typed), "Sub-one");
    assert_eq!(output_name(&after(typed, [Message::Undo(1)])), "Sub");
}

#[test]
fn two_visits_to_the_name_of_a_cue_undo_as_two_steps() {
    let name = |app: &App| app.workspace.cue(CueId(3)).map(|cue| cue.name.clone());
    let type_into = |app: App, text: &str| {
        (1..=text.len()).fold(app, |app, end| {
            let edit = Edit::Name(text[..end].to_owned());
            after(app, [Message::Edit(CueId(3), edit), Message::Held(true)])
        })
    };
    let first = type_into(show(cue(3)), "Rain");
    let second = type_into(after(first, [Message::Held(false)]), "Rain on the roof");
    let once = after(second, [Message::Undo(1)]);
    assert_eq!(name(&once).as_deref(), Some("Rain"));
    let twice = after(once, [Message::Undo(1)]);
    assert_eq!(name(&twice).as_deref(), Some("Sound"));
}
