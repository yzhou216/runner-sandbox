// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! List of the audio devices, which chooses the device that outputs on the
//! default device play on

use iced::Element;

use crate::{
    back::Device,
    front::{
        App, Message,
        view::{
            devices::{Choice, choices},
            dropdown, faults, keep_keys, theme,
        },
    },
};

/// Returns whether an output on the default device cannot play, while the
/// list shows a device that it does not find
pub(super) fn missing(app: &App) -> bool {
    app.workspace
        .outputs()
        .iter()
        .any(|output| output.device == Device::Default && faults::fault(app, output.id).is_some())
}

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let options = choices(&app.audio_devices, "System default");
    let chosen = app.config.sound.device.as_deref();
    let selected = options
        .iter()
        .find(|option| option.id.as_deref() == chosen)
        .cloned();
    let keymap = &app.config.keymap;
    // Device that is not there reads in yellow, as its warning does, rather
    // than in the gray of a list where nothing is chosen, and says so first,
    // so that a name cut to the room of the list keeps the words
    let lost = selected.is_none() && missing(app);
    let list = dropdown(
        options,
        selected,
        |choice: Choice| Message::ChooseAudioDevice(choice.id),
        keymap,
    )
    // Device that is not there goes by its ID, which the warnings name
    .placeholder(placeholder(chosen.unwrap_or_default(), lost))
    .on_open(Message::ListAudioDevices)
    // Outputs on the default device play on the device that this list
    // chooses, which the Device list of an output tells in blue while its
    // highlight rests on the default
    .lit(app.keyboard.menu && app.keyboard.default_highlighted);
    keep_keys(
        list.placeholder_color(lost.then_some(theme::PALETTE.warning)),
        keymap,
    )
}

/// Returns what the list reads while it shows no choice: the ID of the device
/// that is not there, led by words where it is `lost`, so that the yellow is
/// not the only sign
pub(super) fn placeholder(chosen: &str, lost: bool) -> String {
    if lost {
        format!("Not found: {chosen}")
    } else {
        chosen.to_owned()
    }
}
