// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Audio devices, which the configuration file names by the IDs that cpal
//! gives them, since devices on ALSA share names: a dozen of them can be
//! named after one sound card
//!
//! The devices of every host that cpal was built with are listed, those of
//! the default host first, as JACK's follow ALSA's on GNU/Linux.  A host
//! that is not running, as JACK without its server, lists nothing.

use std::str::FromStr;

use kira::backend::cpal::cpal::{
    self, Device, DeviceDirection, DeviceId, Host,
    traits::{DeviceTrait, HostTrait},
};

/// Returns the ID and the name of each audio device that sound can play on,
/// or why those of the default host cannot be listed
pub fn devices() -> Result<Vec<(String, String)>, String> {
    let default = cpal::default_host();
    let first = listed(&default).map_err(|error| error.to_string())?;
    let others = cpal::available_hosts()
        .into_iter()
        .filter(|id| *id != default.id())
        .filter_map(|id| cpal::host_from_id(id).ok())
        .flat_map(|host| listed(&host).unwrap_or_default());
    Ok(first.into_iter().chain(others).collect())
}

/// Returns the devices of `host` that sound can play on, leaving out those
/// that tell that they only record, as JACK's input does, which cpal lists
/// among the outputs as well
fn listed(host: &Host) -> Result<Vec<(String, String)>, cpal::Error> {
    Ok(host
        .output_devices()?
        .filter_map(|device| {
            let id = device.id().ok()?.to_string();
            let description = device.description().ok();
            let records = description
                .as_ref()
                .is_some_and(|description| description.direction() == DeviceDirection::Input);
            let name =
                description.map_or_else(String::new, |description| description.name().to_owned());
            (!records).then_some((id, name))
        })
        .collect())
}

/// Returns the audio device whose ID is `id`, or else the only device
/// named `name`, as on a machine that gives the device another ID, or why
/// there is none.  Two devices of one name are told apart by ID alone.
pub(super) fn find(id: &str, name: Option<&str>) -> Result<Device, String> {
    let named = || {
        let name = name.filter(|name| !name.is_empty())?;
        let mut alike = cpal::default_host()
            .output_devices()
            .ok()?
            .filter(|device| {
                device
                    .description()
                    .is_ok_and(|description| description.name() == name)
            });
        let found = alike.next()?;
        alike.next().is_none().then_some(found)
    };
    DeviceId::from_str(id)
        .ok()
        .and_then(|parsed| {
            cpal::host_from_id(parsed.host())
                .ok()?
                .device_by_id(&parsed)
        })
        .or_else(named)
        .ok_or_else(|| {
            let named = name.filter(|name| !name.is_empty()).unwrap_or(id);
            format!("there is no audio device {named}")
        })
}
