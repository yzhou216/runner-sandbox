// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the manual page of the configuration file, whose table lists
//! the keys of every action in the order of the help page

use std::iter;

use super::*;

const MANUAL: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/man/cinq.5.scd"));

/// Tells whether a line of scdoc opens a row of a table: `[`, `|`, or `]`,
/// then the alignment of its first cell, then a space
fn opens_row(line: &str) -> bool {
    matches!(
        line.as_bytes(),
        [
            b'[' | b'|' | b']',
            b'[' | b'-' | b']' | b'<' | b'=' | b'>' | b' ',
            b' ',
            ..
        ]
    )
}

/// Returns the rows of the first table of the manual, each as its cells.
/// Each cell takes a line of its own, which opens a row or, with `:`, adds
/// to it, and which holds the alignment of the cell, a space, and its text.
fn table() -> Vec<Vec<String>> {
    MANUAL
        .lines()
        .skip_while(|line| !opens_row(line))
        .take_while(|line| opens_row(line) || line.starts_with(':'))
        .fold(Vec::new(), |mut rows: Vec<Vec<String>>, line| {
            // Bold, as in the header, is no part of the text
            let cell = line.get(2..).unwrap_or_default().trim().trim_matches('*');
            match rows.last_mut() {
                Some(row) if line.starts_with(':') => row.push(cell.to_owned()),
                _ => rows.push(vec![cell.to_owned()]),
            }
            rows
        })
}

/// Returns the keys of an action as the file writes them, apart by spaces
/// where the screen joins them, as the table of the manual does
fn notated(keymap: &Keymap, action: Action) -> String {
    match keymap.keys(action) {
        [] => "none".to_owned(),
        keys => keys.iter().map(|keys| written(keys)).join(", "),
    }
}

#[test]
fn manual_lists_the_keys_of_the_keymap_in_order() {
    let keymap = Keymap::default();
    let header = vec!["Action".to_owned(), keymap.name().to_owned()];
    let actions =
        Action::listed().map(|action| vec![action.name().to_owned(), notated(&keymap, action)]);
    // Fails on the first row of man/cinq.5.scd that differs
    itertools::assert_equal(table(), iter::chain([header], actions));
}
