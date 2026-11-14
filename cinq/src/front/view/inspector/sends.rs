// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Levels at which an audio or video cue sends to each output of the show
//!
//! Each output has a field, which holds the level that the cue sends at,
//! or nothing where the cue does not send there.  Typing a level sends the
//! cue there, and emptying the field takes the send away.  The fields
//! scroll within the inspector, so that every output stays in reach, as
//! the 32 outputs of a Dante Virtual Soundcard need.  The name of an output
//! that cannot play now shows in yellow where cinq tries its device again
//! by itself, and in red where the output must change, and the header
//! tells how many cannot, so that the color is never the only sign.

use iced::{
    Center, Element, Fill,
    widget::{column, row, scrollable, text, text_input},
};

use super::LEVEL_ROOM;
use crate::{
    back::{Cue, Fault, parse_level},
    front::{
        App, Message, Setting,
        view::{SPACING, button, char_width, faults, format, theme},
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
        let name = text(&output.name).style(match faults::fault(app, output.id) {
            Some(fault) if fault.retried() => text::warning,
            Some(_) => text::danger,
            None => text::default,
        });
        column![
            name,
            row![field, text("dB")].spacing(SPACING).align_y(Center)
        ]
        .spacing(SPACING / 2.0)
        .into()
    });
    let silent = app
        .workspace
        .outputs()
        .iter()
        .filter_map(|output| faults::fault(app, output.id))
        .collect::<Vec<&Fault>>();
    let header = row![
        text("Outputs"),
        button("Edit outputs").on_press(Message::ShowOutputs)
    ]
    .push(
        text(match silent.len() {
            0 => String::new(),
            count => format!("{count} cannot play now"),
        })
        .style(if silent.iter().all(|fault| fault.retried()) {
            text::warning
        } else {
            text::danger
        }),
    )
    .spacing(SPACING)
    .align_y(Center);
    column![
        header,
        // Bar beside the fields, rather than over the last of each row
        scrollable(row(outputs).spacing(2.0 * SPACING).wrap())
            .spacing(SPACING)
            .height(Fill)
    ]
    .spacing(SPACING)
    .into()
}
