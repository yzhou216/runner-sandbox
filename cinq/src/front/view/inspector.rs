// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Inspector, which shows every setting of the selected cue on three tabs

mod field;

use iced::{
    Center, Element, Fill,
    widget::{checkbox, column, progress_bar, radio, row, rule, slider, table, text, text_input},
};
use strum::VariantArray;

use super::{SPACING, char_width, format, keep_keys, theme};
use crate::{
    back::{Cue, Edit, Kind, LEVELS, Media, parse_level},
    front::{App, Message, Tab},
};
use field::Field;

/// Characters that the level field holds, as many as `-60.0` takes, with
/// one to spare
const LEVEL_ROOM: f32 = 6.0;

/// Fields of the Basics tab, one column after the other
const BASICS: [&[Field]; 2] = [
    &[Field::Number, Field::Name, Field::Target, Field::Notes],
    &[
        Field::PreWait,
        Field::Duration,
        Field::PostWait,
        Field::Then,
    ],
];

/// Fields of the Time & Loops tab, with the length of the clip they set
const TIME: [Field; 3] = [Field::Start, Field::End, Field::Duration];

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let tabs = column![text("Inspector")]
        .extend(
            Tab::VARIANTS
                .iter()
                .map(|&tab| radio(tab.to_string(), tab, Some(app.tab), Message::Tab).into()),
        )
        .spacing(SPACING);
    let body = match app.selected.and_then(|id| app.workspace.cue(id)) {
        Some(cue) => inspected(app, cue),
        None if app.current().cues.is_empty() => text("This list has no cues to inspect").into(),
        None => text("Select a cue to inspect it").into(),
    };
    keep_keys(
        row![tabs, rule::vertical(1), body]
            .spacing(SPACING)
            .padding(SPACING)
            .height(240),
        &app.config.keymap,
    )
}

fn inspected<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let fields = match app.tab {
        Tab::Basics => row(BASICS.map(|fields| form(app, cue, fields)))
            .spacing(SPACING)
            .into(),
        Tab::Time => time(app, cue),
        Tab::Levels => levels(app, cue),
    };
    let lock = if app.editable() {
        ""
    } else {
        " (read-only in show mode)"
    };
    column![text(format!("{cue}, {} cue{lock}", cue.kind)), fields]
        .spacing(SPACING)
        .width(Fill)
        .into()
}

fn form<'a>(app: &'a App, cue: &'a Cue, fields: &[Field]) -> Element<'a, Message> {
    table::table(
        [
            table::column(text("Setting"), move |field: Field| text(field.label(cue))),
            table::column(text("Value"), move |field: Field| field.input(app, cue)).width(Fill),
        ],
        fields.iter().copied(),
    )
    .into()
}

fn time<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    match cue.kind {
        Kind::Picture => {
            return text("Picture cues show their file until something stops them").into();
        }
        kind if !kind.plays_file() => return text(format!("{kind} cues play no file")).into(),
        _ => {}
    }
    let id = cue.id;
    let looping = checkbox(cue.looping)
        .label("Loop until stopped")
        .on_toggle_maybe(
            app.editable()
                .then_some(move |looping| Message::Edit(id, Edit::Loop(looping))),
        );
    column![form(app, cue, &TIME), looping]
        .spacing(SPACING)
        .into()
}

fn levels<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let label = match cue.kind {
        Kind::Video if matches!(cue.media, Media::Loaded { sound: None, .. }) => {
            return text("The file of this cue holds no sound").into();
        }
        Kind::Audio | Kind::Video => "Level",
        Kind::Fade => "Fade target to",
        Kind::Picture | Kind::Stop | Kind::Wait | Kind::Memo => {
            return text(format!("{} cues have no level", cue.kind)).into();
        }
    };
    let id = cue.id;
    let editable = app.editable();
    let typed = match &app.typed_level {
        Some((typed, text)) if *typed == id => text.clone(),
        _ => format::decibels(cue.level),
    };
    let field = text_input("", &typed)
        .on_input_maybe(editable.then_some(move |text| Message::LevelTyped(id, text)))
        .width(char_width() * LEVEL_ROOM + text_input::DEFAULT_PADDING.x())
        .style(theme::field(parse_level(&typed).is_some()));
    let level: Element<'_, Message> = if editable {
        slider(LEVELS, cue.level, move |db| {
            Message::Edit(id, Edit::Level(db))
        })
        .step(0.5_f32)
        .into()
    } else {
        progress_bar(LEVELS, cue.level).into()
    };
    let stop = (cue.kind == Kind::Fade).then(|| {
        checkbox(cue.stop_target)
            .label("Stop target when done")
            .on_toggle_maybe(
                editable.then_some(move |stop| Message::Edit(id, Edit::StopTarget(stop))),
            )
    });
    column![
        row![text(label), level, field, text("dB")]
            .spacing(SPACING)
            .align_y(Center),
        stop,
    ]
    .spacing(SPACING)
    .into()
}
