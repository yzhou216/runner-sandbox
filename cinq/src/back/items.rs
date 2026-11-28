// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Changes to vectors and maps that move their items instead of cloning
//! them

use std::{collections::HashMap, hash::Hash};

/// Returns the items with the one at an index changed, or unchanged when
/// the index is past the end
pub(super) fn replaced<T>(items: Vec<T>, index: usize, change: impl FnOnce(T) -> T) -> Vec<T> {
    let mut change = Some(change);
    items
        .into_iter()
        .enumerate()
        .map(|(at, item)| match change.take_if(|_| at == index) {
            Some(change) => change(item),
            None => item,
        })
        .collect()
}

pub(super) fn replaced_where<T>(
    items: Vec<T>,
    matches: impl Fn(&T) -> bool,
    change: impl FnOnce(T) -> T,
) -> Vec<T> {
    match items.iter().position(matches) {
        Some(index) => replaced(items, index, change),
        None => items,
    }
}

/// Removes the entry of `key` from `entries`, which pair each value with
/// a tag, and returns the value if `keep` accepts the tag.  A rejected
/// entry is gone all the same.
pub(super) fn taken<K: Eq + Hash, T, V>(
    entries: &mut HashMap<K, (T, V)>,
    key: &K,
    keep: impl FnOnce(&T) -> bool,
) -> Option<V> {
    entries
        .remove(key)
        .filter(|(tag, _)| keep(tag))
        .map(|(_, value)| value)
}
