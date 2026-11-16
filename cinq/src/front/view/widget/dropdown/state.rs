// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! State that iced keeps for a drop-down list from one frame to the next

use iced::advanced::widget::{Tree, operation::Focusable};

use super::menu::Shared;

pub(super) struct State {
    pub(super) shared: Shared,
    /// Whether the list has the keyboard, which it loses as its menu opens and
    /// has again once a key closes the menu
    pub(super) focused: bool,
    /// Whether the cursor was over the list in the last frame, so that the
    /// list redraws when its look changes
    pub(super) over: bool,
    /// State of the widgets of the menu, which outlives each opening
    pub(super) menu: Tree,
}

impl State {
    pub(super) fn new() -> Self {
        Self {
            shared: Shared::default(),
            focused: false,
            over: false,
            menu: Tree::empty(),
        }
    }
}

impl Focusable for State {
    /// A key that closed the menu has handed the keyboard back already, though
    /// the list takes it only with the next event, so that a question asked
    /// right after the key finds the list holding it
    fn is_focused(&self) -> bool {
        self.focused || self.shared.is_returning()
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.shared.take_returning();
    }
}
