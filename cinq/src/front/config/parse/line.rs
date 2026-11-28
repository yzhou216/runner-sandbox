// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keymaps of the file's own that start each from another, and the keys
//! that a line of them comes to

use std::ops::{ControlFlow, Range};

use toml::{
    Spanned,
    de::{DeString, DeTable, DeValue},
};

use super::{Problem, below_a_table, bind, spanned};
use crate::front::keymap::{Keymap, vi};

/// Keys that a keymap starts from, or the problem that leaves it none
type Start = Result<(Keymap, Vec<Problem>), Problem>;

/// `named_at` is the part of the text that names the keymap, if any, where a
/// problem with the name lies.  Each keymap of the line that [`line()`] finds
/// binds its keys over those of the one after it, from the end of the line
/// back to the keymap named.
pub(super) fn build(name: &str, named_at: Option<Range<usize>>, keymaps: &DeTable) -> Start {
    let (line, root) = line(name, named_at, keymaps);
    line.into_iter().rev().fold(root, |start, (name, entries)| {
        let (start, problems) =
            start.unwrap_or_else(|problem| (Keymap::empty(name), vec![problem]));
        let bindings = entries.iter().filter(|(key, _)| key.get_ref() != "from");
        let (keymap, unbound) = bind(start.named(name), bindings, &format!("keymaps.{name}: "));
        Ok((keymap, [problems, unbound].concat()))
    })
}

/// Returns the keymaps of the file's own that start each from the next, the
/// named one first, and what the last of them starts from.  A keymap met
/// again ends the line, since it starts from itself and so from no keys.
/// The line is walked rather than recursed into, so that a file of
/// thousands of keymaps cannot overflow the stack of the thread that reads
/// it.
fn line<'a, 'de>(
    name: &'a str,
    named_at: Option<Range<usize>>,
    keymaps: &'a DeTable<'de>,
) -> (Vec<(&'a str, &'a DeTable<'de>)>, Start) {
    let mut line = Vec::new();
    let mut next = (name, named_at);
    loop {
        let (name, named_at) = next;
        let entries = match declared(name, named_at.clone(), keymaps) {
            ControlFlow::Continue(entries) => entries,
            ControlFlow::Break(root) => return (line, root),
        };
        if line.iter().any(|&(seen, _)| seen == name) {
            let looped = format!("keymap {name} starts from itself");
            return (line, Err(Problem::at(named_at, looped)));
        }
        line.push((name, entries));
        next = match entries.get("from").map(spanned) {
            Some((DeValue::String(from), span)) => (from.as_ref(), Some(span)),
            Some((_, span)) => return (line, Err(unnamed(name, span))),
            None => return (line, Ok((Keymap::empty(name), Vec::new()))),
        };
    }
}

fn unnamed(name: &str, span: Range<usize>) -> Problem {
    let said = format!("keymaps.{name}.from should be the name of a keymap");
    Problem::at(span, said)
}

/// Returns the problems of every table under `[keymaps]`, so that a keymap
/// that no setting uses shows them now and not on the day that it is chosen.
/// Each table is bound over no keys, so the keys that an entry would keep and
/// the problems of the keys beneath it are left to the keymap in use, and a
/// problem that the keymap in use also finds comes twice, which the caller
/// drops.
pub(super) fn audit(keymaps: &DeTable) -> Vec<Problem> {
    keymaps
        .iter()
        .filter(|(name, _)| name.get_ref() != vi::NAME)
        .flat_map(|(key, value)| {
            let name: &str = key.get_ref().as_ref();
            match spanned(value) {
                (DeValue::Table(entries), _) => audit_table(name, entries, keymaps),
                (_, span) => vec![misplaced(key, name, span)],
            }
        })
        .collect()
}

fn audit_table(name: &str, entries: &DeTable, keymaps: &DeTable) -> Vec<Problem> {
    let bindings = entries.iter().filter(|(key, _)| key.get_ref() != "from");
    let (_, unbound) = bind(Keymap::empty(name), bindings, &format!("keymaps.{name}: "));
    let from = match entries.get("from").map(spanned) {
        Some((DeValue::String(from), span)) => match declared(from, Some(span), keymaps) {
            ControlFlow::Break(start) => {
                start.map_or_else(|problem| vec![problem], |(_, said)| said)
            }
            ControlFlow::Continue(_) => Vec::new(),
        },
        Some((_, span)) => vec![unnamed(name, span)],
        None => Vec::new(),
    };
    [from, unbound].concat()
}

/// Returns the problem of a value under `[keymaps]` that is not a table,
/// which is told as a setting written below a table when it has a setting's
/// name
fn misplaced(key: &Spanned<DeString>, name: &str, span: Range<usize>) -> Problem {
    below_a_table(name).map_or_else(
        || Problem::at(span, format!("keymaps.{name} should be a table")),
        |said| Problem::at(key.span(), said),
    )
}

/// Returns the table of the keymap of the file's own that `name` names, or
/// what ends a line of keymaps there: the keys of a shipped keymap, or the
/// problem with the name
fn declared<'a, 'de>(
    name: &str,
    named_at: Option<Range<usize>>,
    keymaps: &'a DeTable<'de>,
) -> ControlFlow<Start, &'a DeTable<'de>> {
    if name == vi::NAME {
        return ControlFlow::Break(Ok((Keymap::default(), Vec::new())));
    }
    match keymaps.get(name).map(spanned) {
        Some((DeValue::Table(entries), _)) => ControlFlow::Continue(entries),
        Some((_, span)) => {
            let malformed = format!("keymaps.{name} should be a table");
            ControlFlow::Break(Err(Problem::at(span, malformed)))
        }
        // Vi stands in for the keymap that cinq shipped before, so that an
        // operator who relied on it keeps GO and Panic, where a keymap that
        // starts from an unknown one has no keys
        None if name == "cua" => {
            let retired = "cinq no longer ships the cua keymap, so vi stands in for it";
            ControlFlow::Break(Ok((
                Keymap::default(),
                vec![Problem::at(named_at, retired)],
            )))
        }
        None => {
            let unknown = format!("no keymap is named `{name}`");
            ControlFlow::Break(Err(Problem::at(named_at, unknown)))
        }
    }
}
