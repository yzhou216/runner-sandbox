// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! List of the audio devices that sound can play on
//!
//! Each device goes by its name, and by its ID as well where several
//! devices share the name, as the devices of one sound card do on ALSA.
//! The list asks for the devices afresh each time it opens, so that a
//! device plugged in since shows.

#[cfg(test)]
mod tests;

use std::{fmt, iter};

use iced::Element;

use crate::front::{
    App, Message,
    view::{dropdown, keep_keys},
};

/// Option of the list
#[derive(Debug, Clone, PartialEq)]
struct Output {
    /// ID of the device, or none for the default device
    id: Option<String>,
    label: String,
}

impl fmt::Display for Output {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

pub(super) fn view(app: &App) -> Element<'_, Message> {
    let options = options(&app.audio_devices);
    let chosen = app.config.sound.device.as_deref();
    let selected = options
        .iter()
        .find(|option| option.id.as_deref() == chosen)
        .cloned();
    let keymap = &app.config.keymap;
    let list = dropdown(
        options,
        selected,
        |output: Output| Message::ChooseAudioDevice(output.id),
        keymap,
    )
    // Device that is not there goes by its ID, which the warnings name
    .placeholder(chosen.unwrap_or_default())
    .on_open(Message::ListAudioDevices);
    keep_keys(list, keymap)
}

fn options(devices: &[(String, String)]) -> Vec<Output> {
    let shared = |name: &str| devices.iter().filter(|(_, other)| other == name).count() > 1;
    let default = Output {
        id: None,
        label: "Default device".to_owned(),
    };
    let listed = devices.iter().map(|(id, name)| Output {
        id: Some(id.clone()),
        label: if name.is_empty() {
            id.clone()
        } else if shared(name) {
            format!("{name} ({id})")
        } else {
            name.clone()
        },
    });
    iter::once(default).chain(listed).collect()
}
