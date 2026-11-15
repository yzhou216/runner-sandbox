// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Actions that keys can be bound to

mod description;
mod message;

use strum::{EnumString, IntoStaticStr, VariantArray};

use crate::front::message::Page;

/// [`Action::VARIANTS`] lists every action in the order of the help page,
/// in the groups that a user of Vim scans: GO, Panic, and standby, then
/// moving, then editing, then the files and the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum Action {
    Go,
    Panic,
    Standby,
    Cancel,
    Next,
    Previous,
    First,
    Last,
    HalfPageDown,
    HalfPageUp,
    PageDown,
    PageUp,
    ScrollDown,
    ScrollUp,
    TopRow,
    MiddleRow,
    BottomRow,
    RowToMiddle,
    RowToTop,
    RowToBottom,
    NextList,
    PreviousList,
    NextTab,
    PreviousTab,
    Back,
    Forward,
    Search,
    SearchBackward,
    NextMatch,
    PreviousMatch,
    Edit,
    EditStart,
    ChangeName,
    NextField,
    PreviousField,
    Pick,
    CloseMenu,
    AddBelow,
    AddAbove,
    Cut,
    Copy,
    PasteBelow,
    PasteAbove,
    Delete,
    MoveDown,
    MoveUp,
    Undo,
    Redo,
    Open,
    Save,
    SaveAs,
    Fullscreen,
    Help,
    CloseHelp,
}

impl Action {
    pub fn name(self) -> &'static str {
        self.into()
    }

    /// Actions that keep the show in hand wherever the keyboard is: their
    /// keys act alone, with any modifiers held, and over text fields, menus,
    /// and the help page alike
    pub(super) const AT_ONCE: [Action; 2] = [Action::Go, Action::Panic];

    /// Actions that apply to the rows that a motion reaches, as `d` and `y`
    /// do in Vim.  Each takes its operator key from a binding that doubles
    /// a key, such as `d d`.
    pub(super) const OPERATORS: [Action; 3] = [Action::Cut, Action::Copy, Action::Delete];

    /// Actions whose keys are the motions that follow an operator, each
    /// reaching the rows from the selection to where it would move it
    pub(super) const MOTIONS: [Action; 4] =
        [Action::Next, Action::Previous, Action::First, Action::Last];

    pub fn acts_at_once(self) -> bool {
        Self::AT_ONCE.contains(&self)
    }

    /// Returns whether a key of the action acts on the cue list, which the
    /// keys of an open menu do not
    pub(super) fn acts_in_list(self) -> bool {
        !matches!(self, Action::Pick | Action::CloseMenu)
    }

    /// Returns whether a key of the action acts while a menu is open, where
    /// every other key is kept from acting
    pub(super) fn acts_in_menu(self) -> bool {
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
    pub(super) fn shares_place(self, other: Action) -> bool {
        (self.acts_in_list() && other.acts_in_list())
            || (self.acts_in_menu() && other.acts_in_menu())
    }

    /// Describes the operator `self` applied over the rows that `motion`
    /// reaches
    pub fn over(self, motion: Action) -> String {
        let verb = match self {
            Action::Copy => "Copy",
            Action::Cut => "Cut",
            _ => "Delete",
        };
        let rows = match motion {
            Action::Next => "the selected cue or output and the next",
            Action::Previous => "the selected cue or output and the previous",
            Action::First => "the cues or outputs from the selection up to the first",
            _ => "the cues or outputs from the selection down to the last",
        };
        format!("{verb} {rows}")
    }

    /// Returns whether a key of the action that is held down acts only
    /// once: those of [`Action::AT_ONCE`], so that holding the key of Panic
    /// does not cut a panic short, and that of fullscreen, which would
    /// otherwise toggle the stage window back and forth
    pub fn acts_once_a_press(self) -> bool {
        self.acts_at_once() || self == Action::Fullscreen
    }

    pub(super) fn acts_in_fields(self) -> bool {
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
    /// `typed` handles before it asks.
    pub fn acts_over_help(self) -> bool {
        self.acts_at_once() || matches!(self, Action::Help | Action::CloseHelp)
    }

    /// Returns the page that the action moves by, if it is one of the four
    /// that do
    pub fn page(self) -> Option<Page> {
        match self {
            Action::HalfPageDown => Some(Page::HalfDown),
            Action::HalfPageUp => Some(Page::HalfUp),
            Action::PageDown => Some(Page::WholeDown),
            Action::PageUp => Some(Page::WholeUp),
            _ => None,
        }
    }
}
