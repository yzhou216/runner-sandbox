// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the levels typed into fields, which apply once the operator
//! leaves the field, so that a cue that plays does not follow each key

use super::{after, show, typed_setting, with_sound};
use crate::{
    back::{Command, CueId, Edit, InstrumentId, Kind, OutputId, PatchEdit, Place},
    front::{App, Focus, Message, Setting},
};

const CUE: CueId = CueId(1);

/// Returns the interface with cue 1 loaded, at -30 dB, and selected
fn loud() -> App {
    let app = with_sound(show(Some(CUE)));
    let workspace = app.workspace.apply(Command::Edit(CUE, Edit::Level(-30.0)));
    App { workspace, ..app }
}

/// Returns the interface with cue 1 playing at -30 dB
fn playing() -> App {
    after(loud(), [Message::Go])
}

fn run_level(app: &App) -> Option<f32> {
    app.workspace.active_of(CUE).map(|active| active.level)
}

fn sends(app: &App) -> Option<f32> {
    let cue = app.workspace.cue(CUE)?;
    app.workspace.sends(cue).get(&OutputId(1)).copied()
}

#[test]
fn level_typed_into_a_playing_cue_applies_once_the_field_is_left() {
    let level = Setting::Level(CUE);
    let app = ["-", "-2", "-25"]
        .into_iter()
        .fold(playing(), |app, text| typed_setting(app, level, text));
    assert_eq!(run_level(&app), Some(-30.0), "no prefix moves the run");
    assert_eq!(app.typed, Some((level, "-25".to_owned())));
    let entered = after(app, [Message::Focus(Focus::Cues)]);
    assert_eq!(run_level(&entered), Some(-25.0), "Enter leaves the field");
    assert_eq!(entered.typed, None);
}

#[test]
fn level_typed_is_applied_by_anything_that_leaves_the_field_but_a_tick() {
    let level = Setting::Level(CUE);
    let leaving = [
        Message::Focus(Focus::Next),
        Message::Focus(Focus::Previous),
        Message::Clicked,
        Message::Step(1),
    ];
    leaving.into_iter().for_each(|message| {
        let app = after(typed_setting(playing(), level, "-25"), [message.clone()]);
        assert_eq!(run_level(&app), Some(-25.0), "{message:?}");
    });
    let ticked = after(
        typed_setting(playing(), level, "-25"),
        [Message::Tick(std::time::Instant::now())],
    );
    assert_eq!(
        run_level(&ticked),
        Some(-30.0),
        "a tick is not the operator"
    );
}

#[test]
fn go_fires_a_cue_at_the_level_typed_into_its_field() {
    let app = after(
        typed_setting(loud(), Setting::Level(CUE), "-25"),
        [Message::Go],
    );
    assert_eq!(run_level(&app), Some(-25.0));
}

#[test]
fn slider_moves_a_cue_at_once_and_text_left_in_the_field_applies_first() {
    let app = after(
        typed_setting(playing(), Setting::Level(CUE), "-25"),
        [Message::Edit(CUE, Edit::Level(-10.0))],
    );
    assert_eq!(run_level(&app), Some(-10.0));
    assert_eq!(app.typed, None);
}

#[test]
fn level_that_reads_as_no_level_leaves_the_cue_and_tells_why() {
    let level = Setting::Level(CUE);
    let app = typed_setting(playing(), level, "loud");
    assert_eq!(
        app.notices.unapplied, None,
        "the field is red while it is typed"
    );
    let left = after(app, [Message::Focus(Focus::Next)]);
    assert_eq!(run_level(&left), Some(-30.0));
    assert_eq!(
        left.notices.unapplied.as_deref(),
        Some("Not applied: levels run from -60 to +12 dB")
    );
    let emptied = after(
        typed_setting(playing(), level, ""),
        [Message::Focus(Focus::Next)],
    );
    assert_eq!(run_level(&emptied), Some(-30.0), "a cue always has a level");
    assert_eq!(
        emptied.notices.unapplied.as_deref(),
        Some("Not applied: type a level, -60 to +12 dB")
    );
}

