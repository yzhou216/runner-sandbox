// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration file, documented in scrollreversed(5).

mod error;
mod path;

use std::{fs, path::Path};

pub use self::{
    error::Error,
    path::{default_path, documented_path},
};
use crate::reverser::Settings;

pub fn parse(text: &str) -> Result<Settings, toml::de::Error> {
    toml::from_str(text)
}

/// Reads `path`, or the default file if it exists.
pub fn load(path: Option<&Path>) -> Result<Settings, Error> {
    match path {
        Some(path) => read(path),
        None => default_path()
            .filter(|path| path.exists())
            .map_or_else(|| Ok(Settings::default()), |path| read(&path)),
    }
}

fn read(path: &Path) -> Result<Settings, Error> {
    fs::read_to_string(path)
        .map_err(|source| Error::Read {
            path: path.to_owned(),
            source,
        })
        .and_then(|text| {
            parse(&text).map_err(|source| Error::Parse {
                path: path.to_owned(),
                source: Box::new(source),
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reverser::{Axes, Devices};

    #[test]
    fn empty_file_gives_no_settings() {
        assert_eq!(parse("").ok(), Some(Settings::default()));
    }

    #[test]
    fn keys_set_their_settings() {
        assert_eq!(
            parse("device = \"trackpad\"\naxis = \"vertical\"\nverbose = true\n").ok(),
            Some(Settings {
                device: Some(Devices::Trackpad),
                axis: Some(Axes::Vertical),
                verbose: Some(true),
            })
        );
    }

    #[test]
    fn mistakes_are_rejected() {
        [
            "device = \"pen\"",
            "devices = \"mouse\"",
            "verbose = \"yes\"",
        ]
        .into_iter()
        .for_each(|text| assert!(parse(text).is_err(), "{text} parsed"));
    }

    #[test]
    fn missing_explicit_file_is_an_error() {
        assert!(matches!(
            load(Some(Path::new("/nonexistent/scrollreversed.toml"))),
            Err(Error::Read { .. })
        ));
    }
}
