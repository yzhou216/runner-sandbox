// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the watch writes out the problems of each reading of the file
//!
//! On a terminal, the problems of each reading take the place of those
//! before on the screen, so that the screen keeps showing the problems of
//! the file as it stands while it is edited, and once none are left, a line
//! says so.  A file or a pipe takes the problems of each reading after those
//! of the reading before, as a log.  A thread of the watch's own does the
//! writing, in the order of the readings.

#[cfg(test)]
mod tests;

use std::{
    env,
    io::{self, IsTerminal, Write},
    sync::mpsc,
    thread,
};

use crate::front::keymap::config::Config;

/// Escape sequence that moves the cursor of a terminal to the top left of
/// its screen and erases the screen
const CLEAR: &str = "\x1b[H\x1b[2J";

/// Line that tells that the problems told before are gone
const NONE_LEFT: &str = "config.toml: no problems\n";

/// Where the watch writes out the problems of each reading of the file
pub enum Errors {
    /// Terminal, whose screen shows the problems of the latest reading in
    /// place of those before
    Screen(Box<dyn Write + Send>),
    /// File or pipe, which takes the problems of each reading after those
    /// of the reading before
    Log(Box<dyn Write + Send>),
}

impl Errors {
    /// Returns standard error: a screen when it is a terminal that can
    /// clear itself, which one that calls itself dumb cannot, as the shell
    /// of Emacs does, and a log otherwise
    pub fn standard() -> Self {
        let screen =
            io::stderr().is_terminal() && env::var_os("TERM").is_some_and(|term| term != "dumb");
        if screen {
            Errors::Screen(Box::new(io::stderr()))
        } else {
            Errors::Log(Box::new(io::stderr()))
        }
    }
}

/// Starts the thread of a watch that writes out to `errors` the problems of
/// each configuration sent to it, in the order sent, and returns the way to
/// it, whose going ends the thread
pub(super) fn warner(errors: Errors) -> mpsc::Sender<Config> {
    let (sender, configs) = mpsc::channel();
    let (screen, mut target) = match errors {
        Errors::Screen(target) => (true, target),
        Errors::Log(target) => (false, target),
    };
    thread::spawn(move || {
        configs.iter().fold(Told::Nothing, |told, config| {
            let (text, told) = written(&config, told, screen);
            // Sidebar lists the problems all the same, so a standard error
            // that cannot be written to, such as a closed terminal, is no
            // reason to stop
            let _ = target.write_all(text.as_bytes());
            told
        })
    });
    sender
}

/// What the problems written out last told of the file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Told {
    Nothing,
    Problems,
    NoProblems,
}

/// Returns what to write out for the problems of `config` after `told`,
/// along with what it tells: the problems as miette draws them, or once
/// problems were told, a line that says none are left.  On a screen, the
/// text clears what was written before.
fn written(config: &Config, told: Told, screen: bool) -> (String, Told) {
    let drawn: String = config
        .reports()
        .map(|report| format!("{report:?}\n"))
        .collect();
    let (text, now) = match (drawn.is_empty(), told) {
        (false, _) => (drawn, Told::Problems),
        (true, Told::Problems) => (NONE_LEFT.to_owned(), Told::NoProblems),
        (true, _) => (String::new(), told),
    };
    let clear = screen && told != Told::Nothing && !text.is_empty();
    let start = if clear { CLEAR } else { "" };
    (start.to_owned() + &text, now)
}
