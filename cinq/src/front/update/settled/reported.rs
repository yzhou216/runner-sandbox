// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Messages that report how the network, the configuration, a link, or an
//! audio device went

use super::super::{App, Message};

impl App {
    /// Applies a message that reports a problem or a state outside the
    /// interface, and passes any other on to [`Self::held_by`]
    pub(super) fn reported_by(self, message: Message) -> Self {
        match message {
            Message::NetworkChecked | Message::LightTold(_) => self,
            Message::OscFailed(problem) => Self {
                problems: self.problems.with_osc(Some(problem)),
                ..self
            },
            Message::OscIgnored(line) => self.with_ignored(line),
            Message::OscListening => Self {
                problems: self.problems.with_osc(None),
                ..self
            },
            // Keys typed halfway went by the bindings before, so the update
            // drops them, as it does for most messages
            Message::Reconfigured(config) => Self {
                problems: self
                    .problems
                    .with_osc_setting(config.osc_listen == self.config.osc_listen),
                config: *config,
                ..self
            },
            Message::LinkOpened(result) => Self {
                problems: self.problems.with_link(result.err()),
                ..self
            },
            Message::AudioDevicesListed(listed) => self.with_audio_devices(listed),
            Message::AudioDeviceKept(kept) => Self {
                problems: self.problems.with_device(kept.err()),
                ..self
            },
            message => self.held_by(message),
        }
    }
}
