// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Output kept back while the problems screen of `config.toml` shows
//!
//! The problems screen takes the alternate screen of a terminal, and what
//! reaches the terminal meanwhile is lost when it switches back.  From
//! [`hold`] to [`release`], what cinq writes with [`write`] goes to a buffer
//! in memory instead.  On POSIX systems, so does what anything else writes
//! to standard output and error, such as the libraries for sound and
//! graphics, since [`hold`] points the descriptors at a pipe; on Windows,
//! only what [`write`] takes is held.  [`release`] writes the buffer on the
//! main screen, once the problems clear or when the program ends.  The
//! buffer keeps the newest megabyte of output and [`release`] says how many
//! bytes before it were dropped.  The problems screen itself draws on
//! [`terminal`], which stays the real one meanwhile.

mod buffer;
#[cfg(unix)]
mod redirect;
#[cfg(test)]
mod tests;

#[cfg(unix)]
use std::{fs::File, os::fd::AsFd};
use std::{
    io::{self, Write},
    mem,
    sync::{Mutex, MutexGuard, PoisonError},
};

use buffer::Buffer;
#[cfg(unix)]
use redirect::Redirect;

/// Terminal on standard error, the real one while output is held
#[cfg(unix)]
pub type Terminal = File;

/// Terminal on standard error, the real one while output is held
#[cfg(not(unix))]
pub type Terminal = io::Stderr;

static HELD: Mutex<Held> = Mutex::new(Held {
    holding: false,
    #[cfg(unix)]
    redirect: None,
});

/// Bytes that wait for the main screen, which the thread of the redirect
/// fills too and so cannot sit with the state of [`HELD`]
static BUFFER: Mutex<Buffer> = Mutex::new(Buffer::new());

#[derive(Default)]
struct Held {
    /// Whether output waits for the main screen
    holding: bool,
    /// Descriptors that point at the buffer, when they could
    #[cfg(unix)]
    redirect: Option<Redirect>,
}

impl Held {
    fn hold(self) -> Self {
        if self.holding {
            return self;
        }
        Self {
            holding: true,
            #[cfg(unix)]
            redirect: Redirect::start(retain).ok(),
        }
    }

    /// Returns the state once the main screen shows again, and the bytes
    /// that waited for it
    fn released(self) -> (Self, Vec<u8>) {
        #[cfg(unix)]
        if let Some(redirect) = self.redirect {
            redirect.stop();
        }
        (Self::default(), mem::take(&mut *buffer()).written())
    }
}

/// Writes `bytes` on standard error, or holds them until [`release`] while
/// output is held
pub fn write(bytes: &[u8]) {
    apply(|held| {
        if held.holding {
            retain(bytes);
            (held, Vec::new())
        } else {
            (held, bytes.to_vec())
        }
    });
}

/// Holds output, from before the problems screen takes the terminal
pub fn hold() {
    apply(|held| (held.hold(), Vec::new()));
}

/// Writes the held output on standard error, once the main screen shows
/// again, and stops holding
pub fn release() {
    apply(Held::released);
}

/// Returns standard error as it was before [`hold`], for the problems screen
/// to draw on and to ask about the terminal
#[cfg(unix)]
pub fn terminal() -> io::Result<Terminal> {
    match &locked().redirect {
        Some(redirect) => redirect.terminal(),
        None => io::stderr().as_fd().try_clone_to_owned().map(File::from),
    }
}

/// Returns standard error as it was before [`hold`], for the problems screen
/// to draw on and to ask about the terminal
#[cfg(not(unix))]
pub fn terminal() -> io::Result<Terminal> {
    Ok(io::stderr())
}

/// Steps the state and writes what the step returns while the lock is held,
/// so that no line comes between a switch of screens and the hold or release
/// that goes with it
fn apply(step: impl FnOnce(Held) -> (Held, Vec<u8>)) {
    let mut held = locked();
    let (next, bytes) = step(mem::take(&mut *held));
    *held = next;
    let _ = io::stderr().write_all(&bytes);
}

/// Adds `bytes` to the buffer
fn retain(bytes: &[u8]) {
    let mut buffer = buffer();
    *buffer = mem::take(&mut *buffer).pushed(bytes);
}

/// Locks [`HELD`], which a thread that panicked while it held the lock leaves
/// as it was
fn locked() -> MutexGuard<'static, Held> {
    HELD.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Locks [`BUFFER`], which a thread that panicked while it held the lock
/// leaves as it was
fn buffer() -> MutexGuard<'static, Buffer> {
    BUFFER.lock().unwrap_or_else(PoisonError::into_inner)
}
