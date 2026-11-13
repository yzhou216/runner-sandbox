// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Options of the lists of audio devices, the default first
//!
//! Each device goes by its name, and by its ID as well where several
//! devices share the name, as the devices of one sound card do on ALSA.
//! Each list asks for the devices afresh as it opens, so that a device
//! plugged in since shows.  The default goes by the name that each list
//! gives it, since the default of an output is the device that the
//! sidebar chooses, and the default of the sidebar is that of the system.

#[cfg(test)]
mod tests;

use std::{fmt, iter};

/// Option of a list of audio devices
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Choice {
    /// ID of the device, or none for the default device
    pub(super) id: Option<String>,
    /// Name of the device as the system gives it
    pub(super) name: String,
    label: String,
}

impl fmt::Display for Choice {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label)
    }
}

/// Returns the default, labeled `default`, then `devices`, by the ID and
/// the name of each
pub(super) fn choices(devices: &[(String, String)], default: &str) -> Vec<Choice> {
    let shared = |name: &str| devices.iter().filter(|(_, other)| other == name).count() > 1;
    let default = Choice {
        id: None,
        name: String::new(),
        label: default.to_owned(),
    };
    let listed = devices.iter().map(|(id, name)| Choice {
        id: Some(id.clone()),
        name: name.clone(),
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
