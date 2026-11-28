// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening the links of the help page, the configuration file and the
//! tutorial among them, in the programs that the desktop opens them with

#[cfg(test)]
mod tests;

use std::{
    env,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use iced::Task;

use super::on_own_thread;
use crate::{
    front::{App, Message, config},
    warn,
};

/// Path of the tutorial under a data directory, as packages install it
const TUTORIAL: &str = "doc/cinq/tutorial.pdf";

impl App {
    /// Returns the state without the last failure to open a link, along
    /// with the task that runs `open` on a thread of its own, since an
    /// opener can take a moment to tell that it failed
    pub(super) fn opening(
        self,
        open: impl FnOnce() -> Result<(), String> + Send + 'static,
    ) -> (Self, Task<Message>) {
        let task = on_own_thread(open).map(Message::LinkOpened);
        let app = Self {
            problems: self.problems.with_link(None),
            ..self
        };
        (app, task)
    }
}

/// Opens a web page or a file in the program that the desktop opens it
/// with: through xdg-open on desktops that follow the XDG specifications,
/// and through the system's own opener elsewhere.
pub(super) fn open_link(target: &OsStr) -> Result<(), String> {
    open::that_detached(target)
        .map_err(|error| format!("Cannot open {}: {error}", target.to_string_lossy()))
}

/// Opens the configuration file `file` as [`open_link`] does, after making
/// it when it is not there yet, except on Darwin, which opens it in the
/// default text editor, since few programs there say they open TOML.
pub(super) fn edit_config(file: &Path) -> Result<(), String> {
    config::make(file).map_err(|error| format!("Cannot make {}: {error}", file.display()))?;
    if cfg!(target_os = "macos") {
        open_in_text_editor(file)
    } else {
        open_link(file.as_os_str())
    }
}

/// Waits for `open -t`, which tells whether there is a default text editor
fn open_in_text_editor(file: &Path) -> Result<(), String> {
    let cannot = |why: String| format!("Cannot open {}: {why}", file.display());
    let status = std::process::Command::new("/usr/bin/open")
        .args(["-t", "--"])
        .arg(file)
        .status()
        .map_err(|error| cannot(error.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(cannot(format!("open -t failed with {status}")))
    }
}

/// Opens the tutorial as [`open_link`] does, from the first of
/// [`tutorial_places`] that holds it.  A tutorial that is not installed is
/// told in one line on the help page, and the directories searched go to
/// standard error, where a long list does not crowd the page, once however
/// often the link is followed.
pub(super) fn open_tutorial() -> Result<(), String> {
    let (directories, name) = tutorial_places();
    directories
        .iter()
        .map(|directory| directory.join(name))
        .find(|tutorial| tutorial.is_file())
        .ok_or_else(|| not_found(name, &directories))
        .inspect_err(|searched| warn::once(searched))
        .map_err(|_| "The tutorial is not installed".to_owned())
        .and_then(|tutorial| open_link(tutorial.as_os_str()))
}

/// Returns the directories to look for the tutorial in, in turn, and its
/// path under each, where each platform keeps documents: beside the
/// program on Windows, where the installer package puts it on Darwin, and
/// in the data directories of the XDG Base Directory Specification
/// elsewhere, such as `/usr/share`, where the packages for GNU/Linux put it.
fn tutorial_places() -> (Vec<PathBuf>, &'static str) {
    if cfg!(windows) {
        let program = env::current_exe().ok();
        let directory = program.and_then(|program| Some(program.parent()?.to_path_buf()));
        (directory.into_iter().collect(), "tutorial.pdf")
    } else if cfg!(target_os = "macos") {
        (vec![PathBuf::from("/usr/local/share")], TUTORIAL)
    } else {
        let directories = data_directories(
            env::var_os("XDG_DATA_HOME"),
            env::var_os("XDG_DATA_DIRS").as_deref(),
            dirs::home_dir(),
        );
        (directories, TUTORIAL)
    }
}

/// Returns the data directories in the order to search them, from the
/// values of `XDG_DATA_HOME` and `XDG_DATA_DIRS`, whose paths count only
/// when they are absolute, and from the home directory, which holds the
/// directory of the user's data when `XDG_DATA_HOME` names none.
fn data_directories(
    data_home: Option<OsString>,
    data_dirs: Option<&OsStr>,
    home: Option<PathBuf>,
) -> Vec<PathBuf> {
    let user = data_home
        .map(PathBuf::from)
        .filter(|directory| directory.is_absolute())
        .or_else(|| Some(home?.join(".local/share")));
    let system: Vec<PathBuf> = data_dirs
        .into_iter()
        .flat_map(env::split_paths)
        .filter(|directory| directory.is_absolute())
        .collect();
    let system = if system.is_empty() {
        vec!["/usr/local/share".into(), "/usr/share".into()]
    } else {
        system
    };
    user.into_iter().chain(system).collect()
}

/// Tells that none of `directories` holds `name`, naming them as a
/// sentence lists things.
fn not_found(name: &str, directories: &[PathBuf]) -> String {
    let shown: Vec<String> = directories
        .iter()
        .map(|directory| directory.display().to_string())
        .collect();
    match shown.as_slice() {
        [] => format!("cannot find {name}"),
        [only] => format!("cannot find {name} in {only}"),
        [first, second] => format!("cannot find {name} in {first} or {second}"),
        [rest @ .., last] => format!("cannot find {name} in {}, or {last}", rest.join(", ")),
    }
}
