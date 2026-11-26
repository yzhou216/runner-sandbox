// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::catalog::{self, Catalog};

/// Summary of `--hazards`, the first line of its help.
const PATHS_SUMMARY: &str = "Load hazards from a Scheme file or a directory of .scm files";

/// Locations of hazard definitions, shared by the commands that use them.
#[derive(clap::Args)]
#[group(skip)]
pub struct Args {
    #[arg(long = "hazards", value_name = "PATH", help = PATHS_SUMMARY, long_help = paths_help())]
    paths: Vec<PathBuf>,
}

impl Args {
    pub fn load(&self) -> Result<Catalog> {
        let user_directory = catalog::user_directory().filter(|directory| directory.is_dir());
        user_directory
            .iter()
            .chain(&self.paths)
            .try_fold(Catalog::builtin()?, |catalog, path| catalog.with_path(path))
    }
}

/// Help for `--hazards`, naming the user directory of this system.
fn paths_help() -> String {
    let user_directory = catalog::user_directory()
        .as_deref()
        .map_or("the user directory".into(), with_tilde);
    format!(
        "{PATHS_SUMMARY}\n\n\
         Built-in hazards load first, then those in {user_directory}, and then \
         those at these paths.  A hazard replaces any loaded earlier under the \
         same name."
    )
}

/// `path` with the home directory written as `~`, so that the help reads the
/// same for every user, as do the manual pages generated from it at build
/// time.
fn with_tilde(path: &Path) -> String {
    dirs::home_dir()
        .and_then(|home| path.strip_prefix(home).ok())
        .map_or_else(
            || path.display().to_string(),
            |relative| Path::new("~").join(relative).display().to_string(),
        )
}
