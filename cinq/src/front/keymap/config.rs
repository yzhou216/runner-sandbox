// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Configuration file, which chooses a keymap and binds actions to keys
//!
//! The file is `cinq/config.toml` in the platform's configuration
//! directory, as the dirs crate finds it, unless the command line names a
//! directory of its own to read `config.toml` from.  [`parse`](mod@parse)
//! reads its text into key bindings, and [`watch`](fn@watch) reads the file
//! again whenever it changes, so that the keys follow the file while the
//! program runs, and writes what is wrong with it to standard error each
//! time, pointing into its text, as [`problem`] tells.
//!
//! `keymap` names the keymap that the keys start from, which is vi unless
//! the file names another: a shipped one, or one of the `[keymaps]` table,
//! which starts from the keymap that its `from` names, or from no keys at
//! all without one.  A keymap of the file's own and the `[keys]` table bind
//! each action they name to a key sequence or a list of them, in place of
//! the keys that the keymap they build on gives it:
//!
//! ```toml
//! keymap = "booth"
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
use crate::front::read;
pub(super) use parse::parse;
pub use problem::Problem;
pub use watch::{Errors, guard, restore, watch};

/// Key bindings, along with where they came from and what was wrong with
/// the file they came from
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub keymap: Keymap,
    pub source: Source,
    /// Text of the file, when it could be read, which the problems point
    /// into
    pub text: Option<String>,
    /// Problems with the file, each of which leaves the keys of the keymap
    /// in place
    pub problems: Vec<Problem>,
}

/// Where the key bindings came from
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

/// Returns the file that may hold the configuration: `config.toml` in
/// `directory`, when one is given, or else `cinq/config.toml` in the
/// platform's configuration directory, as the dirs crate finds it, if it
/// has one.  The directory goes by its absolute path, which the watcher
/// reports changes by.
pub fn file(directory: Option<&Path>) -> Option<PathBuf> {
    directory
        .map(|directory| path::absolute(directory).unwrap_or_else(|_| directory.to_owned()))
        .or_else(|| dirs::config_dir().map(|directory| directory.join("cinq")))
        .map(|directory| directory.join("config.toml"))
}

/// Text of a configuration file that [`make`] makes: a line that points
/// to the manual page of the file, and nothing that changes a key
const MADE: &str = "# Key bindings of cinq, which the manual page cinq(5) describes\n";

/// Makes the configuration file `file`, along with the directories that it
/// goes in, holding [`MADE`], unless the file is there already
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

/// Reads the configuration from `file`, or gives the defaults when there
/// is no such file
pub fn load(file: Option<&Path>) -> Config {
    let Some(file) = file else {
        return Config::default();
    };
    match read(file) {
        Ok(Some(text)) => {
            let (keymap, problems) = parse(&text);
            Config {
                keymap,
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
