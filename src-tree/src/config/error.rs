// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Errors reading the configuration file.

use std::{fmt, io, path::PathBuf};

/// Error reading the configuration file
#[derive(Debug)]
pub enum Error {
    /// File could not be read
    Read { path: PathBuf, source: io::Error },
    /// File is not valid TOML or has unknown keys or values
    Parse {
        path: PathBuf,
        source: Box<toml::de::Error>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::Parse { path, source } => {
                write!(f, "invalid configuration in {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {}
