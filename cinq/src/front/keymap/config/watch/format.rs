// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Formatting of the file after it changes, as taplo formats TOML
//!
//! Formatting follows taplo's defaults, which even out spaces and blank
//! lines, line up trailing comments, and fold arrays to fit 80 columns, but
//! never reorders keys or the items of arrays and inline tables, and the
//! line endings stay as they were.  A file that is not TOML stays as it is.

#[cfg(test)]
mod tests;

use std::{fs, path::Path};

use taplo::{
    formatter::{self, Options},
    parser,
};

/// Formats `file` in place, when it is TOML that formatting changes
pub(super) fn reformat(file: &Path) {
    let Ok(text) = fs::read_to_string(file) else {
        return;
    };
    // Save that came in while formatting is newer than the text formatted
    if let Some(formatted) = formatted(&text)
        && fs::read_to_string(file).is_ok_and(|now| now == text)
    {
        // File that cannot be written, as in a read-only Nix store, stays
        let _ = fs::write(file, formatted);
    }
}

/// Returns `text` formatted, or nothing when it is not TOML or is formatted
/// already
fn formatted(text: &str) -> Option<String> {
    let parsed = parser::parse(text);
    // Order that the file was written in stays, whatever taplo's defaults
    let options = Options {
        reorder_keys: false,
        reorder_arrays: false,
        reorder_inline_tables: false,
        crlf: text.contains("\r\n"),
        ..Options::default()
    };
    let formatted = parsed
        .errors
        .is_empty()
        .then(|| formatter::format_syntax(parsed.into_syntax(), options))?;
    (formatted != text).then_some(formatted)
}
