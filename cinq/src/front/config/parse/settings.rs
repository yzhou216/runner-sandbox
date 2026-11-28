// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the settings of the configuration file that lie outside its
//! keymaps: the address to listen on, the sound device, and the readiness
//! of cues

use std::net::{IpAddr, SocketAddr};

use toml::de::{DeTable, DeValue};

use super::{Problem, spanned};
use crate::back::{Destination, READY_AHEAD};

/// Returns the address that `osc_listen` gives, or none to listen to
/// nothing, with a problem when the setting is not an IP address and a
/// port above 0.  A host name is not taken, since it could name no
/// address or several.
pub(super) fn osc_listen(table: &DeTable) -> (Option<SocketAddr>, Option<Problem>) {
    let Some((value, span)) = table.get("osc_listen").map(spanned) else {
        return (None, None);
    };
    let address = match value {
        DeValue::String(text) => socket_address(text),
        _ => None,
    };
    match address {
        Some(address) => (Some(address), None),
        None => {
            let bad = match value {
                // Settings are read as written, so the space is the problem
                DeValue::String(text) if socket_address(text.trim()).is_some() => {
                    "osc_listen must be IP:port, with no space around it"
                }
                DeValue::String(text) if is_host_name(text.trim()) => {
                    "osc_listen must be IP:port, not a host name"
                }
                _ => "osc_listen must be IP:port, as 127.0.0.1:53000",
            };
            (None, Some(Problem::at(span, bad)))
        }
    }
}

fn socket_address(text: &str) -> Option<SocketAddr> {
    text.parse::<SocketAddr>()
        .ok()
        .filter(|address| address.port() > 0)
}

/// Returns whether `text` is a host name and a port, which `osc_listen` does
/// not take, as against an IP address with a bad port or none
fn is_host_name(text: &str) -> bool {
    text.rsplit_once(':').is_some_and(|(host, port)| {
        Destination::port_is_valid(port)
            && host.parse::<IpAddr>().is_err()
            && !host.starts_with('[')
    })
}

/// Returns the ID that `audio_device` gives, or none for the default
/// device, with a problem when the setting is not a string that could be
/// an ID.  Whether a device has that ID shows only as the device opens.
pub(super) fn audio_device(table: &DeTable) -> (Option<String>, Option<Problem>) {
    match table.get("audio_device").map(spanned) {
        Some((DeValue::String(id), _)) if !id.is_empty() => (Some(id.to_string()), None),
        Some((_, span)) => {
            let bad = "audio_device should be an ID that cinq --audio-devices lists";
            (None, Some(Problem::at(span, bad)))
        }
        None => (None, None),
    }
}

/// Returns the seconds that `ready_ahead` sets, or the default with a
/// problem when the setting is not a number, 0 or more
pub(super) fn ready_ahead(table: &DeTable) -> (f64, Option<Problem>) {
    let Some((value, span)) = table.get("ready_ahead").map(spanned) else {
        return (READY_AHEAD, None);
    };
    let seconds = match value {
        DeValue::Integer(whole) => i64::from_str_radix(whole.as_str(), whole.radix())
            .ok()
            .map(|whole| whole as f64),
        DeValue::Float(fraction) => fraction.as_str().parse().ok(),
        _ => None,
    };
    match seconds.filter(|seconds| (0.0..=f64::MAX).contains(seconds)) {
        Some(seconds) => (seconds, None),
        None => {
            let bad = "ready_ahead should be a number of seconds, 0 or more";
            (READY_AHEAD, Some(Problem::at(span, bad)))
        }
    }
}
