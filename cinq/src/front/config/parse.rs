// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the text of the configuration file into key bindings and the
//! settings of the sound output, along with the problems that leave parts
//! of it unused, each in the part of the text that it lies in

mod line;
mod settings;

use std::{net::SocketAddr, ops::Range, slice, sync::LazyLock};

use itertools::Itertools;
use toml::{
    Spanned,
    de::{DeString, DeTable, DeValue},
};

use super::{Problem, Sound};
use crate::front::keymap::{Action, Keymap, binding, vi};
use line::{audit, build};
use settings::{audio_device, osc_listen, ready_ahead};

const SETTINGS: [&str; 6] = [
    "audio_device",
    "keymap",
    "keymaps",
    "keys",
    "osc_listen",
    "ready_ahead",
];

pub(in crate::front) fn parse(text: &str) -> (Keymap, Sound, Option<SocketAddr>, Vec<Problem>) {
    let table = match DeTable::parse(text) {
        Ok(table) => table.into_inner(),
        Err(error) => {
            let problem = Problem::at(error.span(), format!("is not TOML: {}", error.message()));
            return (Keymap::default(), Sound::default(), None, vec![problem]);
        }
    };
    let unknown = unknown_settings(&table);
    let (keymaps, malformed) = keymaps_table(&table);
    let shadowed = shadowed_keymaps(keymaps);
    let (name, named_at, unnamed) = keymap_name(&table);
    let (chosen, built) =
        build(name, named_at, keymaps).unwrap_or_else(|problem| (Keymap::default(), vec![problem]));
    let (keymap, unbound) = bind_keys(chosen, &table);
    let (device, unrouted) = audio_device(&table);
    let (ready_ahead, unready) = ready_ahead(&table);
    let (osc_listen, unlistened) = osc_listen(&table);
    let keys_table = table.get("keys").and_then(|keys| keys.get_ref().as_table());
    let clashing = clashes(&keymap, keys_table);
    let problems = unknown
        .into_iter()
        .chain(malformed)
        .chain(shadowed)
        .chain(unnamed)
        .chain(built)
        .chain(audit(keymaps))
        .chain(unbound)
        .chain(unrouted)
        .chain(unready)
        .chain(unlistened)
        .chain(clashing)
        // The audit repeats the problems that the keymap in use found
        .unique_by(|problem| (problem.span.clone(), problem.message.clone()));
    (
        keymap,
        Sound {
            device,
            ready_ahead,
        },
        osc_listen,
        in_file_order(problems),
    )
}

fn unknown_settings(table: &DeTable) -> Vec<Problem> {
    table
        .keys()
        .filter(|name| !SETTINGS.contains(&name.get_ref().as_ref()))
        .map(|name| Problem::at(name.span(), format!("no setting is named `{name}`")))
        .collect()
}

/// Returns the `[keymaps]` table, or an empty one where there is none, with
/// the problem of one that is not a table
fn keymaps_table<'a>(table: &'a DeTable<'a>) -> (&'a DeTable<'a>, Vec<Problem>) {
    static NONE: LazyLock<DeTable<'static>> = LazyLock::new(DeTable::new);
    match table.get("keymaps").map(spanned) {
        Some((DeValue::Table(keymaps), _)) => (keymaps, Vec::new()),
        Some((_, span)) => (&NONE, vec![Problem::at(span, "keymaps should be a table")]),
        None => (&NONE, Vec::new()),
    }
}

fn shadowed_keymaps(keymaps: &DeTable) -> Vec<Problem> {
    keymaps
        .keys()
        .filter(|name| name.get_ref() == vi::NAME)
        .map(|name| {
            let unused = format!("keymaps.{name} goes unused, since {name} is a shipped keymap");
            Problem::at(name.span(), unused)
        })
        .collect()
}

/// Binds the entries of `[keys]` over `keymap`, with the problem of a `keys`
/// that is not a table
fn bind_keys(keymap: Keymap, table: &DeTable) -> (Keymap, Vec<Problem>) {
    match table.get("keys").map(spanned) {
        Some((DeValue::Table(keys), _)) => bind(keymap, keys, ""),
        Some((_, span)) => (keymap, vec![Problem::at(span, "keys should be a table")]),
        None => (keymap, Vec::new()),
    }
}

