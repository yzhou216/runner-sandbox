// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Actions that keys can be bound to

mod description;
mod message;
mod places;

use strum::{EnumString, IntoStaticStr, VariantArray};

use crate::front::message::Page;
pub use description::Rows;

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
    Collapse,
    Expand,
    ToggleGroup,
    CollapseAll,
    ExpandAll,
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
    Hint,
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
    Indent,
    Outdent,
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
    /// Returns every action in the order of the help page and of the table
    /// of cinq(5), which is that of [`Action::VARIANTS`] except that the
    /// menu actions follow `Cancel`, where the keys that act in a menu sit
    /// together.  The variants keep them late, since the earlier of two
    /// actions that clash keeps the key, and a menu key must not take it
    /// from a move.
    pub fn listed() -> impl Iterator<Item = Action> {
        const MENU: [Action; 2] = [Action::Pick, Action::CloseMenu];
        Self::VARIANTS
            .iter()
            .copied()
            .filter(|action| !MENU.contains(action))
            .flat_map(|action| {
                let after: &[Action] = if action == Action::Cancel { &MENU } else { &[] };
                std::iter::once(action).chain(after.iter().copied())
            })
    }

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
    pub(super) const OPERATORS: [Action; 5] = [
        Action::Cut,
        Action::Copy,
        Action::Delete,
        Action::Indent,
        Action::Outdent,
    ];

    /// Actions whose keys are the motions that follow an operator, each
    /// reaching the rows from the selection to where it would move it
    pub(super) const MOTIONS: [Action; 4] =
        [Action::Next, Action::Previous, Action::First, Action::Last];

    pub fn acts_at_once(self) -> bool {
        Self::AT_ONCE.contains(&self)
    }

    /// Describes the operator `self` applied over the rows that `motion`
    /// reaches, which are `rows`.  A `count` typed so far moves the motion
    /// by that many rows, or to that row for `First` and `Last`, as the
    /// operator will take it.
    pub fn over(self, motion: Action, rows: Rows, count: Option<usize>) -> String {
        let verb = match self {
            Action::Copy => "Copy",
            Action::Cut => "Cut",
            Action::Indent => "Indent",
            Action::Outdent => "Outdent",
            _ => "Delete",
        };
        let (one, many) = rows.nouns();
        let by = count.map_or(String::new(), |count| format!(" {count}"));
        let reached = match (motion, count) {
            (Action::Next, _) => format!("the selected {one} and the next{by}"),
            (Action::Previous, _) => format!("the selected {one} and the previous{by}"),
            (Action::First | Action::Last, Some(count)) => {
                format!("the {many} from the selection to row {count}")
            }
            (Action::First, _) => format!("the {many} from the selection up to the first"),
            _ => format!("the {many} from the selection down to the last"),
        };
        format!("{verb} {reached}")
    }

    /// Returns whether a key of the action that is held down acts only
    /// once: those of [`Action::AT_ONCE`], so that holding the key of Panic
    /// does not cut a panic short, those of fullscreen, hint mode, and
    /// help, which would otherwise toggle the stage window, the labels, or
    /// the page back and forth, and those of open, save, and save as, which
    /// would otherwise open a file dialog on every repeat
    pub fn acts_once_a_press(self) -> bool {
        self.acts_at_once()
            || matches!(
                self,
                Action::Fullscreen
                    | Action::Hint
                    | Action::Help
                    | Action::Open
                    | Action::Save
                    | Action::SaveAs
            )
    }

    /// Returns whether a key of the action only moves the selection or the
    /// list in view, which leaves the key to paste what a copy or cut took
    /// where it was, since pasting into the other list takes such moves first
    pub fn only_moves(self) -> bool {
        matches!(
            self,
            Action::Next
                | Action::Previous
                | Action::First
                | Action::Last
                | Action::HalfPageDown
                | Action::HalfPageUp
                | Action::PageDown
                | Action::PageUp
                | Action::ScrollDown
                | Action::ScrollUp
                | Action::TopRow
                | Action::MiddleRow
                | Action::BottomRow
                | Action::NextList
                | Action::PreviousList
                | Action::NextMatch
                | Action::PreviousMatch
                | Action::Back
                | Action::Forward
        )
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
