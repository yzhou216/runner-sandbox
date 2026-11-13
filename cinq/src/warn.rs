// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Warnings on standard error about what cinq skips, such as the tags of a
//! video file that cannot be read
//!
//! Files are read again while they cannot be played, so that a warning
//! comes out once however often it is given.  They go out through [`held`],
//! which keeps them for the main screen while the problems screen of
//! `config.toml` shows, since a line written on the problems screen would
//! be lost.

#[cfg(test)]
mod tests;

use std::{
    collections::BTreeSet,
    mem,
    sync::{Mutex, MutexGuard, PoisonError},
};

use crate::held;

static WARNINGS: Mutex<Warnings> = Mutex::new(Warnings {
    given: BTreeSet::new(),
});

#[derive(Default)]
struct Warnings {
    /// Lines that went out or wait
    given: BTreeSet<String>,
}

impl Warnings {
    /// Returns the state after `warning` is given, and what to write now:
    /// its line, unless it was given before
    fn warned(self, warning: &str) -> (Self, String) {
        let line = format!("cinq: {warning}\n");
        if self.given.contains(&line) {
            (self, String::new())
        } else {
            let given = self.given.into_iter().chain([line.clone()]).collect();
            (Self { given }, line)
        }
    }
}

/// Writes a line on standard error that says `warning` after `cinq: `,
/// unless it did before
pub fn once(warning: &str) {
    let mut warnings = locked();
    let (next, line) = mem::take(&mut *warnings).warned(warning);
    *warnings = next;
    held::write(line.as_bytes());
}

/// Locks [`WARNINGS`], which a thread that panicked while it held the lock
/// leaves as it was
fn locked() -> MutexGuard<'static, Warnings> {
    WARNINGS.lock().unwrap_or_else(PoisonError::into_inner)
}
