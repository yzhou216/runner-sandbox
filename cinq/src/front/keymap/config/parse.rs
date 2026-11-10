// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading the text of the configuration file into key bindings and the
//! seconds ahead to make sounds ready, along with the problems that leave
//! parts of it unused, each in the part of the text that it lies in

use std::{ops::Range, slice};

use itertools::Itertools;
use toml::{
    Spanned,
    de::{DeString, DeTable, DeValue},
};

use super::Problem;
use crate::{
    back::READY_AHEAD,
    front::keymap::{Action, Keymap, Preset, binding},
};

const SETTINGS: [&str; 4] = ["keymap", "keymaps", "keys", "ready_ahead"];

pub(in crate::front::keymap) fn parse(text: &str) -> (Keymap, f64, Vec<Problem>) {
    let table = match DeTable::parse(text) {
        Ok(table) => table.into_inner(),
        Err(error) => {
            let problem = Problem::at(error.span(), format!("is not TOML: {}", error.message()));
            return (Keymap::default(), READY_AHEAD, vec![problem]);
        }
    };
    let unknown = table
        .keys()
        .filter(|name| !SETTINGS.contains(&name.get_ref().as_ref()))
        .map(|name| Problem::at(name.span(), format!("no setting is named `{name}`")));
    let none = DeTable::new();
    let (keymaps, malformed) = match table.get("keymaps").map(spanned) {
        Some((DeValue::Table(keymaps), _)) => (keymaps, Vec::new()),
        Some((_, span)) => (&none, vec![Problem::at(span, "keymaps should be a table")]),
        None => (&none, Vec::new()),
    };
    let shadowed = keymaps
        .keys()
        .filter(|name| name.get_ref().parse::<Preset>().is_ok())
        .map(|name| {
            let unused = format!("keymaps.{name} goes unused, since {name} is a shipped keymap");
            Problem::at(name.span(), unused)
        });
    let default = (Preset::default().name(), None);
    let ((name, named_at), unnamed) = match table.get("keymap").map(spanned) {
        Some((DeValue::String(name), span)) => ((name.as_ref(), Some(span)), None),
        Some((_, span)) => {
            let unnamed = Problem::at(span, "keymap should be the name of a keymap");
            (default, Some(unnamed))
        }
        None => (default, None),
    };
    let (chosen, built) = build(name, named_at, keymaps, &[])
        .unwrap_or_else(|problem| (Keymap::default(), vec![problem]));
    let (keymap, unbound) = match table.get("keys").map(spanned) {
        Some((DeValue::Table(keys), _)) => bind(chosen, keys, ""),
        Some((_, span)) => (chosen, vec![Problem::at(span, "keys should be a table")]),
        None => (chosen, Vec::new()),
    };
    let (ready_ahead, unready) = ready_ahead(&table);
    let problems = unknown
        .chain(malformed)
        .chain(shadowed)
        .chain(unnamed)
        .chain(built)
        .chain(unbound)
        .chain(unready)
        .chain(keymap.conflicts().into_iter().map(Problem::new))
        .collect();
    (keymap, ready_ahead, problems)
}

/// Returns the seconds that `ready_ahead` sets, or the default with a
/// problem when the setting is not a number, 0 or more
fn ready_ahead(table: &DeTable) -> (f64, Option<Problem>) {
    let Some((value, span)) = table.get("ready_ahead").map(spanned) else {
        return (READY_AHEAD, None);
    };
    let seconds = match value {
        DeValue::Integer(whole) => i64::from_str_radix(whole.as_str(), whole.radix())
            .ok()
            .map(|whole| whole as f64),
        DeValue::Float(fraction) => fraction.as_str().parse().ok(),
        _ => None,
    };
    match seconds.filter(|seconds| (0.0..=f64::MAX).contains(seconds)) {
        Some(seconds) => (seconds, None),
        None => {
            let bad = "ready_ahead should be a number of seconds, 0 or more";
            (READY_AHEAD, Some(Problem::at(span, bad)))
        }
    }
}

/// `named_at` is the part of the text that names the keymap, if any, where a
/// problem with the name lies.  `chain` holds the keymaps that start from
/// this one, so that a keymap that comes back around to itself starts from
/// no keys instead.
fn build(
    name: &str,
    named_at: Option<Range<usize>>,
    keymaps: &DeTable,
    chain: &[&str],
) -> Result<(Keymap, Vec<Problem>), Problem> {
    if let Ok(preset) = name.parse::<Preset>() {
        return Ok((preset.into(), Vec::new()));
    }
    let entries = match keymaps.get(name).map(spanned) {
        Some((DeValue::Table(entries), _)) => entries,
        Some((_, span)) => {
            let malformed = format!("keymaps.{name} should be a table");
            return Err(Problem::at(span, malformed));
        }
        None => {
            let unknown = format!("no keymap is named `{name}`");
            return Err(Problem::at(named_at, unknown));
        }
    };
    if chain.contains(&name) {
        let looped = format!("keymap {name} starts from itself");
        return Err(Problem::at(named_at, looped));
    }
    let chain = [chain, &[name]].concat();
    let start = match entries.get("from").map(spanned) {
        Some((DeValue::String(from), span)) => build(from, Some(span), keymaps, &chain),
        Some((_, span)) => {
            let unnamed = format!("keymaps.{name}.from should be the name of a keymap");
            Err(Problem::at(span, unnamed))
        }
        None => Ok((Keymap::empty(name), Vec::new())),
    };
    let (start, problems) = start.unwrap_or_else(|problem| (Keymap::empty(name), vec![problem]));
    let bindings = entries.iter().filter(|(key, _)| key.get_ref() != "from");
    let (keymap, unbound) = bind(start.named(name), bindings, &format!("keymaps.{name}: "));
    Ok((keymap, [problems, unbound].concat()))
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

fn entry(
    keymap: Keymap,
    name: &Spanned<DeString>,
    value: &Spanned<DeValue>,
) -> (Keymap, Vec<Problem>) {
    let Ok(action) = name.get_ref().parse::<Action>() else {
        let unknown = format!("no action is named `{name}`");
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
    let sequences = sequences.into_iter().unique().collect();
    (keymap.with(action, sequences), problems)
}

fn spanned<T>(value: &Spanned<T>) -> (&T, Range<usize>) {
    (value.get_ref(), value.span())
}
