// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Policy that decides which scroll events to reverse.

mod axis;
mod device;
mod settings;

use std::fmt;

pub use self::{
    axis::{Axes, Axis},
    device::{Device, Devices},
    settings::Settings,
};

/// Which scroll events to reverse, built by chaining setters
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Reverser {
    devices: Devices,
    axes: Axes,
    verbose: bool,
}

impl Reverser {
    pub fn devices(self, devices: Devices) -> Self {
        Self { devices, ..self }
    }

    pub fn axes(self, axes: Axes) -> Self {
        Self { axes, ..self }
    }

    pub fn verbose(self, verbose: bool) -> Self {
        Self { verbose, ..self }
    }

    /// Applies the settings that one source gives, keeping the others.
    pub fn with(self, settings: Settings) -> Self {
        let Settings {
            device,
            axis,
            verbose,
        } = settings;
        Self {
            devices: device.unwrap_or(self.devices),
            axes: axis.unwrap_or(self.axes),
            verbose: verbose.unwrap_or(self.verbose),
        }
    }

    pub fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Returns the axes to reverse for a scroll event from `device`.
    pub fn reversed_axes(&self, device: Device) -> impl Iterator<Item = Axis> {
        let Self { devices, axes, .. } = *self;
        Axis::ALL
            .into_iter()
            .filter(move |&axis| devices.contains(device) && axes.contains(axis))
    }
}

impl fmt::Display for Reverser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} scrolling on {}",
            self.devices.summary(),
            self.axes.summary()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reversed(reverser: Reverser, device: Device) -> Vec<Axis> {
        reverser.reversed_axes(device).collect()
    }

    #[test]
    fn default_reverses_the_mouse_on_both_axes() {
        let reverser = Reverser::default();
        assert_eq!(reversed(reverser, Device::Mouse), Axis::ALL);
        assert_eq!(reversed(reverser, Device::Trackpad), []);
    }

    #[test]
    fn selection_limits_devices_and_axes() {
        let reverser = Reverser::default()
            .devices(Devices::Trackpad)
            .axes(Axes::Horizontal);
        assert_eq!(reversed(reverser, Device::Trackpad), [Axis::Horizontal]);
        assert_eq!(reversed(reverser, Device::Mouse), []);
    }

    #[test]
    fn later_settings_override_earlier_ones() {
        let file = Settings {
            device: Some(Devices::All),
            axis: Some(Axes::Vertical),
            verbose: None,
        };
        let arguments = Settings {
            device: Some(Devices::Trackpad),
            ..Settings::default()
        };
        assert_eq!(
            Reverser::default().with(file).with(arguments),
            Reverser::default()
                .devices(Devices::Trackpad)
                .axes(Axes::Vertical)
        );
    }

    #[test]
    fn empty_settings_change_nothing() {
        let earlier = Reverser::default().devices(Devices::All).verbose(true);
        assert_eq!(earlier.with(Settings::default()), earlier);
    }

    #[test]
    fn summary_names_devices_and_axes() {
        assert_eq!(
            Reverser::default().to_string(),
            "mouse scrolling on both axes"
        );
        assert_eq!(
            Reverser::default()
                .devices(Devices::All)
                .axes(Axes::Vertical)
                .to_string(),
            "mouse and trackpad scrolling on the vertical axis"
        );
    }
}
