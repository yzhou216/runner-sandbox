// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! List of the audio devices, which chooses the default device of the
//! machine, the one that outputs on the default device play on

use iced::Element;

use crate::front::{
    App, Message,
    view::{
        devices::{Choice, choices},
        dropdown, keep_keys,
    },
};

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let options = choices(&app.audio_devices, "System default");
    let chosen = app.config.sound.device.as_deref();
    let selected = options
        .iter()
        .find(|option| option.id.as_deref() == chosen)
        .cloned();
    let keymap = &app.config.keymap;
    let list = dropdown(
        options,
        selected,
        |choice: Choice| Message::ChooseAudioDevice(choice.id),
        keymap,
    )
    // Device that is not there goes by its ID, which the warnings name
    .placeholder(chosen.unwrap_or_default())
    .on_open(Message::ListAudioDevices)
    // Outputs on the default device play on the device that this list
    // chooses, which the Device list of an output tells in blue while its
    // highlight rests on the default
    .lit(app.menu && app.default_highlighted);
    keep_keys(list, keymap)
}
