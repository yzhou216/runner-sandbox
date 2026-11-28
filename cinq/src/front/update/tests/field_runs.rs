// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the edits of one field that undo takes back one visit at a time:
//! keys typed in a row make one change, until the keyboard leaves the field or
//! enters it again

use super::{after, history::cue, output_editing::on, show, typed_into, typing};
use crate::{
    back::{CueId, Edit},
    front::{App, Caret, Focus, Message, Setting, app::Held},
};

/// Types `text` into the name of the selected output, one key at a time, with
/// the reports of the widgets that the keyboard stays in the field
fn into_output(app: App, text: &str) -> App {
    let id = app.output.expect("an output is selected");
    typed_into(app, text, |prefix| {
        Message::SettingTyped(Setting::OutputName(id), prefix)
    })
}

/// Types `text` into the name of cue 3, one key at a time, with the reports of
/// the widgets that the keyboard stays in the field
fn into_cue(app: App, text: &str) -> App {
    typed_into(app, text, |prefix| {
        Message::Edit(CueId(3), Edit::Name(prefix))
    })
}

fn cue_name(app: &App) -> Option<&str> {
    app.workspace.cue(CueId(3)).map(|cue| cue.name.as_str())
}

fn output_name(app: &App) -> &str {
    &app.workspace.outputs()[2].name
}

/// Messages that tell that the keyboard left the field, or came into it
fn between() -> [Message; 4] {
    [
        Message::Held(Held::default()),
        Message::Focus(Focus::Name(Caret::End)),
        Message::Focus(Focus::Next),
        Message::Clicked,
    ]
}

#[test]
fn two_visits_to_the_name_of_an_output_undo_as_two_steps() {
    between().into_iter().for_each(|leaving| {
        let first = into_output(on(3), "Sub-one");
        let second = into_output(after(first, [leaving.clone()]), "Sub-two");
        let once = after(second, [Message::Undo(1)]);
        assert_eq!(output_name(&once), "Sub-one", "{leaving:?}");
        assert_eq!(output_name(&after(once, [Message::Undo(1)])), "Sub");
    });
}

#[test]
fn two_visits_to_the_name_of_a_cue_undo_as_two_steps() {
    let first = into_cue(show(cue(3)), "Rain");
    let second = into_cue(
        after(first, [Message::Held(Held::default())]),
        "Rain on the roof",
    );
    let once = after(second, [Message::Undo(1)]);
    assert_eq!(cue_name(&once), Some("Rain"));
    let twice = after(once, [Message::Undo(1)]);
    assert_eq!(cue_name(&twice), Some("Sound"));
}

#[test]
fn two_visits_to_the_name_of_a_cue_by_tab_undo_as_two_steps() {
    let first = into_cue(show(cue(3)), "Rain");
    let away = typing(typing(first, "Tab"), "Shift+Tab");
    let second = into_cue(away, "Rain on the roof");
    let once = after(second, [Message::Undo(1)]);
    assert_eq!(cue_name(&once), Some("Rain"));
}
