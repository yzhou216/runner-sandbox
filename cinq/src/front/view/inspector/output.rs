// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Settings of the output that the outputs view has selected
//!
//! A name that another output has, channels that are neither one nor two,
//! and a level out of range stay in their fields, marked, without applying,
//! as a level of a cue does.

use iced::{
    Center, Element,
    widget::{column, row, text, text_input},
};

use super::LEVEL_ROOM;
use crate::{
    back::{Channels, Device, Fault, Output, parse_level},
    front::{
        App, Message, Setting,
        view::{
            OUTPUT_NAME, SPACING, char_width,
            devices::{Choice, choices},
            dropdown, faults, format, keep_keys, outputs, theme,
        },
    },
};

/// Columns of the font that the labels of the settings take, as many as
/// the longest of them
const LABEL: f32 = 14.0;

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let Some(output) = app.output.and_then(|id| app.workspace.output(id)) else {
        return text("Select an output to inspect it").into();
    };
    let id = output.id;
    let editable = app.editable();
    let typed = |setting: Setting, value: String| match &app.typed {
        Some((typed, text)) if *typed == setting => text.clone(),
        _ => value,
    };
    let field = move |setting: Setting,
                      placeholder: &str,
                      value: String,
                      valid: fn(&App, &Output, &str) -> bool| {
        let shown = typed(setting, value);
        let fits = valid(app, output, &shown);
        text_input(placeholder, &shown)
            .on_input_maybe(editable.then_some(move |text| Message::SettingTyped(setting, text)))
            .style(theme::field(fits))
    };
    let name = field(
        Setting::OutputName(id),
        "Name",
        output.name.clone(),
        |app, output, name| {
            !name.trim().is_empty()
                && app
                    .workspace
                    .outputs()
                    .iter()
                    .all(|other| other.id == output.id || other.name != name)
        },
    )
    .id(OUTPUT_NAME);
    let channels = field(
        Setting::Channels(id),
        "1-2",
        output.channels.to_string(),
        |_, _, channels| Channels::parse(channels).is_some(),
    );
    let new_cues = field(
        Setting::NewCues(id),
        "off",
        output.new_cues.map(format::decibels).unwrap_or_default(),
        |_, _, level| level.trim().is_empty() || parse_level(level).is_some(),
    )
    .width(char_width() * LEVEL_ROOM + text_input::DEFAULT_PADDING.x());
    let sending = app
        .workspace
        .cues()
        .filter(|cue| cue.kind.plays_file() && app.workspace.sends(cue).contains_key(&id))
        .count();
    let settings = [
        ("Name", name.into()),
        ("Device", device(app, output)),
        ("Channels", channels.into()),
        (
            "New cues send",
            row![new_cues, text("dB")]
                .spacing(SPACING)
                .align_y(Center)
                .into(),
        ),
    ];
    let lock = if editable {
        ""
    } else {
        " (read-only in show mode)"
    };
    column![text(format!("{}, Output{lock}", output.name))]
        .extend(
            settings
                .into_iter()
                .map(|(label, input): (&str, Element<'_, Message>)| {
                    row![text(label).width(char_width() * LABEL), input]
                        .spacing(SPACING)
                        .align_y(Center)
                        .into()
                }),
        )
        .push(text(match sending {
            1 => "1 cue sends here".to_owned(),
            count => format!("{count} cues send here"),
        }))
        .spacing(SPACING)
        .into()
}

/// Returns the list of the devices that the output can play on, which
/// shows a device that is not there by its name
fn device<'a>(app: &'a App, output: &'a Output) -> Element<'a, Message> {
    let id = output.id;
    // Device that is not there, which reads so in words and in yellow
    // rather than in the gray of a list where nothing is chosen
    let missing = matches!(faults::fault(app, id), Some(Fault::Missing(_)));
    let name = outputs::device(&output.device);
    let placeholder = if missing {
        format!("{name}, not found")
    } else {
        name.to_owned()
    };
    let options = choices(&app.audio_devices, outputs::device(&Device::Default));
    let selected = options
        .iter()
        .find(|choice| match &output.device {
            Device::Default => choice.id.is_none(),
            Device::Named { id, .. } => choice.id.as_ref() == Some(id),
        })
        .cloned()
        .filter(|_| !missing);
    let keymap = &app.config.keymap;
    let list = dropdown(
        options,
        selected,
        move |choice: Choice| {
            let device = match choice.id {
                None => Device::Default,
                Some(device) => Device::Named {
                    id: device,
                    name: choice.name,
                },
            };
            Message::OutputDevice(id, device)
        },
        keymap,
    )
    .placeholder(placeholder);
    let list = if missing {
        list.placeholder_color(theme::PALETTE.warning)
    } else {
        list
    }
    .on_open(Message::ListAudioDevices);
    keep_keys(list, keymap)
}
