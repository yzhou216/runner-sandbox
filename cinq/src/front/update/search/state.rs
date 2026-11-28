// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! State of a search, the line while it is open and the search accepted last

use super::line::Recall;
use crate::{back::CueId, front::Spot};

/// Search line while it is open
pub(in crate::front) struct Search {
    pub text: String,
    /// Whether the search goes up the list, as Vim's `?` does
    pub backward: bool,
    /// Selection as it was when the line opened, where cancelling the search
    /// puts it back and where the search looks first
    pub(super) began: Spot,
    /// Whether no row matches the text
    pub(super) missed: bool,
    /// Whether the line tells so, which it does for a while
    pub warned: bool,
    /// Searches that the keys of recall went through, while the text is one of
    /// them
    pub(super) recall: Option<Recall>,
    /// Collapsed groups that the preview expanded to show the match, which
    /// close again once the match moves elsewhere or the search is cancelled
    pub(super) opened: Vec<CueId>,
}

/// Search accepted last, which the keys of the next and previous match
/// repeat
#[derive(Default, Clone)]
pub(in crate::front) struct Searched {
    pub text: String,
    pub backward: bool,
    /// Searches accepted before, oldest first, which the keys of recall go
    /// through.  Nothing clears them.
    pub history: Vec<String>,
}

impl Search {
    pub(super) fn opened_at(began: Spot, backward: bool) -> Self {
        Self {
            text: String::new(),
            backward,
            began,
            missed: false,
            warned: false,
            recall: None,
            opened: Vec::new(),
        }
    }
}
