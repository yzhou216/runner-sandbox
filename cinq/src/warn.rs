// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Warnings on standard error about what cinq skips, such as the tags of a
//! video file that cannot be read
//!
//! Files are read again while they cannot be played, so that a warning
//! comes out once however often it is given.  Warnings that anyone on the
//! network can make distinct, as an OSC address can, would otherwise fill the
//! terminal and the memory, so that after [`LIMIT`] distinct lines one last
//! line says that further warnings go unreported, and nothing more is written
//! or kept.  They go out through [`held`], which keeps them for the main
//! screen while the problems screen of `config.toml` shows, since a line
//! written on the problems screen would be lost.

#[cfg(test)]
mod tests;

use std::{collections::BTreeSet, io, mem, sync::Mutex};

use crate::{held, lock::lock};

/// Text of the kind that a system error without a kind of its own has
const UNCATEGORIZED: &str = "uncategorized error";

const LIMIT: usize = 1000;

static WARNINGS: Mutex<Warnings> = Mutex::new(Warnings {
    given: BTreeSet::new(),
    silenced: false,
});

#[derive(Default)]
struct Warnings {
    /// Lines that went out or wait
    given: BTreeSet<String>,
    /// Whether the line that ends the warnings went out
    silenced: bool,
}

impl Warnings {
    /// Returns the state after `warning` is given, and what to write now:
    /// its line, unless it was given before or [`LIMIT`] lines went out, in
    /// which case the first warning past it gives the line that ends them
    fn warned(self, warning: &str) -> (Self, String) {
        let line = format!("cinq: {warning}\n");
        if self.silenced || self.given.contains(&line) {
            (self, String::new())
        } else if self.given.len() >= LIMIT {
            let last = "cinq: further warnings go unreported\n".to_owned();
            (
                Self {
                    silenced: true,
                    ..self
                },
                last,
            )
        } else {
            // Inserted in place, so that a warning costs no more than a
            // search of the set
            let mut given = self.given;
            given.insert(line.clone());
            (Self { given, ..self }, line)
        }
    }
}

/// Writes a line on standard error that says `warning` after `cinq: `,
/// unless it did before
pub fn once(warning: &str) {
    let mut warnings = lock(&WARNINGS);
    let (next, line) = mem::take(&mut *warnings).warned(warning);
    *warnings = next;
    held::write(line.as_bytes());
}

/// Returns the words of `error` for the screen: the text of its kind, as
/// `permission denied`, where the operating system gave it, since its own
/// message adds the error number, and otherwise its own message, since an
/// error of cinq's makes the kind `uncategorized error` or too general.  An
/// error of the system that has no kind of its own, as a missing device or
/// an unsupported address family has not, gives the message of the system
/// without its number, in lower case.
pub fn words(error: &io::Error) -> String {
    let kind = error.kind().to_string();
    match error.raw_os_error() {
        None => error.to_string(),
        Some(_) if kind != UNCATEGORIZED => kind,
        Some(_) => {
            let message = error.to_string();
            let message = message
                .rsplit_once(" (os error ")
                .map_or(message.as_str(), |(message, _)| message)
                .trim_end_matches('.');
            let mut letters = message.chars();
            letters
                .next()
                .map(|first| first.to_lowercase().chain(letters).collect())
                .unwrap_or_default()
        }
    }
}
