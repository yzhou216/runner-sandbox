// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration file, which chooses a keymap, binds actions to keys, and
//! sets how far ahead sounds are made ready
//!
//! `keymap` names the keymap that the keys start from, which is vi unless
//! the file names another: a shipped one, or one of the `[keymaps]` table,
//! which starts from the keymap that its `from` names, or from no keys at
//! all without one.  A keymap of the file's own and the `[keys]` table bind
//! each action they name to a key sequence or a list of them, in place of
//! the keys that the keymap they build on gives it.  `ready_ahead` counts
//! the seconds before a sound starts that it is made ready:
//!
//! ```toml
//! keymap = "booth"
//! ready_ahead = 5
//!
//! [keymaps.booth]
//! from = "cua"
//! go = ["Space", "F5"]
//!
//! [keys]
//! delete = []
//! ```

mod parse;
mod problem;
mod watch;

use std::{
    fs::{self, File},
    io::{self, Write},
    path::{self, Path, PathBuf},
};

use super::Keymap;
use crate::{back::READY_AHEAD, front::read};
pub(super) use parse::parse;
pub use problem::Problem;
pub use watch::{Errors, guard, restore, watch};

#[derive(Debug, Clone)]
pub struct Config {
    pub keymap: Keymap,
    pub ready_ahead: f64,
    pub source: Source,
    /// Text of the file, when it could be read, which the problems point
    /// into
    text: Option<String>,
    /// Problems with the file, each of which leaves the keys of the keymap
    /// in place
    pub problems: Vec<Problem>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keymap: Keymap::default(),
            ready_ahead: READY_AHEAD,
            source: Source::default(),
            text: None,
            problems: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Source {
    /// Configuration file that is there, even when it cannot be read
    File(PathBuf),
    /// Defaults, which the missing file would change
    Missing(PathBuf),
    /// Defaults, since the platform has no configuration directory
    #[default]
    NoDirectory,
}

impl Source {
    /// Returns the configuration file, which the keys follow as it changes
    pub fn file(&self) -> Option<&Path> {
        match self {
            Source::File(file) | Source::Missing(file) => Some(file),
            Source::NoDirectory => None,
        }
    }
}

/// The directory goes by its absolute path, which the watcher reports
/// changes by.
pub fn file(directory: Option<&Path>) -> Option<PathBuf> {
    directory
        .map(|directory| path::absolute(directory).unwrap_or_else(|_| directory.to_owned()))
        .or_else(|| dirs::config_dir().map(|directory| directory.join("cinq")))
        .map(|directory| directory.join("config.toml"))
}

const MADE: &str = "# Settings of cinq, which the manual page cinq(5) describes\n";

pub fn make(file: &Path) -> io::Result<()> {
    if let Some(directory) = file.parent() {
        fs::create_dir_all(directory)?;
    }
    match File::create_new(file) {
        Ok(mut made) => made.write_all(MADE.as_bytes()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn load(file: Option<&Path>) -> Config {
    let Some(file) = file else {
        return Config::default();
    };
    match read(file) {
        Ok(Some(text)) => {
            let (keymap, ready_ahead, problems) = parse(&text);
            Config {
                keymap,
                ready_ahead,
                source: Source::File(file.to_owned()),
                text: Some(text),
                problems,
            }
        }
        Err(error) => Config {
            source: Source::File(file.to_owned()),
            problems: vec![Problem::new(format!("cannot be read: {error}"))],
            ..Config::default()
        },
        Ok(None) => Config {
            source: Source::Missing(file.to_owned()),
            ..Config::default()
        },
    }
}
