// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Warnings on standard error about what cinq skips, such as the tags of a
//! video file that cannot be read
//!
//! Files are read again while they cannot be played, so that a warning
//! comes out once however often it is given.  While the problems screen of
//! `config.toml` shows, a warning waits for the main screen, since a line
//! written on the problems screen would corrupt it.

#[cfg(test)]
mod tests;

use std::{
    collections::BTreeSet,
    io::{self, Write},
    iter, mem,
    sync::{Mutex, MutexGuard, PoisonError},
};

static WARNINGS: Mutex<Warnings> = Mutex::new(Warnings {
    held: false,
    waiting: Vec::new(),
    given: BTreeSet::new(),
});

#[derive(Default)]
struct Warnings {
    /// Whether the problems screen shows
    held: bool,
    /// Lines that wait for the main screen, in the order they came
    waiting: Vec<String>,
    /// Lines that were written or wait
    given: BTreeSet<String>,
}

impl Warnings {
    /// Returns the state after `warning` is given, and what to write now:
    /// its line, unless it was given before or the problems screen shows
    fn warned(self, warning: &str) -> (Self, String) {
        let line = format!("cinq: {warning}\n");
        if self.given.contains(&line) {
            return (self, String::new());
        }
        let given = iter::once(line.clone()).chain(self.given).collect();
        if self.held {
            let waiting = self.waiting.into_iter().chain([line]).collect();
            (
                Self {
                    waiting,
                    given,
                    ..self
                },
                String::new(),
            )
        } else {
            (Self { given, ..self }, line)
        }
    }

    fn holding(self) -> (Self, String) {
        (Self { held: true, ..self }, String::new())
    }

    /// Returns the state once the main screen shows again, and the lines
    /// that waited for it
    fn released(self) -> (Self, String) {
        let waited = self.waiting.concat();
        let state = Self {
            held: false,
            waiting: Vec::new(),
            ..self
        };
        (state, waited)
    }
}

/// Writes a line on standard error that says `warning` after `cinq: `,
/// unless it did before
pub fn once(warning: &str) {
    apply(|warnings| warnings.warned(warning));
}

/// Makes warnings wait, from before the problems screen takes standard
/// error
pub fn hold() {
    apply(Warnings::holding);
}

/// Writes the warnings that waited, once the main screen shows again
pub fn release() {
    apply(Warnings::released);
}

/// Steps the warnings and writes what the step returns while the lock is
/// held, so that no line comes between a switch of screens and the hold or
/// release that goes with it
fn apply(step: impl FnOnce(Warnings) -> (Warnings, String)) {
    let mut warnings = locked();
    let (next, text) = step(mem::take(&mut *warnings));
    *warnings = next;
    let _ = io::stderr().write_all(text.as_bytes());
}

/// Locks [`WARNINGS`], which a thread that panicked while it held the lock
/// leaves as it was
fn locked() -> MutexGuard<'static, Warnings> {
    WARNINGS.lock().unwrap_or_else(PoisonError::into_inner)
}
