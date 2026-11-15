// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Choosing the audio device that sound plays on, which the configuration
//! file keeps
//!
//! The choice applies at once, and the configuration file takes it on a
//! thread of its own.  The reading of the file that follows names the same
//! device, so the sound stays where it is.

use iced::Task;

use super::on_own_thread;
use crate::{
    back::audio_devices,
    front::{App, Message, keymap::config},
};

/// Lists the audio devices on a thread of its own, since a sound server
/// can keep the listing waiting
pub(in crate::front) fn listing() -> Task<Message> {
    on_own_thread(audio_devices).map(Message::AudioDevicesListed)
}

impl App {
    /// A listing that fails keeps the devices listed before, since the
    /// output tells its own problems with the device it plays on.
    pub(super) fn with_audio_devices(self, listed: Result<Vec<(String, String)>, String>) -> Self {
        Self {
            audio_devices: listed.unwrap_or(self.audio_devices),
            ..self
        }
    }

    /// Without a configuration directory, the choice lasts until cinq ends.
    pub(super) fn chose_audio_device(self, device: Option<String>) -> (Self, Task<Message>) {
        let keeping = self.config.source.file().map(|file| {
            let (file, device) = (file.to_owned(), device.clone());
            on_own_thread(move || config::keep_audio_device(&file, device.as_deref()))
                .map(Message::AudioDeviceKept)
        });
        let app = Self {
            config: self.config.with_audio_device(device),
            device_problem: None,
            ..self
        };
        (app, keeping.unwrap_or_else(Task::none))
    }
}
