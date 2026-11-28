// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Key bindings, which turn key presses into the messages that buttons send

mod action;
#[cfg(test)]
mod desk;
mod press;
#[cfg(test)]
mod tests;
mod typing;
pub(super) mod vi;

use std::collections::HashMap;

use itertools::Itertools;
use strum::VariantArray;

pub use action::{Action, Rows};
#[cfg(test)]
pub(super) use desk::{declared, desk};
#[cfg(test)]
pub use press::sequence;
pub use press::{KeyPress, Press, shown, written};

#[derive(Debug, Clone)]
pub struct Keymap {
    name: String,
    bindings: HashMap<Action, Vec<Vec<Press>>>,
}

impl Default for Keymap {
    fn default() -> Self {
        let bindings = Action::VARIANTS.iter().map(|&action| {
            let keys = vi::keys(action).iter();
            (action, keys.filter_map(|keys| binding(keys).ok()).collect())
        });
        Self {
            name: vi::NAME.to_owned(),
            bindings: bindings.collect(),
        }
    }
}

impl Keymap {
    pub(super) fn empty(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            bindings: HashMap::new(),
        }
    }

    pub(super) fn named(self, name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..self
        }
    }

    pub(super) fn with(self, action: Action, keys: Vec<Vec<Press>>) -> Self {
        // Pair chained last wins over the keys that the action had
        let bindings = self.bindings.into_iter().chain([(action, keys)]).collect();
        Self { bindings, ..self }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn keys(&self, action: Action) -> &[Vec<Press>] {
        self.bindings.get(&action).map_or(&[], Vec::as_slice)
    }

    /// Writes the first key sequence bound to an action, which stands for
    /// the rest where there is room for one
    pub fn key(&self, action: Action) -> Option<String> {
        self.keys(action).first().map(|keys| shown(keys))
    }

    pub fn hint(&self, action: Action, label: &str) -> Option<String> {
        self.key(action).map(|keys| format!("{keys}: {label}"))
    }

    pub fn listed(&self, action: Action) -> String {
        match self.keys(action) {
            [] => "none".to_owned(),
            keys => keys.iter().map(|keys| shown(keys)).join(", "),
        }
    }

    #[cfg(test)]
    fn conflicts(&self) -> Vec<String> {
        self.clashes().into_iter().map(|(_, said)| said).collect()
    }

    /// Describes the bindings that hide others, each with the action that
    /// cannot be reached, whose entry in the file the operator should change:
    /// those that [`conflict`] finds, and a key of [`Action::AT_ONCE`], which
    /// keeps every other sequence that holds it from being typed, by acting
    /// halfway through it or when held with other modifiers than it is bound
    /// with
    pub(super) fn clashes(&self) -> Vec<(Action, String)> {
        let hidden = self
            .bound()
            .array_combinations()
            .filter_map(|[earlier, later]| conflict(earlier, later));
        // Sequence that begins with the very key bound is the binding of the
        // key itself, or one that `conflict` describes
        let cut_short = self.bound().filter_map(|(owner, keys)| {
            let (action, key) = keys.iter().find_map(|&press| self.at_once_bound(press))?;
            (keys[0] != key).then(|| {
                let said = format!(
                    "`{}` acts at once as {}, so `{}` can never be typed",
                    written(&[key]),
                    action.name(),
                    written(keys)
                );
                (owner, said)
            })
        });
        hidden.chain(cut_short).collect()
    }

    /// Returns the bindings that a key typed in the cue list reads, which
    /// leaves out those of an open menu
    fn typable(&self) -> impl Iterator<Item = Binding<'_>> + Clone {
        self.bound().filter(|(action, _)| action.acts_in_cue_list())
    }

    fn bound(&self) -> impl Iterator<Item = Binding<'_>> + Clone {
        Action::VARIANTS.iter().flat_map(move |&action| {
            self.keys(action)
                .iter()
                .map(move |keys| (action, keys.as_slice()))
        })
    }
}

type Binding<'a> = (Action, &'a [Press]);

/// A binding cannot start with a digit, since digits typed first count the
/// command.
pub(super) fn binding(text: &str) -> Result<Vec<Press>, String> {
    let keys = press::sequence(text)?;
    match keys.first() {
        None => Err("a binding needs at least one key".to_owned()),
        Some(key) if key.digit().is_some() => {
            Err(format!("`{text}` starts with a digit, which counts"))
        }
        Some(_) => Ok(keys),
    }
}

/// Describes how one of two bindings, in the order of [`Action::VARIANTS`],
/// hides the other, if one does.  Bindings of actions that act in different
/// places, such as picking in a menu and scrolling the cue list, hide
/// nothing.
fn conflict((earlier, keys): Binding, (later, other): Binding) -> Option<(Action, String)> {
    if !earlier.shares_place(later) {
        return None;
    }
    if keys == other {
        let (earlier, later_name) = (earlier.name(), later.name());
        return Some((
            later,
            format!(
                "`{}` is bound to both {earlier} and {later_name}, so {earlier} keeps it and {later_name} loses it",
                written(keys)
            ),
        ));
    }
    [(keys, other, later), (other, keys, earlier)]
        .into_iter()
        .find(|(short, long, _)| long.starts_with(short))
        .map(|(short, long, owner)| {
            (
                owner,
                format!(
                    "`{}` comes first, so `{}` can never be typed",
                    written(short),
                    written(long)
                ),
            )
        })
}
