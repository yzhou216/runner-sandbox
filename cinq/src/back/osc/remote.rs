// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Commands that OSC messages give, at the addresses that QLab's OSC
//! dictionary names, so that remote controls made for QLab work as they are
//!
//! Each address may carry `/workspace/{id}` once before it, whatever the ID,
//! since one show is open.  A cue goes by its number as the cue list shows
//! it, which the front end looks up.  There are no wildcards.

#[cfg(test)]
mod tests;

use super::incoming::Message;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Remote {
    /// GO on the list in view
    Go,
    Panic,
    /// Pauses every running cue
    Pause,
    /// Resumes every paused cue
    Resume,
    Start(String),
    Stop(String),
    PauseCue(String),
    ResumeCue(String),
    /// Moves standby to the cue
    Playhead(String),
}

impl Remote {
    pub fn number(&self) -> Option<&str> {
        match self {
            Self::Start(number)
            | Self::Stop(number)
            | Self::PauseCue(number)
            | Self::ResumeCue(number)
            | Self::Playhead(number) => Some(number),
            Self::Go | Self::Panic | Self::Pause | Self::Resume => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reading {
    Command(Remote),
    /// Release of a button, which sends a zero after the one of its press
    /// and must not act again
    Released,
    Unknown,
}

impl Message {
    pub fn reading(&self) -> Reading {
        let segments: Vec<&str> = self.address.split('/').skip(1).collect();
        match route(&segments) {
            None => Reading::Unknown,
            Some(_) if self.arguments.len() == 1 && self.arguments[0].is_off() => Reading::Released,
            Some(remote) => Reading::Command(remote),
        }
    }
}

/// Returns the command that `segments` name.  The workspace prefix is taken
/// once, since a datagram of thousands of them would otherwise nest as deep
/// as the stack of the listener allows.
fn route(segments: &[&str]) -> Option<Remote> {
    let number = |number: &&str| number.trim().to_owned();
    let segments = match segments {
        ["workspace", _, rest @ ..] => rest,
        all => all,
    };
    match segments {
        ["go"] => Some(Remote::Go),
        ["panic"] => Some(Remote::Panic),
        ["pause"] => Some(Remote::Pause),
        ["resume"] => Some(Remote::Resume),
        ["cue", cue, "start"] => Some(Remote::Start(number(cue))),
        ["cue", cue, "stop"] => Some(Remote::Stop(number(cue))),
        ["cue", cue, "pause"] => Some(Remote::PauseCue(number(cue))),
        ["cue", cue, "resume"] => Some(Remote::ResumeCue(number(cue))),
        ["playhead", cue] => Some(Remote::Playhead(number(cue))),
        _ => None,
    }
}
