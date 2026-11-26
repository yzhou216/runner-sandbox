// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hazards available by name.
//!
//! The hazards in `hazards/` of the source tree are compiled into geoai.
//! Hazards loaded later replace earlier ones of the same name, so a user file
//! can override a built-in hazard.

use std::{
    collections::{BTreeMap, BTreeSet},
    env, fmt, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{hazard::Hazard, script};

/// File names and Scheme sources of the built-in hazards.
const BUILTIN: [(&str, &str); 1] = [("earthquake.scm", include_str!("../hazards/earthquake.scm"))];

#[derive(Default)]
pub struct Catalog {
    entries: BTreeMap<String, Entry>,
}

pub struct Entry {
    pub hazard: Hazard,
    pub source: Source,
}

/// Where a hazard was defined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Builtin,
    File(PathBuf),
}

impl fmt::Display for Source {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Builtin => formatter.write_str("built-in"),
            Self::File(path) => write!(formatter, "{}", path.display()),
        }
    }
}

impl Catalog {
    /// Catalog of the hazards built into geoai.
    pub fn builtin() -> Result<Self> {
        BUILTIN
            .iter()
            .try_fold(Self::default(), |catalog, (name, source)| {
                Ok(catalog.with_hazards(script::load(source, name)?, &Source::Builtin))
            })
    }

    /// Add the hazards defined in the Scheme file at `path`, or in each `.scm`
    /// file of the directory at `path` in name order.
    pub fn with_path(self, path: &Path) -> Result<Self> {
        if path.is_dir() {
            scheme_files(path)
                .with_context(|| format!("cannot list {}", path.display()))?
                .iter()
                .try_fold(self, |catalog, file| catalog.with_file(file))
        } else {
            self.with_file(path)
        }
    }

    pub fn get(&self, name: &str) -> Option<&Hazard> {
        self.entries.get(name).map(|entry| &entry.hazard)
    }

    /// Entries in name order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.values()
    }

    fn with_file(self, path: &Path) -> Result<Self> {
        let hazards = script::load_file(path)?;
        Ok(self.with_hazards(hazards, &Source::File(path.to_owned())))
    }

    fn with_hazards(self, hazards: Vec<Hazard>, source: &Source) -> Self {
        let mut entries = self.entries;
        entries.extend(hazards.into_iter().map(|hazard| {
            let name = hazard.name().to_owned();
            let source = source.clone();
            (name, Entry { hazard, source })
        }));
        Self { entries }
    }
}

/// Directory of the user's hazard files, `geoai/hazards` in the user
/// configuration directory.
pub fn user_directory() -> Option<PathBuf> {
    config_directory().map(|config| config.join("geoai").join("hazards"))
}

/// `$XDG_CONFIG_HOME` if it is an absolute path, or else `~/.config`, on
/// every POSIX system.  This includes macOS, whose platform convention would
/// otherwise be `~/Library/Application Support`.  Other systems follow their
/// own convention, such as `%APPDATA%` on Windows.
fn config_directory() -> Option<PathBuf> {
    if cfg!(unix) {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|directory| directory.is_absolute())
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
    } else {
        dirs::config_dir()
    }
}

/// Scheme files in `directory`, leaving out hidden files such as the lock
/// files that editors create.
fn scheme_files(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<BTreeSet<_>>>()?;
    Ok(paths
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "scm"))
        .filter(|path| !is_hidden(path))
        .collect())
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.as_encoded_bytes().starts_with(b"."))
}
