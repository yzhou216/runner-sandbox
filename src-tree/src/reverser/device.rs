// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Devices that scroll events come from, and selections of them.

use std::fmt;

use clap::ValueEnum;
use serde::Deserialize;

/// Kind of device a scroll event came from
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    /// Mouse, including the Magic Mouse and mice with free-spinning wheels
    Mouse,
    /// Built-in or external trackpad
    Trackpad,
}

impl Device {
    /// Guesses the device for events of unknown senders: touch surfaces set a
    /// scroll or momentum phase, and wheels set neither.
    pub fn from_phases((scroll, momentum): (i64, i64)) -> Self {
        if scroll == 0 && momentum == 0 {
            Self::Mouse
        } else {
            Self::Trackpad
        }
    }
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Mouse => "mouse",
            Self::Trackpad => "trackpad",
        })
    }
}

/// Devices whose scrolling is reversed
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Devices {
    /// Mice, including the Magic Mouse
    #[default]
    Mouse,
    /// Built-in and external trackpads
    Trackpad,
    /// Every device
    All,
}

impl Devices {
    /// Returns whether scrolling from `device` is reversed.
    pub fn contains(self, device: Device) -> bool {
        match self {
            Self::Mouse => device == Device::Mouse,
            Self::Trackpad => device == Device::Trackpad,
            Self::All => true,
        }
    }

    /// Names the selection for log messages.
    pub fn summary(self) -> &'static str {
        match self {
            Self::Mouse => "mouse",
            Self::Trackpad => "trackpad",
            Self::All => "mouse and trackpad",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phases_tell_wheels_from_touch_surfaces() {
        assert_eq!(Device::from_phases((0, 0)), Device::Mouse);
        // Began, changed, ended, cancelled, and may-begin scroll phases
        [1, 2, 4, 8, 128]
            .into_iter()
            .for_each(|phase| assert_eq!(Device::from_phases((phase, 0)), Device::Trackpad));
        // Begin, continue, and end momentum phases
        [1, 2, 3]
            .into_iter()
            .for_each(|phase| assert_eq!(Device::from_phases((0, phase)), Device::Trackpad));
    }

    #[test]
    fn selections_contain_their_devices() {
        let contained = |devices: Devices| {
            [Device::Mouse, Device::Trackpad]
                .into_iter()
                .filter(|&device| devices.contains(device))
                .collect::<Vec<_>>()
        };
        assert_eq!(contained(Devices::Mouse), [Device::Mouse]);
        assert_eq!(contained(Devices::Trackpad), [Device::Trackpad]);
        assert_eq!(contained(Devices::All), [Device::Mouse, Device::Trackpad]);
    }
}
