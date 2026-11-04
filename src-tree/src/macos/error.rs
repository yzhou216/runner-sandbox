// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reasons the daemon stops.

use std::fmt;

/// Reason the daemon stopped
#[derive(Debug)]
pub enum Error {
    /// Window server refused the tap although the process is trusted
    Refused,
    /// Access was granted while waiting, but the window server only honours
    /// it for a new process, which launchd starts in its place
    Restart,
    /// Run loop source for the tap could not be created
    Source,
    /// Run loop ran out of sources, which happens when the window server
    /// invalidates the tap
    Invalidated,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Refused => "the window server refused to create the event tap",
            Self::Restart => {
                "Accessibility access granted; exiting so that launchd restarts with it"
            }
            Self::Source => "cannot create a run loop source for the event tap",
            Self::Invalidated => "the window server invalidated the event tap",
        })
    }
}

impl std::error::Error for Error {}
