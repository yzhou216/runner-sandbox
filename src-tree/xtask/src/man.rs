// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Manual pages, rendered from the definitions the daemon uses.

mod command_page;
mod config_page;
mod date;
mod roff_ext;

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use scrollreversed::config;

/// Writes scrollreversed(1) and scrollreversed(5) into `dir`.
pub fn write(root: &Path, dir: &Path) -> io::Result<Vec<PathBuf>> {
    let config_path = config::documented_path()
        .ok_or_else(|| io::Error::other("cannot find the home directory"))?;
    let date = date::release_date(root);
    fs::create_dir_all(dir)?;
    [
        (
            "scrollreversed.1",
            command_page::render(&config_path, &date)?,
        ),
        ("scrollreversed.5", config_page::render(&config_path, &date)),
    ]
    .into_iter()
    .map(|(name, roff)| {
        let path = dir.join(name);
        fs::write(&path, roff).map(|()| path)
    })
    .collect()
}
