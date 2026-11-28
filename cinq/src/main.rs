// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Cue-based show controller in the spirit of QLab
//!
//! Audio cues play sound files, video cues play video files on a stage
//! window, where picture cues show picture files, light cues fade the
//! instruments of a light patch, fade cues change the level of a playing
//! audio or video cue and fade out the pictures of the cues that they stop,
//! start cues start another cue and pause cues pause it, devamp cues let a
//! looping cue leave its loop, stop cues end another cue, GoTo cues move
//! standby to another cue, group cues fire the cues that they hold all at
//! once, network cues send OSC messages, and wait and memo cues pace and
//! annotate the show.
//! GO fires the standby cue together with every cue that auto-continue or
//! auto-follow chains to it.  Selection and standby are independent:
//! inspecting a cue never changes what the next GO fires.
//!
//! The back end in [`back`] holds the show, every rule for changing it, and
//! the audio, video, light, and network output, and it does not depend on
//! iced.  The front end in [`front`] draws the show and the stage with iced
//! and turns operator input into back-end commands.  The command line in
//! [`cli`] names the show to open, and can name the directory that
//! config.toml comes from, or asks for the list of audio devices instead.

mod back;
mod cli;
mod front;
mod held;
mod lock;
mod warn;
// App bundle, logo, and manual pages, which only the build script lays out,
// draws, and renders, compiled here for their tests
#[cfg(test)]
mod bundle;
#[cfg(test)]
mod logo;
#[cfg(test)]
mod man;

use std::{
    io::{self, Write},
    process::ExitCode,
};

use clap::Parser;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    if cli.audio_devices {
        return audio_devices();
    }
    match front::run(cli.config_dir, cli.show) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Reason for the failure, which iced leaves out of its own message
            let reason = match &error {
                iced::Error::ExecutorCreationFailed(reason) => reason.to_string(),
                iced::Error::WindowCreationFailed(reason) => reason.to_string(),
                iced::Error::GraphicsCreationFailed(reason) => reason.to_string(),
            };
            failed(&format!("{error}: {reason}"))
        }
    }
}

/// Writes `message` on standard error and returns the status of a failure.
/// A write that fails, as to a closed pipe or a full disk, leaves the status
/// as it is, where `eprintln!` would panic and end with another.
fn failed(message: &str) -> ExitCode {
    let _ = writeln!(io::stderr(), "cinq: {message}");
    ExitCode::FAILURE
}

/// Lists the ID and the name of each audio device on standard output, one
/// device to a line
fn audio_devices() -> ExitCode {
    let listed = back::audio_devices().and_then(|devices| {
        devices
            .iter()
            .try_for_each(|(id, name)| writeln!(io::stdout(), "{id}\t{name}"))
            // Reader that stops early, as head does, is no failure
            .or_else(|error| match error.kind() {
                io::ErrorKind::BrokenPipe => Ok(()),
                _ => Err(error),
            })
            .map_err(|error| error.to_string())
    });
    match listed {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => failed(&format!("cannot list the audio devices: {error}")),
    }
}
