// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Settings of the selected output, each a label, a field, and a line of help

use iced::{
    Center, Element, Padding,
    widget::{container, row, text, text_input},
};

use super::{super::LEVEL_ROOM, CHANNELS_ROOM, NAME_ROOM, device, help, typing};
use crate::{
    back::Output,
    front::{
        App, Message, Setting,
        message::reasons,
        view::{OUTPUT_NAME, SPACING, format, reached, theme},
    },
};

/// Returns the settings of `output`, each as its label and the cells that
/// follow it: the field, and the help beside it
pub(super) fn of<'a>(
    app: &'a App,
    output: &'a Output,
) -> [(&'static str, [Element<'a, Message>; 2]); 4] {
    [
        ("Name", name(app, output)),
        (
            "Device",
            [device(app, output), help(device::hint(app, output), None)],
        ),
        ("Channels", channels(app, output)),
        ("New cues send", new_cues(app, output)),
    ]
}

/// Returns the field for `setting`, with the reason that the text in it is
/// not applied, if it is not
fn field<'a>(
    app: &App,
    setting: Setting,
    placeholder: &str,
    value: String,
    columns: f32,
    reject: &dyn Fn(&str) -> Option<String>,
) -> (text_input::TextInput<'a, Message>, Option<String>) {
    let (input, shown) = typing(app, setting, placeholder, value, columns);
    let rejection = reject(&shown);
    (input.style(theme::field(rejection.is_none())), rejection)
}

fn name<'a>(app: &App, output: &Output) -> [Element<'a, Message>; 2] {
    let (name, rejection) = field(
        app,
        Setting::OutputName(output.id),
        "Name",
        output.name.clone(),
        NAME_ROOM,
        &|name| reasons::name(app, output, name),
    );
    [reached(name.id(OUTPUT_NAME)), help("", rejection)]
}

fn channels<'a>(app: &App, output: &Output) -> [Element<'a, Message>; 2] {
    let (channels, rejection) = field(
        app,
        Setting::Channels(output.id),
        "1-2",
        output.channels.to_string(),
        CHANNELS_ROOM,
        &reasons::channels,
    );
    [reached(channels), help(reasons::channels_help(), rejection)]
}

/// Returns the cells of the level that new cues send to `output`
fn new_cues<'a>(app: &App, output: &Output) -> [Element<'a, Message>; 2] {
    let (new_cues, rejection) = field(
        app,
        Setting::NewCues(output.id),
        "off",
        output.new_cues.map(format::decibels).unwrap_or_default(),
        LEVEL_ROOM,
        &reasons::level,
    );
    [
        // Unit keeps a gap from the help that follows it, which would
        // otherwise read on from it
        container(
            row![reached(new_cues), text("dB")]
                .spacing(SPACING)
                .align_y(Center),
        )
        .padding(Padding::ZERO.right(SPACING))
        .into(),
        help(
            "New cues send here at this level; empty for none",
            rejection,
        ),
    ]
}
