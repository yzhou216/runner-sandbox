// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Places where the keys of an action act

use super::Action;

impl Action {
    /// Returns whether a key of the action acts on the cue list, which the
    /// keys of an open menu do not
    pub(in crate::front::keymap) fn acts_in_cue_list(self) -> bool {
        !matches!(self, Action::Pick | Action::CloseMenu)
    }

    /// Returns whether a key of the action acts while a menu is open, where
    /// every other key is kept from acting
    pub(in crate::front::keymap) fn acts_in_menu(self) -> bool {
        self.acts_at_once()
            || matches!(
                self,
                Action::Cancel
                    | Action::Next
                    | Action::Previous
                    | Action::PageDown
                    | Action::PageUp
                    | Action::First
                    | Action::Last
                    | Action::Help
                    | Action::Pick
                    | Action::CloseMenu
            )
    }

    /// Returns whether keys of the two actions act in the same place, the
    /// cue list or an open menu.  Only keys that do can hide each other, so
    /// that a key may pick in a menu and scroll the list.
    pub(in crate::front::keymap) fn shares_place(self, other: Action) -> bool {
        (self.acts_in_cue_list() && other.acts_in_cue_list())
            || (self.acts_in_menu() && other.acts_in_menu())
    }

    /// Returns whether a key of the action acts while a closed drop-down
    /// list has the keyboard, beside those that act in every field.  The
    /// list has no text of its own, so that the keys of undo and redo act
    /// there, and so do those of [`Action::leaves_lists`].
    pub fn acts_in_lists(self) -> bool {
        self.acts_in_fields() || matches!(self, Action::Undo | Action::Redo) || self.leaves_lists()
    }

    /// Returns whether the action takes the keyboard out of a closed list, a
    /// checkbox, or a button that has it, so that the keys of hint mode, whose
    /// labels the list would swallow, and of the cue list, which a new cue
    /// selects, act next
    pub fn leaves_lists(self) -> bool {
        matches!(self, Action::Hint | Action::AddBelow | Action::AddAbove)
    }

    pub fn acts_in_fields(self) -> bool {
        self.acts_at_once()
            || matches!(
                self,
                Action::Cancel
                    | Action::NextField
                    | Action::PreviousField
                    | Action::Back
                    | Action::Forward
                    | Action::Open
                    | Action::Save
                    | Action::SaveAs
            )
    }

    /// Returns whether a key of the action acts while the help page is
    /// open.  The keys that scroll the page are the page's own, which
    /// `typed` handles before it asks.  The links of the page take labels.
    pub fn acts_over_help(self) -> bool {
        self.acts_at_once() || matches!(self, Action::Help | Action::CloseHelp | Action::Hint)
    }
}
