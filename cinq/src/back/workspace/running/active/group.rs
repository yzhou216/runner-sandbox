// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Run of a group, whose span the cues that it holds set

use super::{Active, Effect};

impl Active {
    /// Returns the run of a group with its action lasting `length` seconds
    /// after its pre-wait, or without an end where `endless`, as the cues
    /// that the group holds have it.  Any other run stays as it is.
    pub(in crate::back::workspace::running) fn spanning(self, length: f64, endless: bool) -> Self {
        match self.effect {
            Effect::Group { .. } => Self {
                length,
                effect: Effect::Group { endless },
                ..self
            },
            _ => self,
        }
    }

    /// Returns whether the run is that of a group
    pub(in crate::back::workspace::running) fn holds(&self) -> bool {
        matches!(self.effect, Effect::Group { .. })
    }
}
