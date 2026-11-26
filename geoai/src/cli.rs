// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod hazards;
pub mod info;
pub mod list;
pub mod run;

use clap::{Parser, Subcommand};
use itertools::Itertools;

use crate::storage::Location;

/// Copyright and license notice that the AGPL recommends attaching to a
/// program.  Its opening line naming the program is left out, since clap
/// prints the name above it.
const NOTICE: &str = "\
Copyright (C) 2026  Yiyu Zhou <yiyu@yiyuzhou.io>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.";

/// Geospatial hazard processing
#[derive(Parser)]
#[command(version, long_version = long_version(), after_help = NOTICE)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Version followed by the notice, as printed by `--version`.
fn long_version() -> String {
    format!("{}\n{NOTICE}", env!("CARGO_PKG_VERSION"))
}

#[derive(Subcommand)]
pub enum Command {
    /// Print metadata and statistics of a GeoTIFF
    Info(info::Args),
    /// List the available hazards
    List(list::Args),
    /// Compute a hazard from its inputs and write it as a GeoTIFF
    Run(run::Args),
}

/// `error` followed by its causes, leaving out any cause that only repeats
/// the end of the message before it, as the errors of the tiff crate do.
pub fn message(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(ToString::to_string)
        .dedup_by(|previous, cause| previous.ends_with(cause.as_str()))
        .join(": ")
}

/// Location written as `text` in an argument, with the causes of any error,
/// which clap would otherwise leave out.
fn location(text: &str) -> Result<Location, String> {
    Location::parse(text).map_err(|error| message(&error))
}
