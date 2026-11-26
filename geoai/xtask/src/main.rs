// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fs, io, iter,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use clap_mangen::Man;
use geoai::cli::Cli;

/// Development tasks of geoai, run as `cargo xtask`
#[derive(Parser)]
enum Task {
    /// Write a manual page for geoai and for each of its commands into
    /// DIRECTORY
    Man { directory: PathBuf },
}

fn main() -> Result<()> {
    match Task::parse() {
        Task::Man { directory } => write_manual(&directory)
            .with_context(|| format!("cannot write manual pages into {}", directory.display())),
    }
}

/// Write the pages into `directory`, each naming geoai and its version in the
/// footer.  The version leaves out the license notice of `--version`, which
/// the page of geoai already shows after the help.
fn write_manual(directory: &Path) -> io::Result<()> {
    let mut geoai = Cli::command()
        .long_version(None)
        .disable_help_subcommand(true);
    geoai.build();
    let source = format!("geoai {}", geoai.get_version().unwrap_or_default());
    fs::create_dir_all(directory)?;
    iter::once(&geoai)
        .chain(geoai.get_subcommands())
        .filter(|command| !command.is_hide_set())
        .try_for_each(|command| {
            Man::new(command.clone())
                .source(&source)
                .generate_to(directory)
                .map(drop)
        })
}
