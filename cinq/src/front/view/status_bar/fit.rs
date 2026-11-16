// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints that fit the room of the bar, and what the latest change told,
//! which shortens at a word once no key can give way
//!
//! Cutting the status where the room ends would leave half a word of the
//! name of the show, so a notice that does not fit gives way itself, at a
//! word, and ends in an ellipsis.

use itertools::Itertools;
use unicode_width::UnicodeWidthStr;

use super::{LINE, giving_way};
use crate::front::keymap::Action;

/// Ends a notice that was shortened
pub(super) const ELLIPSIS: &str = "…";

/// Returns the hints joined, each one whole, without as many of those
/// that give way as it takes for the rest to fit in `room` columns
pub(super) fn fitted(
    hints: &[(Option<Action>, String)],
    room: usize,
    yielding: &[Action],
) -> String {
    giving_way(
        yielding,
        |gone| {
            hints
                .iter()
                .filter(|(action, _)| !action.is_some_and(|action| gone.contains(&action)))
                .map(|(_, hint)| hint)
                .join(LINE)
        },
        |hints| hints.width() <= room,
    )
}

/// Returns the hints that `hints` gives for what the latest change `told`,
/// as [`fitted`] joins them in `room` columns, and where they still do not
/// fit once every key has given way, with the notice cut after the longest
/// of its words that lets them fit.  The keys that follow the notice after
/// a separator give way first, but for the last.
pub(super) fn shown(
    hints: impl Fn(Option<String>) -> Vec<(Option<Action>, String)>,
    told: Option<String>,
    room: usize,
    yielding: &[Action],
) -> String {
    let joined = |told: Option<String>| fitted(&hints(told), room, yielding);
    let full = joined(told.clone());
    if full.width() <= room {
        return full;
    }
    told.iter()
        .flat_map(|told| shortened(told))
        .map(|short| joined(Some(short)))
        .find(|joined| joined.width() <= room)
        .unwrap_or(full)
}

/// Returns `told` with the keys after its first part that only teach given up,
/// all but the last, which takes the change back, then cut after each word of
/// its first part in turn, the longest first, with an ellipsis in place of the
/// rest of the part, which a comma needs not, since the clause before it is whole
fn shortened(told: &str) -> impl Iterator<Item = String> + '_ {
    let head = told.split(LINE).next().unwrap_or(told);
    let rest = &told[head.len()..];
    let last = told
        .rsplit_once(LINE)
        .filter(|(before, _)| before.len() > head.len());
    let rest = last.map_or(rest.to_owned(), |(_, last)| format!("{LINE}{last}"));
    let lighter = last.map(|_| format!("{head}{rest}"));
    let cuts = head
        .match_indices(' ')
        .map(|(end, _)| end)
        .rev()
        .map(move |end| {
            let clause = head[..end].ends_with(',');
            let kept = head[..end].trim_end_matches([',', ':', ';']);
            let stop = if clause || kept.ends_with(ELLIPSIS) {
                ""
            } else {
                ELLIPSIS
            };
            format!("{kept}{stop}{rest}")
        });
    lighter.into_iter().chain(cuts)
}
