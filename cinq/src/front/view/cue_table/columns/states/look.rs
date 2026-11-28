// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! What decides how a row of the State column reads besides the cue itself

use std::collections::BTreeSet;

use crate::back::{OutputId, Problem};

/// Empty set that [`Look::default`] points at, as a reference to a new one
/// would not outlive the call
static NONE: BTreeSet<String> = BTreeSet::new();

/// Circumstances of a row that tell its State column what to say
#[derive(Clone, Copy)]
pub(in crate::front::view::cue_table) struct Look<'a> {
    pub(super) next: bool,
    pub(super) selected: bool,
    pub(super) hidden: Option<Problem<'a>>,
    pub(super) failing: &'a [OutputId],
    pub(super) unreached: &'a BTreeSet<String>,
}

impl Default for Look<'_> {
    fn default() -> Self {
        Self {
            next: false,
            selected: false,
            hidden: None,
            failing: &[],
            unreached: &NONE,
        }
    }
}

impl<'a> Look<'a> {
    pub(in crate::front::view::cue_table) fn with_next(self, next: bool) -> Self {
        Self { next, ..self }
    }

    /// Returns the look of a row that is `selected`, where red turns lighter
    pub(in crate::front::view::cue_table) fn with_selected(self, selected: bool) -> Self {
        Self { selected, ..self }
    }

    pub(in crate::front::view::cue_table) fn with_hidden(
        self,
        hidden: Option<Problem<'a>>,
    ) -> Self {
        Self { hidden, ..self }
    }

    pub(in crate::front::view::cue_table) fn with_failing(self, failing: &'a [OutputId]) -> Self {
        Self { failing, ..self }
    }

    pub(in crate::front::view::cue_table) fn with_unreached(
        self,
        unreached: &'a BTreeSet<String>,
    ) -> Self {
        Self { unreached, ..self }
    }
}
