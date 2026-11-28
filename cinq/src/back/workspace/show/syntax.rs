// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a TOML error lies in the text of a show file

use itertools::Itertools;

/// Returns what keeps `text` from being TOML or a show in one line, led by
/// where it lies, since the drawing of the line and the caret that the error
/// holds would not fit where the problem is told
pub(super) fn read_problem(text: &str, error: &toml::de::Error) -> String {
    let message = error.message().lines().join(" ");
    match error.span() {
        Some(span) => {
            let before = text.get(..span.start).unwrap_or(text);
            let line = before.matches('\n').count() + 1;
            let column = before
                .rsplit('\n')
                .next()
                .map_or(0, |row| row.chars().count())
                + 1;
            let place = table_place(text, span.start)
                .map(|place| format!("{place}: "))
                .unwrap_or_default();
            format!("line {line}, column {column}: {place}{message}")
        }
        None => message,
    }
}

/// Returns which output the error at byte `start` of `text` lies in, as
/// `output 2`, where it lies at the head of an `[[output]]` table, as a
/// field that the table lacks does, so that a show with twenty outputs
/// names the one
fn table_place(text: &str, start: usize) -> Option<String> {
    const HEAD: &str = "[[output]]";
    text.get(start..)?.starts_with(HEAD).then(|| {
        let before = text.get(..start).unwrap_or_default();
        // Head may carry a comment after it
        let earlier = before
            .lines()
            .filter(|line| line.trim_start().starts_with(HEAD))
            .count();
        format!("output {}", earlier + 1)
    })
}
