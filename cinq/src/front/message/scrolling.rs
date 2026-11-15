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
