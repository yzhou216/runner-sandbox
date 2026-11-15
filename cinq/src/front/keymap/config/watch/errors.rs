// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the watch writes out the problems of each reading of the file
//!
//! On a terminal, the problems take a screen of their own while any are
//! left, the alternate screen that full-screen programs use.  Each reading
//! draws its problems there in place of those before, and the first reading
//! without any switches back to the screen as it was, prompt and all.
//! Problems left when the program ends come back to that screen once, so
//! that they stay in sight.  A file or a pipe takes the problems of each
//! reading in turn, as a log.  A thread of the watch's own writes them out,
//! in the order of the readings.  Whatever else is written to standard
//! output and error while the problems show, by cinq and by the libraries
//! it uses, waits in [`held`] for the main screen and comes out there once
//! the problems clear, or before those left at the end.  The problems
//! screen itself draws on the terminal that [`held`] keeps apart.

mod look;
#[cfg(test)]
mod tests;

use std::{
    env,
    io::{self, IsTerminal, Write},
    iter, panic, process,
    sync::{
        Mutex, MutexGuard, PoisonError, TryLockError,
        mpsc::{self, RecvTimeoutError},
    },
    thread,
    time::Duration,
};

use terminal_size::terminal_size_of;

use crate::{front::keymap::config::Config, held};

/// Escape sequence that switches a terminal to its alternate screen
const ENTER: &str = "\x1b[?1049h";

/// Escape sequence that switches a terminal back to its main screen, which
/// shows what it showed before
const LEAVE: &str = "\x1b[?1049l";

/// Escape sequence that moves the cursor of a terminal to the top left of
/// its screen and erases the screen
const CLEAR: &str = "\x1b[H\x1b[2J";

/// Time between looks at the size of the terminal while the problems show
const RESIZE: Duration = Duration::from_millis(250);

/// Configuration whose problems the alternate screen of standard error
/// shows, while it does
static SHOWN: Mutex<Option<Config>> = Mutex::new(None);

pub enum Errors {
    /// Terminal on standard error
    Screen,
    /// File or pipe
    Log(Box<dyn Write + Send>),
}

impl Errors {
    pub fn standard() -> Self {
        if switchable() {
            look::install();
            Self::Screen
        } else {
            Self::Log(Box::new(io::stderr()))
        }
    }
}

/// Makes the program switch standard error back from the problems screen
/// however it ends: on a panic, on Control+c, and on the signals that end
/// a program
pub fn guard() {
    if !switchable() {
        return;
    }
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        // Panics on other threads, such as those that decode video, leave
        // the program running
        if thread::current().name() == Some("main") {
            match SHOWN.try_lock() {
                Ok(mut shown) => restored(&mut shown),
                Err(TryLockError::Poisoned(poisoned)) => restored(&mut poisoned.into_inner()),
                // Thread that panics while it holds the lock would wait for
                // itself
                Err(TryLockError::WouldBlock) => {}
            }
        }
        hook(info);
    }));
    // Only a handler set before makes this fail, and none is
    let _ = ctrlc::set_handler(|| {
        restore();
        process::exit(130);
    });
}

/// Switches standard error back from the problems screen, if it shows, and
/// writes out the problems left once
pub fn restore() {
    restored(&mut shown());
}

/// Starts the thread of a watch that writes out to `errors` the problems of
/// each configuration sent to it, in turn, and returns the way to it, whose
/// going ends the thread
pub(super) fn warner(errors: Errors) -> mpsc::Sender<Config> {
    let (sender, configs) = mpsc::channel();
    thread::spawn(move || match errors {
        Errors::Screen => screen(&configs),
        // Sidebar lists the problems even when standard error is gone
        Errors::Log(mut target) => configs.iter().for_each(|config| {
            let _ = target.write_all(written(&config).as_bytes());
        }),
    });
    sender
}

/// Draws the problems of each configuration from `configs` on the problems
/// screen of standard error, and draws them again when the terminal changes
/// size while they show
fn screen(configs: &mpsc::Receiver<Config>) {
    let size = || {
        held::terminal()
            .ok()
            .and_then(|terminal| terminal_size_of(&terminal))
    };
    // Readings, and while the problems show, a pause now and then without one
    let readings = iter::from_fn(|| {
        let reading = if shown().is_some() {
            configs.recv_timeout(RESIZE)
        } else {
            configs.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };
        match reading {
            Err(RecvTimeoutError::Disconnected) => None,
            reading => Some(reading.ok()),
        }
    });
    readings.fold(size(), |seen, reading| {
        let now = size();
        let mut shown = shown();
        let resized = shown.as_ref().filter(|_| now != seen).cloned();
        if let Some(config) = reading.or(resized) {
            let problems = !config.problems.is_empty();
            if problems {
                held::hold();
            }
            let _ = held::terminal().and_then(|mut terminal| {
                terminal.write_all(switched(shown.is_some(), &config).as_bytes())
            });
            *shown = problems.then_some(config);
            if !problems {
                held::release();
            }
        }
        now
    });
}

/// Returns what takes a terminal from the problems screen, when it is
/// `showing`, or from its main screen, to the problems of `config`: their
/// own screen while there are any, and the main screen once there are none
fn switched(showing: bool, config: &Config) -> String {
    let drawn = written(config);
    match (showing, drawn.is_empty()) {
        (false, true) => String::new(),
        (true, true) => LEAVE.to_owned(),
        (false, false) => format!("{ENTER}{CLEAR}{drawn}"),
        (true, false) => format!("{CLEAR}{drawn}"),
    }
}

/// Returns the problems of `config` as miette draws them
fn written(config: &Config) -> String {
    config
        .reports()
        .map(|report| format!("{report:?}\n"))
        .collect()
}

/// Switches the terminal back from the problems screen that shows those of
/// `config`, then writes what was held meanwhile on the main screen, and
/// last the problems, so that they stay in sight
fn restored(shown: &mut Option<Config>) {
    if let Some(config) = shown.take() {
        let _ = held::terminal().and_then(|mut terminal| terminal.write_all(LEAVE.as_bytes()));
        held::release();
        let _ = io::stderr().write_all(written(&config).as_bytes());
    }
}

/// Returns whether standard error is a terminal that can switch screens,
/// which one that calls itself dumb cannot
fn switchable() -> bool {
    // Windows consoles set no TERM, but take escape sequences once asked
    let escapes = anstyle_query::windows::enable_ansi_colors() == Some(true)
        || env::var_os("TERM").is_some_and(|term| term != "dumb");
    held::terminal().is_ok_and(|terminal| terminal.is_terminal()) && escapes
}

/// Locks [`SHOWN`], which a thread that panicked while it held the lock
/// leaves as it was
fn shown() -> MutexGuard<'static, Option<Config>> {
    SHOWN.lock().unwrap_or_else(PoisonError::into_inner)
}
