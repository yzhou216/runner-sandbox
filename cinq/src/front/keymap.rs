// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Key bindings, which turn key presses into the messages that buttons send

mod action;
pub mod config;
mod preset;
mod press;
#[cfg(test)]
mod tests;
mod typing;

use std::collections::HashMap;

use itertools::Itertools;
use strum::VariantArray;

pub use action::Action;
pub use preset::Preset;
pub use press::{Press, written};

#[derive(Debug, Clone)]
pub struct Keymap {
    name: String,
    bindings: HashMap<Action, Vec<Vec<Press>>>,
}

impl From<Preset> for Keymap {
    fn from(preset: Preset) -> Self {
        let bindings = Action::VARIANTS.iter().map(|&action| {
            let keys = preset.keys(action).iter();
            (action, keys.filter_map(|keys| binding(keys).ok()).collect())
        });
        Self {
            name: preset.name().to_owned(),
            bindings: bindings.collect(),
        }
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Preset::default().into()
    }
}

impl Keymap {
    fn empty(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            bindings: HashMap::new(),
        }
    }

    fn named(self, name: &str) -> Self {
        Self {
            name: name.to_owned(),
            ..self
        }
    }

    fn with(self, action: Action, keys: Vec<Vec<Press>>) -> Self {
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
        self.keys(action).first().map(|keys| written(keys))
    }

    pub fn hint(&self, action: Action, label: &str) -> Option<String> {
        self.key(action).map(|keys| format!("{keys}: {label}"))
    }

    pub fn listed(&self, action: Action) -> String {
        match self.keys(action) {
            [] => "none".to_owned(),
            keys => keys.iter().map(|keys| written(keys)).join(", "),
        }
    }

    /// Describes the bindings that hide others: those that [`conflict`]
    /// finds, and a key of [`Action::AT_ONCE`], which keeps every other
    /// sequence that holds it from being typed, by acting halfway through it
    /// or when held with other modifiers than it is bound with
    fn conflicts(&self) -> Vec<String> {
        let hidden = self
            .bound()
            .array_combinations()
            .filter_map(|[earlier, later]| conflict(earlier, later));
        // Sequence that begins with the very key bound is the binding of the
        // key itself, or one that `conflict` describes
        let cut_short = self.bound().filter_map(|(_, keys)| {
            let (action, key) = keys.iter().find_map(|&press| self.at_once_bound(press))?;
            (keys[0] != key).then(|| {
                format!(
                    "`{}` acts at once as {}, so `{}` can never be typed",
                    written(&[key]),
                    action.name(),
                    written(keys)
                )
            })
        });
        hidden.chain(cut_short).collect()
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
fn binding(text: &str) -> Result<Vec<Press>, String> {
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
/// hides the other, if one does
fn conflict((earlier, keys): Binding, (later, other): Binding) -> Option<String> {
    if keys == other {
        let (earlier, later) = (earlier.name(), later.name());
        return Some(format!(
            "`{}` is bound to both {earlier} and {later}, so {later} goes without",
            written(keys)
        ));
    }
    [(keys, other), (other, keys)]
        .into_iter()
        .find(|(short, long)| long.starts_with(short))
        .map(|(short, long)| {
            format!(
                "`{}` comes first, so `{}` can never be typed",
                written(short),
                written(long)
            )
        })
}
