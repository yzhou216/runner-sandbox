// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Failures of the file that a cue names
//!
//! The sound, video, and picture outputs, and the loading of files, tell
//! alike why a file cannot be read and mark the cue that names it
//! unplayable.

#[cfg(test)]
mod tests;

use std::io;

use super::{Command, Cue, Media, Stamp, Workspace};
use crate::warn;

/// Returns the command that records why the file of `cue` cannot be
/// played, with the stamp that the file has now, so that the failure stays
/// until the file changes, which a read of its header alone could not say
/// it had cured
pub(super) fn unplayable(workspace: &Workspace, cue: &Cue, reason: String) -> Command {
    let seen = workspace
        .path(cue)
        .and_then(|path| Stamp::of(&path)?.when_readable(&path));
    Command::Loaded {
        cue: cue.id,
        file: cue.file.clone(),
        media: Media::Failed(reason),
        seen,
    }
}

/// Tells why a file cannot be read, in the same words for sound, video,
/// and picture files, and without the error number of the operating system
pub(super) fn unreadable(error: io::Error) -> String {
    match error.kind() {
        io::ErrorKind::NotFound => "file not found".to_owned(),
        _ => warn::words(&error),
    }
}
