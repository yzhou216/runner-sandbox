// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Device list of the output that the outputs view has selected

use iced::Element;

use crate::{
    back::{Device, Fault, Output},
    front::{
        App, Message,
        view::{
            devices::{Choice, choices, chosen, default_label},
            dropdown, faults, keep_keys, outputs, theme,
        },
    },
};

/// Returns whether the device of the output is not there
pub(super) fn missing(app: &App, output: &Output) -> bool {
    matches!(faults::fault(app, output.id), Some(Fault::Missing(_)))
}

/// Returns the list of the devices that the output can play on, which
/// shows a device that is not there by its name
pub(super) fn view<'a>(app: &'a App, output: &'a Output) -> Element<'a, Message> {
    let id = output.id;
    // Device that is not there, which reads so in words and in yellow
    // rather than in the gray of a list where nothing is chosen
    let missing = missing(app, output);
    let placeholder = placeholder(app, output);
    let options = choices(&app.audio_devices, &default_label(app));
    let default_row = options.iter().position(|option| option.id.is_none());
    let selected = chosen(&options, &output.device)
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
    .placeholder(placeholder)
    // The sidebar tells in color which device the default stands for
    .on_highlight(move |row| Message::DefaultHighlighted(row.is_some() && row == default_row));
    let list = if missing {
        list.placeholder_color(theme::PALETTE.warning)
    } else {
        list
    }
    .on_open(Message::ListAudioDevices);
    keep_keys(list, keymap)
}

/// Returns what the list reads while it shows no choice: the device that is
/// not there, which an output on the Default device keeps the name of too,
/// since the sidebar and not this list chooses it
pub(super) fn placeholder(app: &App, output: &Output) -> String {
    let name = match &output.device {
        Device::Default => default_label(app),
        device => outputs::device(device).to_owned(),
    };
    if missing(app, output) {
        format!("{name}, not found")
    } else {
        name
    }
}

/// Returns the help beside the list: what to do where the device is not
/// there, and nothing where it is.  Choosing another device in this list
/// takes an output off the Default device, which the sidebar changes for all
/// that are on it.
pub(super) fn hint(app: &App, output: &Output) -> &'static str {
    match (missing(app, output), &output.device) {
        (false, _) => "",
        (true, Device::Default) => "Connect it, or set the sidebar default",
        (true, _) => "Connect the device, or pick another",
    }
}
