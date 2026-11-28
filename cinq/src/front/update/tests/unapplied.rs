// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the reason that the status bar tells when the keyboard leaves a
//! field that holds a value that was not applied

use super::{after, show, typed_setting};
use crate::{
    back::{OutputId, Place},
    front::{App, Focus, Message, Setting},
};

const MAIN: OutputId = OutputId(1);

/// Returns the interface with the outputs shown, Main and Output 1, and
/// `text` typed into `setting`
fn typed(setting: Setting, text: &str) -> App {
    let app = after(
        show(None),
        [Message::ShowOutputs, Message::Add(Place::After, 1)],
    );
    typed_setting(app, setting, text)
}

fn leaving(app: App, message: Message) -> Option<String> {
    app.next(message).0.notices.unapplied
}

#[test]
fn leaving_a_field_with_a_value_that_was_rejected_tells_the_reason() {
    let name = || typed(Setting::OutputName(MAIN), "Output 1");
    assert_eq!(name().notices.unapplied, None, "the field still says why");
    assert_eq!(
        leaving(name(), Message::Focus(Focus::Next)).as_deref(),
        Some("Not applied: another output has this name")
    );
    assert_eq!(
        leaving(name(), Message::Step(1)).as_deref(),
        Some("Not applied: another output has this name"),
        "any other way out tells it too"
    );
    let channels = typed(Setting::Channels(MAIN), "1-3");
    assert_eq!(
        leaving(channels, Message::Focus(Focus::Previous)).as_deref(),
        Some("Not applied: channels are 3 for one, or 1-2 or 1,3 for two, up to 256")
    );
    let level = typed(Setting::NewCues(MAIN), "loud");
    assert_eq!(
        leaving(level, Message::Focus(Focus::Next)).as_deref(),
        Some("Not applied: levels run from -60 to +12 dB, or empty for none")
    );
}

#[test]
fn a_value_that_applied_or_a_click_or_typing_in_the_field_tells_nothing() {
    let fine = typed(Setting::OutputName(MAIN), "Front");
    assert_eq!(leaving(fine, Message::Focus(Focus::Next)), None);
    let bad = || typed(Setting::OutputName(MAIN), "Output 1");
    assert_eq!(
        leaving(bad(), Message::Clicked),
        None,
        "a click can land in it"
    );
    let more = Message::SettingTyped(Setting::OutputName(MAIN), "Output 12".to_owned());
    assert_eq!(leaving(bad(), more), None);
}

#[test]
fn typing_in_another_field_leaves_the_first_and_tells_why_it_did_not_apply() {
    let moved = Message::SettingTyped(Setting::NewCues(MAIN), "-6".to_owned());
    assert_eq!(
        leaving(typed(Setting::Channels(MAIN), "1-3"), moved).as_deref(),
        Some("Not applied: channels are 3 for one, or 1-2 or 1,3 for two, up to 256")
    );
}

#[test]
fn the_reason_goes_with_the_next_thing_that_the_operator_does() {
    let name = typed(Setting::OutputName(MAIN), "Output 1");
    let told = name.next(Message::Focus(Focus::Next)).0;
    assert!(told.notices.unapplied.is_some());
    let moved = told.next(Message::Step(1)).0;
    assert_eq!(
        moved.notices.unapplied, None,
        "the next thing that the operator does"
    );
}
