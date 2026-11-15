// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which cues a search finds, and in what order

use crate::back::Cue;

/// Returns the `count` rows that follow `row` down the list, wrapping around
/// and ending on `row`, or up the list when `backward`.  Without a row they
/// begin at the top, or at the bottom going up.
pub(super) fn rows_after(
    row: Option<usize>,
    count: usize,
    backward: bool,
) -> impl Iterator<Item = usize> {
    let row = row.unwrap_or(if backward { 0 } else { count.saturating_sub(1) });
    (1..=count).map(move |offset| {
        if backward {
            (row + count - offset % count) % count
        } else {
            (row + offset) % count
        }
    })
}

/// Returns whether the name or the number of the cue holds `needle`, which
/// is lowercase, in any case
pub(super) fn matches(cue: &Cue, needle: &str) -> bool {
    !needle.is_empty()
        && [&cue.name, &cue.number]
            .into_iter()
            .any(|field| field.to_lowercase().contains(needle))
}

/// Returns the first row after `row` that matches `text`, wrapping around,
/// so that `row` is the last to match
pub(super) fn found(cues: &[Cue], text: &str, row: Option<usize>, backward: bool) -> Option<usize> {
    let needle = text.to_lowercase();
    rows_after(row, cues.len(), backward).find(|&row| matches(&cues[row], &needle))
}

/// Returns how many cues hold `text`
pub(super) fn matching(cues: &[Cue], text: &str) -> usize {
    let needle = text.to_lowercase();
    cues.iter().filter(|cue| matches(cue, &needle)).count()
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
