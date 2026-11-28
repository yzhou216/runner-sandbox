// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading of the files that the interface owns, with failures told the
//! way the interface tells them

use std::{fs, io, path::Path};

use crate::warn;

/// Reads a file, or returns none when there is no such file.  A failure
/// goes on standard error whole, with the path, as a failed save does, and
/// the screen gets its words alone, as `permission denied`.
pub(super) fn read(file: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(file) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => {
            warn::once(&format!("cannot open {}: {error}", file.display()));
            Err(warn::words(&error))
        }
    }
}
