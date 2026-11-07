// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the watch writes out the problems of each reading of the file
//!
//! On a terminal, the problems of each reading take the place of those
//! before on the screen, which clears once none are left.  A file or a pipe
//! takes them in turn, as a log.  A thread of the watch's own writes them
//! out, in the order of the readings.

#[cfg(test)]
mod tests;

use std::{
    env,
    io::{self, IsTerminal, Write},
    iter,
    sync::mpsc,
    thread,
};

use crate::front::keymap::config::Config;

/// Escape sequence that moves the cursor of a terminal to the top left of
/// its screen and erases the screen
const CLEAR: &str = "\x1b[H\x1b[2J";

/// Where the watch writes out the problems of each reading of the file
pub enum Errors {
    /// Terminal, whose screen shows the problems of the latest reading alone
    Screen(Box<dyn Write + Send>),
    /// File or pipe, which takes the problems of each reading in turn
    Log(Box<dyn Write + Send>),
}

impl Errors {
    /// Returns standard error, as a screen when it is a terminal that can
    /// clear itself, which one that calls itself dumb cannot
    pub fn standard() -> Self {
        // Windows consoles set no TERM, but take escape sequences once asked
        let escapes = anstyle_query::windows::enable_ansi_colors() == Some(true)
            || env::var_os("TERM").is_some_and(|term| term != "dumb");
        if io::stderr().is_terminal() && escapes {
            Self::Screen(Box::new(io::stderr()))
        } else {
            Self::Log(Box::new(io::stderr()))
        }
    }
}

/// Starts the thread of a watch that writes out to `errors` the problems of
/// each configuration sent to it, in turn, and returns the way to it, whose
/// going ends the thread
pub(super) fn warner(errors: Errors) -> mpsc::Sender<Config> {
    let (sender, configs) = mpsc::channel();
    let screen = matches!(errors, Errors::Screen(_));
    let (Errors::Screen(mut target) | Errors::Log(mut target)) = errors;
    thread::spawn(move || {
        configs.iter().fold(false, |shown, config| {
            // Sidebar lists the problems even when standard error is gone
            let _ = target.write_all(written(&config, screen, shown).as_bytes());
            !config.problems.is_empty()
        })
    });
    sender
}

/// Returns the problems of `config` as miette draws them, after a clearing
/// of the `screen` when it has `shown` those of the reading before
fn written(config: &Config, screen: bool, shown: bool) -> String {
    let clear = if screen && shown { CLEAR } else { "" };
    let drawn = config.reports().map(|report| format!("{report:?}\n"));
    iter::once(clear.to_owned()).chain(drawn).collect()
}
