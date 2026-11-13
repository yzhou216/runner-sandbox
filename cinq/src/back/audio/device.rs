// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio devices, which the configuration file names by the IDs that cpal
//! gives them, since devices on ALSA share names: a dozen of them can be
//! named after one sound card

use std::str::FromStr;

use kira::backend::cpal::cpal::{
    self, Device, DeviceId,
    traits::{DeviceTrait, HostTrait},
};

/// Returns the ID and the name of each audio device that sound can play on
pub fn devices() -> Result<Vec<(String, String)>, String> {
    let devices = cpal::default_host()
        .output_devices()
        .map_err(|error| error.to_string())?;
    Ok(devices
        .filter_map(|device| {
            let id = device.id().ok()?.to_string();
            let name = device
                .description()
                .map(|description| description.name().to_owned())
                .unwrap_or_default();
            Some((id, name))
        })
        .collect())
}

/// Returns the audio device whose ID is `id`, or why there is none
pub(super) fn find(id: &str) -> Result<Device, String> {
    DeviceId::from_str(id)
        .ok()
        .and_then(|parsed| {
            cpal::host_from_id(parsed.host())
                .ok()?
                .device_by_id(&parsed)
        })
        .ok_or_else(|| format!("there is no audio device {id}"))
}
