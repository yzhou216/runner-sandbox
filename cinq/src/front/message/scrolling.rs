// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Distances and heights in the view of the cue list that the keys of Vim
//! name

/// Distance and direction of a move by a page, as Vim names them
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    HalfDown,
    HalfUp,
    WholeDown,
    WholeUp,
}

impl Page {
    pub fn is_half(self) -> bool {
        matches!(self, Self::HalfDown | Self::HalfUp)
    }

    /// Returns 1 for a move downwards and -1 for one upwards
    pub fn sign(self) -> isize {
        match self {
            Self::HalfDown | Self::WholeDown => 1,
            Self::HalfUp | Self::WholeUp => -1,
        }
    }
}

/// Height in the view of the cue list, which `H`, `M`, and `L` select a row
/// at, and which `z t`, `z z`, and `z b` put the selected row at
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Height {
    Top,
    Middle,
    Bottom,
}

/// What a key does to the groups of the cue list, as Vim's keys do to folds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fold {
    /// Collapses the selected group, or the group that holds the selected
    /// cue where that is no expanded group, as `z c` closes a fold
    Close,
    /// Expands the selected group, as `z o` opens a fold
    Open,
    /// Expands the selected group where it is collapsed, and collapses it
    /// otherwise, as `z a` does to a fold
    Toggle,
    /// Collapses every group of the list, as `z M` closes every fold
    CloseAll,
    /// Expands every group of the list, as `z R` opens every fold
    OpenAll,
}
