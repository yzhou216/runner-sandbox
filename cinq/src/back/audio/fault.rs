// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Why an output cannot play, which tells whether cinq mends it itself, as
//! it opens a device again every two seconds, or the show must change, as
//! for channels past those of the device

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Fault {
    /// The device is not there, as one unplugged or not started, and why
    Missing(String),
    /// The device is there but cannot open, and why
    Refused(String),
    /// The device has stopped while it played, and why
    Stopped(String),
    /// The channels of the output lie past the count that the device has
    Channels(u16),
}

impl Fault {
    /// Returns whether cinq tries the device again by itself
    pub fn retried(&self) -> bool {
        !matches!(self, Fault::Channels(_))
    }

    /// Returns what the fault does to the output, in a few words
    pub fn state(&self) -> String {
        match self {
            Fault::Missing(_) => "Not found, retrying".to_owned(),
            Fault::Refused(_) => "Cannot open, retrying".to_owned(),
            Fault::Stopped(_) => "Stopped, retrying".to_owned(),
            Fault::Channels(count) => format!("Past its {count} channels"),
        }
    }

    /// Returns why the device fails, where it fails for a reason beyond
    /// its absence or its channels
    pub fn reason(&self) -> Option<&str> {
        match self {
            Fault::Refused(reason) | Fault::Stopped(reason) => Some(reason),
            Fault::Missing(_) | Fault::Channels(_) => None,
        }
    }
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::Missing(reason) => formatter.write_str(reason),
            Fault::Refused(reason) => write!(formatter, "cannot open: {reason}"),
            Fault::Stopped(reason) => write!(formatter, "stopped: {reason}"),
            Fault::Channels(count) => {
                write!(
                    formatter,
                    "its channels lie past the {count} that the device has"
                )
            }
        }
    }
}
