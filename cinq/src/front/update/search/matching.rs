// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which rows a search finds, and in what order
//!
//! A row is the texts that a search matches: those that its columns show, but
//! for the state.

#[cfg(test)]
mod tests;

/// Returns the `count` rows that follow `row` down the list, wrapping around
/// and ending on `row`, or up the list when `backward`.  Without a row they
/// begin at the top, or at the bottom going up.
fn rows_after(row: Option<usize>, count: usize, backward: bool) -> impl Iterator<Item = usize> {
    let row = row.unwrap_or(if backward { 0 } else { count.saturating_sub(1) });
    (1..=count).map(move |offset| {
        if backward {
            (row + count - offset % count) % count
        } else {
            (row + offset) % count
        }
    })
}

/// Tells whether a step from row `from` to row `to` went past the end of the
/// list, which a step to the same row does, as the only match is the one
/// that it began on.  A step with no row at either end went nowhere.
pub(super) fn wrapped(from: Option<usize>, to: Option<usize>, backward: bool) -> bool {
    from.zip(to)
        .is_some_and(|(from, to)| if backward { to >= from } else { to <= from })
}

/// Returns `text` in lowercase, with the letters that a search takes for
/// each other written alike: the Greek sigma that ends a word, the dotted and
/// dotless Turkish `i`, and the German sharp `s`.  Each character is folded
/// alone, since the lowercase of a whole string turns a capital sigma at the
/// end of the text into the final sigma, which a search for the first part of
/// a word then misses.
fn folded(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\u{3c2}' => "\u{3c3}".to_owned(),
            '\u{131}' | '\u{130}' => "i".to_owned(),
            '\u{df}' | '\u{1e9e}' => "ss".to_owned(),
            other => other.to_lowercase().collect(),
        })
        .collect()
}

/// Returns whether one of the texts of a row holds `needle`, which is
/// [`folded`], in any case
fn matches(texts: &[String], needle: &str) -> bool {
    !needle.is_empty() && texts.iter().any(|text| folded(text).contains(needle))
}

/// Returns the first row after `row` that matches `text`, wrapping around,
/// so that `row` is the last to match
pub(super) fn found(
    rows: &[Vec<String>],
    text: &str,
    row: Option<usize>,
    backward: bool,
) -> Option<usize> {
    let needle = folded(text);
    rows_after(row, rows.len(), backward).find(|&row| matches(&rows[row], &needle))
}

/// Returns how many rows hold `text`
pub(super) fn matching(rows: &[Vec<String>], text: &str) -> usize {
    let needle = folded(text);
    rows.iter().filter(|texts| matches(texts, &needle)).count()
}

/// Returns how many steps over `matches` matches end on the match that
/// `steps` steps end on, at most one lap, so that a count as large as the
/// operator can type never keeps the window waiting.  A whole number of laps
/// ends where it began, which is the last lap rather than none.
pub(super) fn within_a_lap(steps: usize, matches: usize) -> usize {
    match (steps.checked_rem(matches), steps) {
        (None | Some(0), 0) | (None, _) => 0,
        (Some(0), _) => matches,
        (Some(rest), _) => rest,
    }
}