#[test]
fn emptying_the_send_of_a_playing_cue_keeps_it_sending_until_the_field_is_left() {
    let send = Setting::Send(CUE, OutputId(1));
    let emptied = typed_setting(typed_setting(playing(), send, "-6"), send, "");
    let sending = sends(&emptied);
    assert!(sending.is_some() && sending != Some(-6.0), "{sending:?}");
    assert!(emptied.workspace.active_of(CUE).is_some());
    let left = after(emptied, [Message::Focus(Focus::Cues)]);
    assert_eq!(sends(&left), None, "leaving the field takes the send away");
}

#[test]
fn send_typed_applies_on_leaving_and_text_that_is_no_level_leaves_it_as_it_was() {
    let send = Setting::Send(CUE, OutputId(1));
    let before = sends(&playing());
    let typed_in = typed_setting(playing(), send, "-6");
    assert_eq!(sends(&typed_in), before);
    assert_eq!(sends(&after(typed_in, [Message::Step(1)])), Some(-6.0));
    let wrong = after(typed_setting(playing(), send, "loud"), [Message::Step(1)]);
    assert_eq!(sends(&wrong), before);
}

#[test]
fn off_typed_in_a_send_takes_it_away_and_minus_infinity_sends_silently() {
    let send = Setting::Send(CUE, OutputId(1));
    let off = after(typed_setting(playing(), send, "off"), [Message::Step(1)]);
    assert_eq!(sends(&off), None);
    let silent = after(typed_setting(playing(), send, "-inf"), [Message::Step(1)]);
    assert_eq!(sends(&silent), Some(-60.0));
}

#[test]
fn light_level_applies_once_its_field_is_left() {
    let id = InstrumentId(1);
    let app = show(Some(CUE));
    let workspace = app
        .workspace
        .apply(Command::Add {
            list: 0,
            kind: Kind::Light,
            at: Place::After(CUE),
            count: 1,
        })
        .apply(Command::Patch(PatchEdit::Add));
    let app = App { workspace, ..app };
    let cue = app.workspace.lists()[0].cues[1].id;
    let lights = |app: &App| {
        app.workspace
            .cue(cue)
            .and_then(|cue| cue.lights.get(&id).copied())
    };
    let app = typed_setting(app, Setting::Light(cue, id), "80");
    assert_eq!(lights(&app), None);
    let left = after(app, [Message::Focus(Focus::Cues)]);
    assert_eq!(lights(&left), Some(80.0));
}

#[test]
fn address_applies_once_its_field_is_left_and_a_refused_text_keeps_the_old_one() {
    let id = InstrumentId(1);
    let app = App {
        workspace: show(None).workspace.apply(Command::Patch(PatchEdit::Add)),
        ..show(None)
    };
    let address = |app: &App| {
        app.workspace
            .patch()
            .iter()
            .find(|instrument| instrument.id == id)
            .map(|instrument| instrument.address_text())
    };
    let before = address(&app);
    assert_eq!(before.as_deref(), Some("1"));
    let typing = ["2", "2/", "2/5", "2/50", "2/500"]
        .into_iter()
        .fold(app, |app, text| {
            typed_setting(app, Setting::Address(id), text)
        });
    assert_eq!(address(&typing), before, "no prefix moves the dimmer");
    let left = after(typing, [Message::Focus(Focus::Cues)]);
    assert_eq!(address(&left).as_deref(), Some("2/500"));
    let refused = after(
        typed_setting(left, Setting::Address(id), "2/5000"),
        [Message::Focus(Focus::Cues)],
    );
    assert_eq!(address(&refused).as_deref(), Some("2/500"), "the old stays");
}

#[test]
fn adding_an_instrument_past_the_last_address_tells_why() {
    let id = InstrumentId(1);
    let one = || App {
        workspace: show(None).workspace.apply(Command::Patch(PatchEdit::Add)),
        ..show(None)
    };
    let added = after(one(), [Message::Patch(PatchEdit::Add)]);
    assert_eq!(added.workspace.patch().len(), 2);
    assert_eq!(added.notices.remark, None);
    let full = App {
        workspace: one()
            .workspace
            .apply(Command::Patch(PatchEdit::Address(id, 32_768, 512))),
        ..one()
    };
    let refused = after(full, [Message::Patch(PatchEdit::Add)]);
    assert_eq!(refused.workspace.patch().len(), 1);
    assert_eq!(
        refused.notices.remark,
        Some("The light patch has no address left for another instrument")
    );
}
