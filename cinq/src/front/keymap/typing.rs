// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading keys typed in a row, which wait until they complete a command

use itertools::Itertools;

mod alone;

use super::{Action, Binding, Keymap, Press, written};

/// What a key press amounts to, after the keys typed before it
#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    /// Keys of a command typed halfway, which wait for more
    pub pending: Vec<Press>,
    /// Action that the keys complete, with the count typed before them.  An
    /// operator that completes with a motion is the action, with the count
    /// typed before it.
    pub action: Option<(Action, Option<usize>)>,
    /// Motion that completes the operator in `action`, with the count typed
    /// before it
    pub motion: Option<(Action, Option<usize>)>,
}

impl Outcome {
    fn of(action: Action, count: Option<usize>) -> Self {
        Self {
            action: Some((action, count)),
            ..Self::default()
        }
    }

    /// Returns the operator, the motion, and the product of the two counts,
    /// as Vim multiplies them, when the keys complete an operator with a
    /// motion
    pub fn across(&self) -> Option<(Action, Action, Option<usize>)> {
        let ((operator, first), (motion, second)) = self.action.zip(self.motion)?;
        Some((operator, motion, product(first, second)))
    }
}

impl Keymap {
    /// Keys that complete or continue no command are dropped, with the keys
    /// typed so far, as Vim beeps and drops them, so that a key mistyped
    /// halfway through a command, such as the `x` of `d x`, never starts a
    /// command of its own.  The keys of GO and Panic act at once all the
    /// same.
    pub fn parse(&self, pending: &[Press], press: Press) -> Outcome {
        if let Some((action, _)) = self.at_once_bound(press) {
            return Outcome::of(action, None);
        }
        if press.digit().is_some() && saturated(pending) {
            return Outcome {
                pending: pending.to_vec(),
                ..Outcome::default()
            };
        }
        let keys = [pending, &[press]].concat();
        let (digits, command) = split_count(&keys);
        let count = counted(digits);
        match self.find(command) {
            Some(action) => Outcome::of(action, count),
            None if self.typable().any(|(_, bound)| bound.starts_with(command)) => Outcome {
                pending: keys,
                ..Outcome::default()
            },
            None => self.operated(&keys, count, command).unwrap_or_default(),
        }
    }

    /// Reads `command` as an operator, then a second count, then a motion,
    /// or returns nothing when it is not one.  The second count and the
    /// motion may still be missing, which keeps the keys waiting.
    fn operated(&self, keys: &[Press], count: Option<usize>, command: &[Press]) -> Option<Outcome> {
        let (operator, key) = self
            .operators()
            .find(|&(_, key)| command.first() == Some(&key))?;
        let (digits, motion) = split_count(&command[1..]);
        let waiting = Outcome {
            pending: keys.to_vec(),
            ..Outcome::default()
        };
        let motions = || {
            self.motions()
                .filter(|(_, bound)| bound.starts_with(motion))
        };
        match motion {
            [] => Some(waiting),
            // Doubled key stands for the rows counted, as in `d 3 d`
            [doubled] if *doubled == key => {
                Some(Outcome::of(operator, product(count, counted(digits))))
            }
            _ => match motions().find(|(_, bound)| *bound == motion) {
                Some((motion, _)) => Some(Outcome {
                    motion: Some((motion, counted(digits))),
                    ..Outcome::of(operator, count)
                }),
                None => motions().next().map(|_| waiting),
            },
        }
    }

    /// Returns the actions that apply to the rows of a motion, each with
    /// the key that its doubled binding repeats, as `d` does for `d d`
    fn operators(&self) -> impl Iterator<Item = (Action, Press)> + '_ {
        Action::OPERATORS.into_iter().flat_map(move |action| {
            self.keys(action)
                .iter()
                .filter_map(move |keys| match keys[..] {
                    [first, second] if first == second => Some((action, first)),
                    _ => None,
                })
        })
    }

    fn motions(&self) -> impl Iterator<Item = Binding<'_>> + Clone {
        self.bound()
            .filter(|(action, _)| Action::MOTIONS.contains(action))
    }

    /// Writes the keys that begin an operator, such as `d` for `d d`, once
    /// each, in the order of the actions
    pub fn operator_keys(&self) -> Vec<String> {
        self.operators()
            .map(|(_, key)| written(&[key]))
            .unique()
            .collect()
    }

    /// Returns what can follow the keys typed so far, apart from the counts
    /// before them: the rest of each binding that they begin, with its
    /// action, in the order of the actions on the help page.  After an
    /// operator, it lists the motions as well, each with the operator that
    /// it completes.  A count alone waits for any command, so nothing in
    /// particular follows it.
    pub fn continuations<'a>(
        &'a self,
        pending: &'a [Press],
    ) -> impl Iterator<Item = (&'a [Press], Action, Option<Action>)> + 'a {
        let (_, typed) = split_count(pending);
        let operator = typed
            .first()
            .and_then(|&first| self.operators().find(|&(_, key)| key == first))
            .map(|(operator, _)| operator);
        let (_, motion) = split_count(typed.get(1..).unwrap_or_default());
        // Count after an operator is no part of the doubled binding
        let bare: Vec<Press> = match operator {
            Some(_) => typed.iter().take(1).chain(motion).copied().collect(),
            None => typed.to_vec(),
        };
        let plain: Vec<_> = self
            .typable()
            .filter(|_| !bare.is_empty())
            .filter_map(|(action, keys)| {
                keys.strip_prefix(&bare[..])
                    .filter(|rest| !rest.is_empty())
                    .map(|rest| (rest, action, None))
            })
            .collect();
        let motions: Vec<_> = operator
            .into_iter()
            .flat_map(|operator| {
                self.motions().filter_map(move |(action, keys)| {
                    keys.strip_prefix(motion)
                        .filter(|rest| !rest.is_empty())
                        .map(|rest| (rest, action, Some(operator)))
                })
            })
            .collect();
        plain.into_iter().chain(motions)
    }

    fn find(&self, keys: &[Press]) -> Option<Action> {
        self.typable()
            .find_map(|(action, bound)| (bound == keys).then_some(action))
    }
}

fn split_count(keys: &[Press]) -> (&[Press], &[Press]) {
    keys.split_at(keys.iter().take_while(|key| key.digit().is_some()).count())
}

fn counted(digits: &[Press]) -> Option<usize> {
    let count = digits
        .iter()
        .filter_map(|key| key.digit())
        .fold(0, |count: usize, digit| {
            count.saturating_mul(10).saturating_add(digit)
        });
    Some(count).filter(|&count| count > 0)
}

/// Tells whether the digits that end `keys` already make the largest count,
/// so that one more changes nothing and a held digit key costs nothing
fn saturated(keys: &[Press]) -> bool {
    let start = keys
        .iter()
        .rposition(|key| key.digit().is_none())
        .map_or(0, |last| last + 1);
    counted(&keys[start..]) == Some(usize::MAX)
}

/// Multiplies the counts typed before an operator and before its motion,
/// where a missing count stands for one, or gives no count when both are
/// missing
fn product(first: Option<usize>, second: Option<usize>) -> Option<usize> {
    first
        .zip(second)
        .map(|(first, second)| first.saturating_mul(second))
        .or(first)
        .or(second)
}
