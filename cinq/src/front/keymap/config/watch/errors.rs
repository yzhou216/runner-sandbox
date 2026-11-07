// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the watch writes out the problems of each reading of the file
//!
//! On a terminal, the problems of the latest reading stay at the bottom, in
//! place of those before, below the lines of the log, until none are left.
//! A file or a pipe takes them in turn, as a log.  A thread of the watch's
//! own writes them out, in the order of the readings.

#[cfg(test)]
mod tests;

use std::{
    env,
    io::{self, IsTerminal, Write},
    sync::{LazyLock, mpsc},
    thread,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::front::keymap::config::Config;

/// Standard error, where the log writes above the problems that a terminal
/// shows
pub static TERMINAL: LazyLock<MultiProgress> = LazyLock::new(MultiProgress::new);

/// Where the watch writes out the problems of each reading of the file
pub enum Errors {
    /// Terminal, which shows the problems of the latest reading below all
    /// else written to it
    Screen(MultiProgress),
    /// File or pipe, which takes the problems of each reading in turn
    Log(Box<dyn Write + Send>),
}

impl Errors {
    /// Returns standard error, as a screen when it is a terminal that can
    /// move its cursor, which one that calls itself dumb cannot
    pub fn standard() -> Self {
        // Windows consoles set no TERM, but take escape sequences once asked
        let escapes = anstyle_query::windows::enable_ansi_colors() == Some(true)
            || env::var_os("TERM").is_some_and(|term| term != "dumb");
        if io::stderr().is_terminal() && escapes {
            Self::Screen(TERMINAL.clone())
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
    thread::spawn(move || match errors {
        Errors::Screen(terminal) => {
            let last = configs
                .iter()
                .fold(None, |bar, config| shown(&terminal, bar, &config));
            // Problems left in the file stay in sight once the program ends
            if let Some(bar) = last {
                bar.abandon();
            }
        }
        // Sidebar lists the problems even when standard error is gone
        Errors::Log(mut target) => configs.iter().for_each(|config| {
            let _ = target.write_all(written(&config).as_bytes());
        }),
    });
    sender
}

/// Shows the problems of `config` at the bottom of `terminal`, in place of
/// those that `bar` shows, and returns the bar that shows them, or none once
/// none are left
fn shown(
    terminal: &MultiProgress,
    bar: Option<ProgressBar>,
    config: &Config,
) -> Option<ProgressBar> {
    let drawn = written(config);
    if drawn.is_empty() {
        if let Some(bar) = bar {
            bar.finish_and_clear();
            terminal.remove(&bar);
        }
        return None;
    }
    let bar = bar.unwrap_or_else(|| {
        let style = ProgressStyle::with_template("{msg}").expect("{msg} is a template");
        terminal.add(ProgressBar::new_spinner().with_style(style))
    });
    bar.set_message(drawn.trim_end().to_owned());
    Some(bar)
}

/// Returns the problems of `config` as miette draws them
fn written(config: &Config) -> String {
    config
        .reports()
        .map(|report| format!("{report:?}\n"))
        .collect()
}
