// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pages of the inspector

use strum::{Display, VariantArray};

/// Page of the inspector, which goes by its name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display, VariantArray)]
pub enum Tab {
    #[default]
    Basics,
    #[strum(to_string = "Time & Loops")]
    Time,
    Levels,
    #[strum(to_string = "Light Patch")]
    Patch,
}

impl Tab {
    /// Returns the tab `offset` places to the right, wrapping around at
    /// either end
    pub fn stepped(self, offset: isize) -> Self {
        let count = Self::VARIANTS.len() as isize;
        // Reduced first, since a count as large as a number can hold would
        // overflow the sum
        Self::VARIANTS[(self as isize + offset.rem_euclid(count)).rem_euclid(count) as usize]
    }

    /// Returns the tab with a number, counted from one, or the last tab
    /// when there are fewer
    pub fn numbered(number: usize) -> Self {
        Self::VARIANTS[number.saturating_sub(1).min(Self::VARIANTS.len() - 1)]
    }
}
