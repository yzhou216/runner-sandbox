// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether a round of sending got any light out, which the sidebar warns of
//! while it did not

use std::io;

use crate::warn;

#[derive(Debug, PartialEq)]
pub(super) enum Round {
    /// Nothing was due or possible to send, which tells nothing
    Idle,
    /// At least one packet left
    Out,
    /// No packet left, for the reason in words
    Stuck(String),
}

impl Round {
    /// Returns what a round told, given the interfaces that the computer
    /// had, whether anything was due, and how each send went.  A round
    /// that got one packet out is out, since a node that one interface
    /// reaches hears the lights, whatever the others do.
    pub(super) fn of(interfaces: usize, due: bool, sends: &[io::Result<usize>]) -> Self {
        if !due {
            return Self::Idle;
        }
        if interfaces == 0 {
            return Self::Stuck("no network interface".to_owned());
        }
        match (
            sends.iter().any(Result::is_ok),
            sends.iter().find_map(|s| s.as_ref().err()),
        ) {
            (true, _) => Self::Out,
            (false, Some(error)) => Self::Stuck(format!("not sent: {}", warn::words(error))),
            (false, None) => Self::Idle,
        }
    }

    /// Returns the problem to tell after this round, given the one told
    /// before, or nothing where the round changes none
    pub(super) fn told(&self, before: &Option<String>) -> Option<Option<String>> {
        let now = match self {
            Self::Idle => return None,
            Self::Out => None,
            Self::Stuck(words) => Some(words.clone()),
        };
        (now != *before).then_some(now)
    }
}
