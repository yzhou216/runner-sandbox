// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Levels at which an audio or video cue sends to each output of the show
//!
//! Each output has a field, which holds the level that the cue sends at,
//! or nothing where the cue does not send there.  Typing a level sends the
//! cue there, and emptying the field takes the send away.

use iced::{
    Center, Element,
    widget::{column, row, text, text_input},
};

use super::LEVEL_ROOM;
use crate::{
    back::{Cue, parse_level},
    front::{
        App, Message, Setting,
        view::{SPACING, button, char_width, format, theme},
    },
};

pub(super) fn view<'a>(app: &'a App, cue: &'a Cue) -> Element<'a, Message> {
    let sends = app.workspace.sends(cue);
    let editable = app.editable();
    let outputs = app.workspace.outputs().iter().map(|output| {
        let setting = Setting::Send(cue.id, output.id);
        let shown = match &app.typed {
            Some((typed, text)) if *typed == setting => text.clone(),
            _ => sends
                .get(&output.id)
                .map(|level| format::decibels(*level))
                .unwrap_or_default(),
        };
        let fits = shown.trim().is_empty() || parse_level(&shown).is_some();
        let field = text_input("off", &shown)
            .on_input_maybe(editable.then_some(move |text| Message::SettingTyped(setting, text)))
            .width(char_width() * LEVEL_ROOM + text_input::DEFAULT_PADDING.x())
            .style(theme::field(fits));
        column![
            text(&output.name),
            row![field, text("dB")].spacing(SPACING).align_y(Center)
        ]
        .spacing(SPACING / 2.0)
        .into()
    });
    column![
        row![
            text("Outputs"),
            button("Edit outputs").on_press(Message::ShowOutputs)
        ]
        .spacing(SPACING)
        .align_y(Center),
        row(outputs).spacing(2.0 * SPACING).wrap()
    ]
    .spacing(SPACING)
    .into()
}
