// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue-based show controller in the spirit of QLab
//!
//! Audio cues play sound files, video cues play video files on a stage
//! window, fade cues change the level of a playing audio or video cue, stop
//! cues end another cue, and wait and memo cues pace and annotate the show.
//! GO fires the standby cue together with every cue that auto-continue or
//! auto-follow chains to it.  Selection and standby are independent:
//! inspecting a cue never changes what the next GO fires.
//!
//! The back end in [`back`] holds the show, every rule for changing it, and
//! the audio and video output, and it does not depend on iced.  The front
//! end in [`front`] draws the show and the stage with iced and turns
//! operator input into back-end commands.  The command line in [`cli`]
//! names the show to open, and can name the directory that the key bindings
//! come from.

// Program with a window of its own, which Windows then starts without a
// console window beside it
#![cfg_attr(windows, windows_subsystem = "windows")]

mod back;
mod cli;
mod front;
// App bundle, logo, and manual pages, which only the build script lays out,
// draws, and renders, compiled here for their tests
#[cfg(test)]
mod bundle;
#[cfg(test)]
mod logo;
#[cfg(test)]
mod man;

use std::process::ExitCode;

use clap::Parser;

/// Time that cinq started at, which the diagnostic lines count from
pub static START: std::sync::LazyLock<std::time::Instant> =
    std::sync::LazyLock::new(std::time::Instant::now);

/// Diagnostic logger of the sandbox run, which writes warnings and errors
/// to stderr
struct Stderr;

impl log::Log for Stderr {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Warn
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!(
                "{:>8.3} {} {}: {}",
                START.elapsed().as_secs_f64(),
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: Stderr = Stderr;

fn main() -> ExitCode {
    std::sync::LazyLock::force(&START);
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Warn));
    let cli = cli::Cli::parse();
    match front::run(cli.config_dir, cli.show) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Reason for the failure, which iced leaves out of its own message
            let reason = match &error {
                iced::Error::ExecutorCreationFailed(reason) => reason.to_string(),
                iced::Error::WindowCreationFailed(reason) => reason.to_string(),
                iced::Error::GraphicsCreationFailed(reason) => reason.to_string(),
            };
            eprintln!("cinq: {error}: {reason}");
            ExitCode::FAILURE
        }
    }
}
