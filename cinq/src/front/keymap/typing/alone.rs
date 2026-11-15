// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! The actions of keys pressed alone, which act in fields and in an open
//! menu, and those of GO and Panic, which act at once

use super::{Action, Keymap, Press};

impl Keymap {
    pub fn acts_in_fields(&self, press: Press) -> bool {
        self.alone(press).is_some_and(Action::acts_in_fields)
    }

    pub fn acts_at_once(&self, press: Press) -> bool {
        self.alone(press).is_some_and(Action::acts_at_once)
    }

    /// Returns whether the key pressed alone is that of [`Action::Cancel`]
    pub fn cancels(&self, press: Press) -> bool {
        self.alone(press) == Some(Action::Cancel)
    }

    /// Returns whether a key that a text field captured acts all the same.
    /// A key of GO or Panic does when it is no character, and a key of
    /// cancel does unless it types one, so that a letter that a user binds
    /// to cancel still types in the field while Control+c, which iced takes
    /// to copy a selection, cancels.  `character` tells whether the key is
    /// a character or the space bar.
    pub fn acts_when_captured(&self, press: Press, character: bool) -> bool {
        (!character && self.acts_at_once(press))
            || (self.cancels(press) && !(character && !press.holds_control_or_alt()))
    }

    pub fn alone(&self, press: Press) -> Option<Action> {
        self.at_once_bound(press)
            .map(|(action, _)| action)
            .or_else(|| self.find(&[press]))
    }

    /// Returns the action that a key pressed alone has while a menu is open,
    /// where the keys of [`Action::Pick`] and [`Action::CloseMenu`] act, and
    /// win over a key that the cue list takes for scrolling
    pub fn in_menu(&self, press: Press) -> Option<Action> {
        self.at_once_bound(press)
            .map(|(action, _)| action)
            .or_else(|| {
                self.bound().find_map(|(action, keys)| {
                    (action.acts_in_menu() && keys[..] == [press]).then_some(action)
                })
            })
    }

    /// Returns the motion that keys of two presses or more complete while a
    /// menu is open, such as `g g`, after the `waiting` keys that came first,
    /// and the keys that wait for another press, as the first `g` does
    pub fn menu_motion(&self, waiting: &[Press], press: Press) -> (Option<Action>, Vec<Press>) {
        let keys = [waiting, &[press]].concat();
        let done = self
            .motions()
            .find(|(_, bound)| *bound == keys)
            .map(|(action, _)| action);
        let longer = self
            .motions()
            .any(|(_, bound)| bound.len() > keys.len() && bound.starts_with(&keys));
        (
            done,
            if done.is_none() && longer {
                keys
            } else {
                Vec::new()
            },
        )
    }

    /// Returns the action of [`Action::AT_ONCE`] whose key is pressed,
    /// together with the key as it is bound: as pressed, or else without
    /// the modifiers held, so that an action bound to the key with those
    /// very modifiers takes it over one bound to the key alone
    pub(in crate::front::keymap) fn at_once_bound(&self, press: Press) -> Option<(Action, Press)> {
        [press, press.bare()].into_iter().find_map(|key| {
            Action::AT_ONCE
                .into_iter()
                .find(|&action| self.keys(action).iter().any(|keys| keys[..] == [key]))
                .map(|action| (action, key))
        })
    }
}
