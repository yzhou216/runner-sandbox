// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{env, process::ExitCode};

use clap::Parser;
use geoai::cli::{Cli, Command, info, list, message, run};

fn main() -> ExitCode {
    keep_steel_home_in_cache();
    match Cli::parse().command {
        Command::Info(args) => info::run(&args),
        Command::List(args) => list::run(&args),
        Command::Run(args) => run::run(&args),
    }
    .map_or_else(
        |error| {
            eprintln!("error: {}", message(&error));
            ExitCode::FAILURE
        },
        |()| ExitCode::SUCCESS,
    )
}

/// Point Steel, unless the user already has, at a directory in geoai's cache.
/// Steel would otherwise create a home of its own among the data of the user,
/// and warn on every run where it cannot.  Hazard files therefore cannot
/// `require` packages installed for Steel itself, only other files by path.
fn keep_steel_home_in_cache() {
    if let (None, Some(cache)) = (env::var_os("STEEL_HOME"), dirs::cache_dir()) {
        // SAFETY: no other thread exists yet to read the environment
        unsafe { env::set_var("STEEL_HOME", cache.join("geoai").join("steel")) };
    }
}
