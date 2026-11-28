// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Hints that fit the room of the bar, and what the latest change told,
//! which shortens at a word once no key can give way
//!
//! Cutting the status where the room ends would leave half a word of the
//! name of the show, so a notice that does not fit gives way itself, at a
//! word, and ends in an ellipsis, as it does at the edge of a room too short
//! for even its first word.

use std::iter;

use itertools::Itertools;
use unicode_width::UnicodeWidthStr;

use super::{LINE, format::silences, giving_way};
use crate::front::{
    keymap::Action,
    view::widget::{ELLIPSIS, shortened as cut_short},
};

/// Fewest words of a notice that stay once it is cut at its end, since
/// `Added cue…` has lost the number that told which cue
const LEAST_WORDS: usize = 3;

/// Fewest words that stay before the ellipsis of a cut that keeps the words
/// from the last `cue` on, as in `Undid… cue 6.5`
const LEAST_BEFORE: usize = 1;

/// Marks where the words that tell which cue a notice names begin
const WHICH: &str = " cue ";

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
/// as [`fitted`] joins them in `room` columns.  Where they still do not fit
/// once every key has given way, the notice is cut as [`shortened`] orders,
/// after the longest of its words that lets them fit.  The keys of
/// `outlasting` give way only after the notice has been cut to its shortest,
/// so that the key that leaves a field stays where a long notice would take
/// its room.  A notice that cannot be cut to [`LEAST_WORDS`] words is left
/// out for the keys that fit.  Where not even that fits, the hints are cut at
/// the edge of the room with an ellipsis, so that no text runs past it.
pub(super) fn shown(
    hints: impl Fn(Option<String>) -> Vec<(Option<Action>, String)>,
    told: Option<String>,
    room: usize,
    yielding: &[Action],
    outlasting: &[Action],
) -> String {
    let fits = |joined: &String| joined.width() <= room;
    let giving = yielding
        .iter()
        .copied()
        .filter(|action| !outlasting.contains(action))
        .collect::<Vec<_>>();
    let cut = |yielding: &[Action]| {
        let shorter = told.iter().flat_map(|told| shortened(told)).map(Some);
        iter::once(told.clone())
            .chain(shorter)
            .map(|told| fitted(&hints(told), room, yielding))
            .find(fits)
    };
    // Notice cut to a stub says nothing, so the keys keep the room instead
    let dropped = |yielding: &[Action]| {
        Some(fitted(&hints(None), room, yielding))
            .filter(|joined| !joined.is_empty() && fits(joined))
    };
    [giving.as_slice(), yielding]
        .into_iter()
        .find_map(|yielding| cut(yielding).or_else(|| dropped(yielding)))
        .unwrap_or_else(|| match room {
            // Ellipsis alone would take a column that is not there
            0 => String::new(),
            _ => cut_short(&fitted(&hints(told), room, yielding), room),
        })
}

/// Returns `told` with the keys after its first part that only teach given up,
/// all but the last, which takes the change back, then with that key given up
/// too where the words tell that a cue went silent, then with the words before
/// the last `cue` of its first part given up for an ellipsis, a word fewer
/// each time, then cut after each word of the part in turn, the longest
/// first, with an ellipsis in place of the rest of the part, which a comma
/// needs not, since the clause before it is whole
fn shortened(told: &str) -> impl Iterator<Item = String> + '_ {
    let head = told.split(LINE).next().unwrap_or(told);
    let rest = &told[head.len()..];
    let last = told
        .rsplit_once(LINE)
        .filter(|(before, _)| before.len() > head.len());
    let rest = last.map_or(rest.to_owned(), |(_, last)| format!("{LINE}{last}"));
    let lighter = last.map(|_| format!("{head}{rest}"));
    // A cue gone silent is what the operator cannot hear, so that its words
    // outlast the key that takes the change back, which the keys teach
    let bare = silences(head).then(|| head.to_owned());
    let spaces = head.match_indices(' ').map(|(end, _)| end).collect_vec();
    let words = |end: usize| head[..end].split_whitespace().count();
    // Words from the last `cue` on tell which cue the change was to, so that
    // the cuts that drop the words before them come before any that drops them
    let middles = head
        .rfind(WHICH)
        .map(|which| {
            spaces
                .iter()
                .rev()
                .filter(|&&end| end < which && words(end) >= LEAST_BEFORE)
                .map(|&end| {
                    let kept = head[..end].trim_end_matches([',', ':', ';']);
                    format!("{kept}{ELLIPSIS}{}{rest}", &head[which..])
                })
                .collect_vec()
        })
        .unwrap_or_default();
    let cuts = spaces
        .iter()
        .rev()
        .filter(|&&end| words(end) >= LEAST_WORDS)
        .map(|&end| {
            let clause = head[..end].ends_with(',');
            let kept = head[..end].trim_end_matches([',', ':', ';']);
            let stop = if clause || kept.ends_with(ELLIPSIS) {
                ""
            } else {
                ELLIPSIS
            };
            format!("{kept}{stop}{rest}")
        })
        .collect_vec();
    lighter.into_iter().chain(bare).chain(middles).chain(cuts)
}
