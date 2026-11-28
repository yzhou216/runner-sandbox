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
    front::{App, Message, config},
};

/// Lists the audio devices on a thread of its own, since a sound server
/// can keep the listing waiting
pub(super) fn listing(serial: u64) -> Task<Message> {
    on_own_thread(audio_devices).map(move |listed| Message::AudioDevicesListed(serial, listed))
}

impl App {
    /// Asks for the devices afresh, which supersedes every asking before,
    /// since each opening of a list asks and the answers of a slow sound
    /// server come in any order.
    pub(super) fn list_devices(self) -> (Self, Task<Message>) {
        let serial = self.device_serial + 1;
        Self {
            device_serial: serial,
            ..self
        }
        .with_task(|_| listing(serial))
    }

    /// A listing that fails keeps the devices listed before, since the
    /// output tells its own problems with the device it plays on.  The answer
    /// to an asking that a later one has superseded changes nothing.
    pub(super) fn with_audio_devices(
        self,
        serial: u64,
        listed: Result<Vec<(String, String)>, String>,
    ) -> Self {
        if serial != self.device_serial {
            return self;
        }
        Self {
            audio_devices: listed.unwrap_or(self.audio_devices),
            ..self
        }
    }

    /// Without a configuration directory, the choice lasts until cinq ends
    pub(super) fn chose_audio_device(self, device: Option<String>) -> (Self, Task<Message>) {
        let keeping = self.config.source.file().map(|file| {
            let (file, device) = (file.to_owned(), device.clone());
            on_own_thread(move || config::keep_audio_device(&file, device.as_deref()))
                .map(Message::AudioDeviceKept)
        });
        let app = Self {
            config: self.config.with_audio_device(device),
            problems: self.problems.with_device(None),
            ..self
        };
        (app, keeping.unwrap_or_else(Task::none))
    }
}
