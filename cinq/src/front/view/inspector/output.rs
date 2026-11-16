// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Settings of the output that the outputs view has selected
//!
//! A name that another output has, channels that are neither one nor two,
//! and a level out of range stay in their fields, marked, without applying,
//! as a level of a cue does.  Each field is as wide as what it holds, and
//! the line beside it, in dim text, tells what the setting takes and what it
//! does.  That line turns red and tells why where the value is not applied,
//! so that the color is never the only sign.

mod device;
mod reasons;
pub(in crate::front) use reasons::rejection;
#[cfg(test)]
mod tests;

use iced::{
    Center, Element, Fill, Padding,
    widget::{column, container, row, text, text_input},
};

use super::LEVEL_ROOM;
use crate::{
    back::Output,
    front::{
        App, Message, Setting,
        view::{OUTPUT_NAME, SPACING, char_width, dropdown_width, format, theme},
    },
};

/// Columns of the font that the labels of the settings take, as many as
/// the longest of them
const LABEL: f32 = 14.0;

/// Columns of the font that the name field holds, with room for a long
/// name of a speaker
const NAME_ROOM: f32 = 24.0;

/// Columns of the font that the channels field holds, as many as `256,256`
/// takes, with one to spare
const CHANNELS_ROOM: f32 = 8.0;

/// Columns of the font that the device list shows of a name, as many as
/// `Dante Virtual Soundcard, not found` takes, so that most device names
/// show whole and a longer one ends in an ellipsis
const DEVICE_ROOM: f32 = 34.0;

/// Returns the width of a field that holds `columns` columns of text
fn room(columns: f32) -> f32 {
    char_width() * columns + text_input::DEFAULT_PADDING.x()
}

/// Returns the line of help beside a setting: `hint`, in dim text, or the
/// reason that the value is not applied, in red, where there is one
fn help<'a>(hint: impl Into<String>, rejection: Option<String>) -> Element<'a, Message> {
    match rejection {
        Some(reason) => text(format!("Not applied: {reason}"))
            .style(text::danger)
            .width(Fill)
            .into(),
        None => text(hint.into()).style(text::secondary).width(Fill).into(),
    }
}

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
    // Field for a setting, with the reason that the text in it is not
    // applied, if it is not
    let field = move |setting: Setting,
                      placeholder: &str,
                      value: String,
                      columns: f32,
                      reject: &dyn Fn(&str) -> Option<String>| {
        let shown = typed(setting, value);
        let rejection = reject(&shown);
        let input = text_input(placeholder, &shown)
            .on_input_maybe(editable.then_some(move |text| Message::SettingTyped(setting, text)))
            .width(room(columns))
            .style(theme::field(rejection.is_none()));
        (input, rejection)
    };
    let (name, name_rejection) = field(
        Setting::OutputName(id),
        "Name",
        output.name.clone(),
        NAME_ROOM,
        &|name| reasons::name(app, output, name),
    );
    let (channels, channels_rejection) = field(
        Setting::Channels(id),
        "1-2",
        output.channels.to_string(),
        CHANNELS_ROOM,
        &reasons::channels,
    );
    let (new_cues, new_cues_rejection) = field(
        Setting::NewCues(id),
        "off",
        output.new_cues.map(format::decibels).unwrap_or_default(),
        LEVEL_ROOM,
        &reasons::level,
    );
    let sending = app
        .workspace
        .cues()
        .filter(|cue| cue.kind.plays_file() && app.workspace.sends(cue).contains_key(&id))
        .count();
    let settings = [
        (
            "Name",
            [name.id(OUTPUT_NAME).into(), help("", name_rejection)],
        ),
        (
            "Device",
            [device(app, output), help(device::hint(app, output), None)],
        ),
        (
            "Channels",
            [
                channels.into(),
                help(reasons::channels_help(), channels_rejection),
            ],
        ),
        (
            "New cues send",
            [
                // The unit keeps a gap from the help that follows it, which
                // would otherwise read on from it
                container(row![new_cues, text("dB")].spacing(SPACING).align_y(Center))
                    .padding(Padding::ZERO.right(SPACING))
                    .into(),
                help(
                    "New cues send here at this level; empty for none",
                    new_cues_rejection,
                ),
            ],
        ),
    ];
    let lock = if editable {
        ""
    } else {
        " (read-only in show mode)"
    };
    column![text(format!("Output: {}{lock}", output.name))]
        .extend(settings.into_iter().map(|(label, cells)| {
            row![text(label).width(char_width() * LABEL)]
                .extend(cells)
                .spacing(SPACING)
                .align_y(Center)
                .into()
        }))
        .push(text(match sending {
            1 => "1 cue sends here".to_owned(),
            count => format!("{count} cues send here"),
        }))
        .spacing(SPACING)
        .into()
}

/// Returns the list of the devices that the output can play on, as wide
/// as its name and no wider, whichever device it holds
fn device<'a>(app: &'a App, output: &'a Output) -> Element<'a, Message> {
    container(device::view(app, output))
        .max_width(dropdown_width(DEVICE_ROOM))
        .into()
}
