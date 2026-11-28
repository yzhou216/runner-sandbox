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
            devices::{Choice, choices, chosen, default_label, named},
            dropdown, faults, keep_keys, theme,
        },
    },
};

fn missing(app: &App, output: &Output) -> bool {
    matches!(faults::fault(app, output.id), Some(Fault::Missing(_)))
}

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
    // Sidebar tells in color which device the default stands for
    .on_highlight(move |row| Message::DefaultHighlighted(row.is_some() && row == default_row))
    .placeholder_color(missing.then_some(theme::PALETTE.warning))
    .on_open(Message::ListAudioDevices);
    keep_keys(list, keymap)
}

/// Returns what the list reads while it shows no choice: the device that is
/// not there, which an output on the Default device keeps the name of too,
/// since the sidebar and not this list chooses it
pub(super) fn placeholder(app: &App, output: &Output) -> String {
    let name = named(app, &output.device);
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
        (true, Device::Default) => "Connect it, or choose another Default device in the sidebar",
        (true, _) => "Connect the device, or choose another",
    }
}