/// Name of the keymap to build and where the file names it, with the
/// problem of a name that is not one
fn keymap_name<'a>(table: &'a DeTable<'a>) -> (&'a str, Option<Range<usize>>, Option<Problem>) {
    match table.get("keymap").map(spanned) {
        Some((DeValue::String(name), span)) => (name.as_ref(), Some(span), None),
        Some((_, span)) => {
            let unnamed = Problem::at(span, "keymap should be the name of a keymap");
            (vi::NAME, None, Some(unnamed))
        }
        None => (vi::NAME, None, None),
    }
}

/// Problems of actions whose keys clash, each at the entry of `[keys]`
/// that the operator wrote, if the action has one
fn clashes(keymap: &Keymap, keys_table: Option<&DeTable>) -> Vec<Problem> {
    keymap
        .clashes()
        .into_iter()
        .map(|(action, said)| {
            let entry = keys_table
                .and_then(|keys| keys.keys().find(|name| name.get_ref() == action.name()));
            Problem::at(entry.map(Spanned::span), said)
        })
        // Actions of different places may hold one sequence, which a key
        // that acts at once cuts short for each of them
        .unique_by(|problem| (problem.span.clone(), problem.message.clone()))
        .collect()
}

/// Problems in the order of the file, so that a terminal and the sidebar
/// read top down, by the end of a place so that an array follows the
/// elements that it holds, and those in no place last
fn in_file_order(problems: impl Iterator<Item = Problem>) -> Vec<Problem> {
    problems
        .sorted_by_key(|problem| {
            problem
                .span
                .as_ref()
                .map_or((true, 0), |span| (false, span.end))
        })
        .collect()
}

fn bind<'a>(
    keymap: Keymap,
    entries: impl IntoIterator<Item = (&'a Spanned<DeString<'a>>, &'a Spanned<DeValue<'a>>)>,
    place: &str,
) -> (Keymap, Vec<Problem>) {
    entries
        .into_iter()
        .fold((keymap, Vec::new()), |(keymap, problems), (name, value)| {
            let (keymap, more) = entry(keymap, name, value);
            let more = more.into_iter().map(|problem| Problem {
                message: format!("{place}{}", problem.message),
                ..problem
            });
            (keymap, problems.into_iter().chain(more).collect())
        })
}

/// Returns what to say of a setting added below a table, which lands in it,
/// where only the cause helps
fn below_a_table(name: &str) -> Option<String> {
    SETTINGS
        .contains(&name)
        .then(|| format!("`{name}` is a setting, which belongs above the first table"))
}

fn entry(
    keymap: Keymap,
    name: &Spanned<DeString>,
    value: &Spanned<DeValue>,
) -> (Keymap, Vec<Problem>) {
    let Ok(action) = name.get_ref().parse::<Action>() else {
        let unknown =
            below_a_table(name.get_ref()).unwrap_or_else(|| format!("no action is named `{name}`"));
        return (keymap, vec![Problem::at(name.span(), unknown)]);
    };
    let values = match value.get_ref() {
        DeValue::Array(values) => &values[..],
        _ => slice::from_ref(value),
    };
    let (sequences, problems): (Vec<_>, Vec<_>) = values
        .iter()
        .map(|value| {
            let keys = match value.get_ref() {
                DeValue::String(text) => {
                    binding(text).map_err(|problem| format!("{name}: {problem}"))
                }
                _ => Err(format!("{name} should be keys or a list of keys")),
            };
            keys.map_err(|problem| Problem::at(value.span(), problem))
        })
        .partition_result();
    let sequences: Vec<_> = sequences.into_iter().unique().collect();
    // Entry of which no key can be read keeps the keys of the action, so that
    // a slip leaves GO and Panic bound, but an empty list unbinds on purpose
    if sequences.is_empty() && !problems.is_empty() && keymap.key(action).is_some() {
        let kept = Problem::at(
            value.span(),
            format!("{name} keeps {}", keymap.listed(action)),
        );
        return (keymap, [problems, vec![kept]].concat());
    }
    (keymap.with(action, sequences), problems)
}

fn spanned<T>(value: &Spanned<T>) -> (&T, Range<usize>) {
    (value.get_ref(), value.span())
}
