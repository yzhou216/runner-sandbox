// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Darwin daemon reversing mouse or trackpad scrolling

use std::process::ExitCode;

use clap::Parser;
use scrollreversed::{cli::Args, config, reverser::Reverser};

fn main() -> ExitCode {
    let args = Args::parse();
    match config::load(args.config()) {
        Ok(file) => run(Reverser::default().with(file).with(args.settings())),
        Err(error) => {
            eprintln!("scrollreversed: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_os = "macos")]
fn run(reverser: Reverser) -> ExitCode {
    let Err(error) = reverser.run();
    eprintln!("scrollreversed: {error}");
    ExitCode::FAILURE
}

#[cfg(not(target_os = "macos"))]
fn run(_: Reverser) -> ExitCode {
    eprintln!("scrollreversed: only Darwin is supported");
    ExitCode::FAILURE
}
